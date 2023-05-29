use amiquip::{Connection, ConsumerMessage, ConsumerOptions, QueueDeclareOptions};
use libp2p::{
    core::upgrade,
    futures::StreamExt,
    mplex,
    noise::{Keypair, NoiseConfig, X25519Spec},
    swarm::{Swarm, SwarmBuilder},
    tcp::TokioTcpConfig,
    Transport,
};
use log::{error, info};

use std::env;
use std::time::Duration;
use tokio::{
    // io::{stdin, AsyncBufReadExt, BufReader},
    select,
    spawn,
    sync::mpsc,
    time::sleep,
};

mod mining_and_blocks;
mod utils;
mod p2p;

use mining_and_blocks::{block::Block, app::App};
use p2p::app_behaviour as p2p_app;
use p2p::event_handlers as p2p_event_handlers;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    info!("Peer Id: {}", p2p_app::PEER_ID.clone());
    let (response_sender, mut response_rcv) = mpsc::unbounded_channel();
    let (init_sender, mut init_rcv) = mpsc::unbounded_channel();
    let (rabbit_sender, mut rabbit_rcv) = mpsc::unbounded_channel();
    let (mine_sender, mut mine_rcv) = mpsc::unbounded_channel();

    let auth_keys = Keypair::<X25519Spec>::new()
        .into_authentic(&p2p_app::KEYS)
        .expect("can create auth keys");

    let transp = TokioTcpConfig::new()
        .upgrade(upgrade::Version::V1)
        .authenticate(NoiseConfig::xx(auth_keys).into_authenticated())
        .multiplex(mplex::MplexConfig::new())
        .boxed();

    let behaviour = p2p_app::AppBehaviour::new(
        App::new(),
        response_sender,
        init_sender.clone(),
        mine_sender.clone(),
    )
    .await;

    let mut swarm = SwarmBuilder::new(transp, behaviour, *p2p_app::PEER_ID)
        .executor(Box::new(|fut| {
            spawn(fut);
        }))
        .build();

    // let mut stdin = BufReader::new(stdin()).lines();

    Swarm::listen_on(
        &mut swarm,
        "/ip4/0.0.0.0/tcp/8080"
            .parse()
            .expect("can get a local socket"),
    )
    .expect("swarm can be started");

    spawn(async move {
        sleep(Duration::from_secs(1)).await;
        info!("sending init event");
        init_sender.send(true).expect("can send init event");
    });

    spawn(async move {
        loop {
            sleep(Duration::from_secs(30)).await;
            info!("sending consume event");
            rabbit_sender.send(true).expect("can send init event");
        }
    });

    let rabbit_dsn = env::var("RABBITMQ_DSN")
        .unwrap_or_else(|_| "amqp://guest:guest@localhost:5672".to_string());
    // Open connection.
    let mut connection = match Connection::insecure_open(&rabbit_dsn) {
        Ok(x) => x,
        Err(_) => panic!("cannot connect to rabbit"),
    };

    // Open a channel - None says let the library choose the channel ID.
    let channel = match connection.open_channel(None) {
        Ok(x) => x,
        Err(_) => panic!("cannot connect to rabbit channel"),
    };

    // Declare the "hello" queue.
    let queue = match channel.queue_declare(
        "blockchain-data",
        QueueDeclareOptions {
            durable: true,
            ..QueueDeclareOptions::default()
        },
    ) {
        Ok(x) => x,
        Err(_) => panic!("cannot connect to rabbit queue"),
    };
    let consumer = match queue.consume(ConsumerOptions::default()) {
        Ok(x) => x,
        Err(_) => panic!("cannot connect to rabbit queue"),
    };

    loop {
        let evt = {
            select! {
                mine = mine_rcv.recv() => {
                    Some(p2p_app::EventType::Mine(mine.expect("mine exists")))
                }
                response = response_rcv.recv() => {
                    Some(p2p_app::EventType::LocalChainResponse(response.expect("response exists")))
                },
                _init = init_rcv.recv() => {
                    Some(p2p_app::EventType::Init)
                }
                _consume = rabbit_rcv.recv() => {
                    Some(p2p_app::EventType::ConsumeUserData)
                }
                event = swarm.select_next_some() => {
                    info!("Unhandled Swarm Event: {:?}", event);
                    None
                },
            }
        };

        if let Some(event) = evt {
            match event {
                p2p_app::EventType::Init => {
                    let peers = p2p_event_handlers::get_list_peers(&swarm);
                    swarm.behaviour_mut().app.genesis();

                    info!("connected nodes: {}", peers.len());
                    if !peers.is_empty() {
                        let req = p2p_app::LocalChainRequest {
                            from_peer_id: peers
                                .iter()
                                .last()
                                .expect("at least one peer")
                                .to_string(),
                        };

                        let json = serde_json::to_string(&req).expect("can jsonify request");
                        swarm
                            .behaviour_mut()
                            .floodsub
                            .publish(p2p_app::CHAIN_TOPIC.clone(), json.as_bytes());
                    }
                }
                p2p_app::EventType::LocalChainResponse(resp) => {
                    let json = serde_json::to_string(&resp).expect("can jsonify response");
                    swarm
                        .behaviour_mut()
                        .floodsub
                        .publish(p2p_app::CHAIN_TOPIC.clone(), json.as_bytes());
                }
                p2p_app::EventType::ConsumeUserData => {
                    let data = match consumer.receiver().recv() {
                        Ok(x) => x,
                        Err(_) => panic!("o rety kotlety"),
                    };
                    let data = match data {
                        ConsumerMessage::Delivery(delivery) => {
                            let body_bytes = delivery.body.to_vec();
                            consumer.ack(delivery).expect("cant accept delivery");
                            String::from_utf8_lossy(&body_bytes).to_string()
                        }
                        other => {
                            println!("Consumer ended: {:?}", other);
                            break;
                        }
                    };
                    println!("COLLECTED {}", data);
                    let json = serde_json::to_string(&data).expect("can jsonify response");
                    swarm
                        .behaviour_mut()
                        .floodsub
                        .publish(p2p_app::DATA_TOPIC.clone(), json.as_bytes());
                    if let Err(e) = swarm.behaviour_mut().mine_sender.send(data) {
                        error!("error sending response via channel, {}", e);
                    }
                }
                p2p_app::EventType::Mine(data) => {
                    println!("MINING: {}", data);
                    let behaviour = swarm.behaviour_mut();
                    let latest_block = behaviour
                        .app
                        .blocks
                        .last()
                        .expect("there is at least one block");
                    let block = Block::new(
                        latest_block.id + 1,
                        latest_block.hash.clone(),
                        data.to_owned(),
                    );
                    let json = serde_json::to_string(&block).expect("can jsonify request");
                    behaviour.app.blocks.push(block);
                    info!("broadcasting new block");
                    behaviour
                        .floodsub
                        .publish(p2p_app::BLOCK_TOPIC.clone(), json.as_bytes());
                }
            }
        }
    }
}

// demo z uzycia redisa
// dodaje wartosc do bazy
// sprawdzic to mozna przez wejscie w terminal redis_1
// > redis-cli
// > KEYS *
// use redis::{Client, RedisResult, Commands};

// fn main() -> RedisResult<()> {
//     // Read Redis DSN from system environment
//     let redis_dsn = std::env::var("REDIS_DSN").expect("REDIS_DSN environment variable not set");

//     // Connect to Redis
//     let client = Client::open(redis_dsn)?;
//     let mut con: redis::Connection = client.get_connection()?;

//     let key = "mykey";
//     let value = "myvalue";
//     let _: () = redis::cmd("SET").arg(key).arg(value).execute(&mut con);
//     Ok(())
// }

// //rabbitmq
// // Port of https://www.rabbitmq.com/tutorials/tutorial-one-python.html. Run this
// // in one shell, and run the hello_world_publish example in another.
// use std::env;
// use amiquip::{Connection, ConsumerMessage, ConsumerOptions, QueueDeclareOptions, Result};
// // use redis::{Client, RedisResult, ConnectionLike, Commands};

// fn main() -> Result<()> {
//     // Retrieve the RabbitMQ DSN and redis DSN from the RABBIT_DSN and REDIS_DSN environment variable.
//     let rabbit_dsn = env::var("RABBITMQ_DSN").unwrap_or_else(|_| "amqp://guest:guest@localhost:5672".to_string());
//     // let redis_dsn = std::env::var("REDIS_DSN").expect("REDIS_DSN environment variable not set");

//     // Open connection.
//     let mut connection = Connection::insecure_open(&rabbit_dsn)?;

//     // Open a channel - None says let the library choose the channel ID.
//     let channel = connection.open_channel(None)?;

//     // Declare the "hello" queue.
//     let queue = channel.queue_declare("blockchain-data", QueueDeclareOptions {
//         durable: true,
//         ..QueueDeclareOptions::default()
//     },)?;

//     // Start a consumer.
//     let consumer = queue.consume(ConsumerOptions::default())?;
//     println!("Waiting for messages. Press Ctrl-C to exit.");

//     // Connect to Redis
//     // let client = Client::open(redis_dsn)?;
//     // let mut redis_connection = client.get_connection()?;

//     for (i, message) in consumer.receiver().iter().enumerate() {
//         match message {
//             ConsumerMessage::Delivery(delivery) => {
//                 let body = String::from_utf8_lossy(&delivery.body);
//                 println!("({:>3}) Received [{}]", i, body);
//                 // let _ : () = redis_connection.set("my_key", body.to_string())?;

//                 consumer.ack(delivery)?;
//             }
//             other => {
//                 println!("Consumer ended: {:?}", other);
//                 break;
//             }
//         }
//     }

//     connection.close()
// }
