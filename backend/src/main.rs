// use chrono::prelude::*;
// use libp2p::{
//     core::upgrade,
//     futures::StreamExt,
//     mplex,
//     noise::{Keypair, NoiseConfig, X25519Spec},
//     swarm::{Swarm, SwarmBuilder},
//     tcp::TokioTcpConfig,
//     Transport,
// };
// use log::{error, info, warn};
// use serde::{Deserialize, Serialize};
// use sha2::{Digest, Sha256};
// use std::time::Duration;
// use tokio::{
//     // io::{stdin, AsyncBufReadExt, BufReader},
//     select, spawn,
//     sync::mpsc,
//     time::sleep,
// };

// const DIFFICULTY_PREFIX: &str = "00";

// mod p2p;

// pub struct App {
//     pub blocks: Vec<Block>,
// }

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct Block {
//     pub id: u64,
//     pub hash: String,
//     pub previous_hash: String,
//     pub timestamp: i64,
//     pub data: String,
//     pub nonce: u64,
// }

// impl Block {
//     pub fn new(id: u64, previous_hash: String, data: String) -> Self {
//         let now = Utc::now();
//         let (nonce, hash) = mine_block(id, now.timestamp(), &previous_hash, &data);
//         Self {
//             id,
//             hash,
//             timestamp: now.timestamp(),
//             previous_hash,
//             data,
//             nonce,
//         }
//     }
// }

// fn calculate_hash(id: u64, timestamp: i64, previous_hash: &str, data: &str, nonce: u64) -> Vec<u8> {
//     let data = serde_json::json!({
//         "id": id,
//         "previous_hash": previous_hash,
//         "data": data,
//         "timestamp": timestamp,
//         "nonce": nonce
//     });
//     let mut hasher = Sha256::new();
//     hasher.update(data.to_string().as_bytes());
//     hasher.finalize().as_slice().to_owned()
// }

// fn mine_block(id: u64, timestamp: i64, previous_hash: &str, data: &str) -> (u64, String) {
//     info!("mining block...");
//     let mut nonce = 0; // replace with random number

//     loop {
//         let hash = calculate_hash(id, timestamp, previous_hash, data, nonce);
//         let binary_hash = hash_to_binary_representation(&hash);
//         if binary_hash.starts_with(DIFFICULTY_PREFIX) {
//             info!(
//                 "mined! nonce: {}, hash: {}, binary hash: {}",
//                 nonce,
//                 hex::encode(&hash),
//                 binary_hash
//             );
//             return (nonce, hex::encode(hash));
//         }
//         nonce += 1; // replace with random number
//     }
// }

// fn hash_to_binary_representation(hash: &[u8]) -> String {
//     let mut res: String = String::default();
//     for c in hash {
//         res.push_str(&format!("{:b}", c));
//     }
//     res
// }

// impl App {
//     fn new() -> Self {
//         Self { blocks: vec![] }
//     }

//     fn genesis(&mut self) {
//         let genesis_block = Block {
//             id: 0,
//             timestamp: Utc::now().timestamp(),
//             previous_hash: String::from("genesis"),
//             data: String::from("genesis!"),
//             nonce: 2836,
//             hash: "0000f816a87f806bb0073dcf026a64fb40c946b5abee2573702828694d5b4c43".to_string(),
//         };
//         self.blocks.push(genesis_block);
//     }

//     fn try_add_block(&mut self, block: Block) {
//         let latest_block = self.blocks.last().expect("there is at least one block");
//         if self.is_block_valid(&block, latest_block) {
//             self.blocks.push(block);
//         } else {
//             error!("could not add block - invalid");
//         }
//     }

//     fn is_block_valid(&self, block: &Block, previous_block: &Block) -> bool {
//         if block.previous_hash != previous_block.hash {
//             warn!("block with id: {} has wrong previous hash", block.id);
//             return false;
//         } else if !hash_to_binary_representation(
//             &hex::decode(&block.hash).expect("can decode from hex"),
//         )
//         .starts_with(DIFFICULTY_PREFIX)
//         {
//             warn!("block with id: {} has invalid difficulty", block.id);
//             return false;
//         } else if block.id != previous_block.id + 1 {
//             warn!(
//                 "block with id: {} is not the next block after the latest: {}",
//                 block.id, previous_block.id
//             );
//             return false;
//         } else if hex::encode(calculate_hash(
//             block.id,
//             block.timestamp,
//             &block.previous_hash,
//             &block.data,
//             block.nonce,
//         )) != block.hash
//         {
//             warn!("block with id: {} has invalid hash", block.id);
//             return false;
//         }
//         true
//     }

//     fn is_chain_valid(&self, chain: &[Block]) -> bool {
//         for i in 0..chain.len() {
//             if i == 0 {
//                 continue;
//             }
//             let first = chain.get(i - 1).expect("has to exist");
//             let second = chain.get(i).expect("has to exist");
//             if !self.is_block_valid(second, first) {
//                 return false;
//             }
//         }
//         true
//     }

//     // We always choose the longest valid chain
//     fn choose_chain(&mut self, local: Vec<Block>, remote: Vec<Block>) -> Vec<Block> {
//         let is_local_valid = self.is_chain_valid(&local);
//         let is_remote_valid = self.is_chain_valid(&remote);

//         if is_local_valid && is_remote_valid {
//             let ll = local.len();
//             let rl = remote.len();
            
//             // added better for resolve method - if they are the same length use one with higher nonce
//             if ll == rl {
//                 let local_latest_block = local.last().expect("there is at least one block");
//                 let remote_latest_block = remote.last().expect("there is at least one block");
                
//                 return if local_latest_block.nonce > remote_latest_block.nonce {local} else {remote};
//             } 
//             else if ll > rl {
//                 local
//             } else {
//                 remote
//             }
//         } else if is_remote_valid && !is_local_valid {
//             remote
//         } else if !is_remote_valid && is_local_valid {
//             local
//         } else {
//             panic!("local and remote chains are both invalid");
//         }
//     }
// }

// #[tokio::main]
// async fn main() {
//     pretty_env_logger::init();

//     info!("Peer Id: {}", p2p::PEER_ID.clone());
//     let (response_sender, mut response_rcv) = mpsc::unbounded_channel();
//     let (init_sender, mut init_rcv) = mpsc::unbounded_channel();

//     let auth_keys = Keypair::<X25519Spec>::new()
//         .into_authentic(&p2p::KEYS)
//         .expect("can create auth keys");

//     let transp = TokioTcpConfig::new()
//         .upgrade(upgrade::Version::V1)
//         .authenticate(NoiseConfig::xx(auth_keys).into_authenticated())
//         .multiplex(mplex::MplexConfig::new())
//         .boxed();

//     let behaviour = p2p::AppBehaviour::new(App::new(), response_sender, init_sender.clone()).await;

//     let mut swarm = SwarmBuilder::new(transp, behaviour, *p2p::PEER_ID)
//         .executor(Box::new(|fut| {
//             spawn(fut);
//         }))
//         .build();

//     // let mut stdin = BufReader::new(stdin()).lines();

//     Swarm::listen_on(
//         &mut swarm,
//         "/ip4/0.0.0.0/tcp/8080"
//             .parse()
//             .expect("can get a local socket"),
//     )
//     .expect("swarm can be started");

//     spawn(async move {
//         sleep(Duration::from_secs(1)).await;
//         info!("sending init event");
//         init_sender.send(true).expect("can send init event");
//     });

//     loop {
//         let evt = {
//             select! {
//                 // line = stdin.next_line() => Some(p2p::EventType::Input(line.expect("can get line").expect("can read line from stdin"))),
//                 response = response_rcv.recv() => {
//                     Some(p2p::EventType::LocalChainResponse(response.expect("response exists")))
//                 },
//                 _init = init_rcv.recv() => {
//                     Some(p2p::EventType::Init)
//                 }
//                 event = swarm.select_next_some() => {
//                     info!("Unhandled Swarm Event: {:?}", event);
//                     None
//                 },
//             }
//         };

//         if let Some(event) = evt {
//             match event {
//                 p2p::EventType::Init => {
//                     let peers = p2p::get_list_peers(&swarm);
//                     swarm.behaviour_mut().app.genesis();

//                     info!("connected nodes: {}", peers.len());
//                     if !peers.is_empty() {
//                         let req = p2p::LocalChainRequest {
//                             from_peer_id: peers
//                                 .iter()
//                                 .last()
//                                 .expect("at least one peer")
//                                 .to_string(),
//                         };

//                         let json = serde_json::to_string(&req).expect("can jsonify request");
//                         swarm
//                             .behaviour_mut()
//                             .floodsub
//                             .publish(p2p::CHAIN_TOPIC.clone(), json.as_bytes());
//                     }
//                 }
//                 p2p::EventType::LocalChainResponse(resp) => {
//                     let json = serde_json::to_string(&resp).expect("can jsonify response");
//                     swarm
//                         .behaviour_mut()
//                         .floodsub
//                         .publish(p2p::CHAIN_TOPIC.clone(), json.as_bytes());
//                 }
//                 // p2p::EventType::Input(line) => match line.as_str() {
//                 //     "ls p" => p2p::handle_print_peers(&swarm),
//                 //     cmd if cmd.starts_with("ls c") => p2p::handle_print_chain(&swarm),
//                 //     cmd if cmd.starts_with("create b") => p2p::handle_create_block(cmd, &mut swarm),
//                 //     _ => error!("unknown command"),
//                 // },
//             }
//         }
//     }
// }


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

//rabbitmq
// Port of https://www.rabbitmq.com/tutorials/tutorial-one-python.html. Run this
// in one shell, and run the hello_world_publish example in another.
use std::env;
use amiquip::{Connection, ConsumerMessage, ConsumerOptions, QueueDeclareOptions, Result};
// use redis::{Client, RedisResult, ConnectionLike, Commands};

fn main() -> Result<()> {
    // Retrieve the RabbitMQ DSN and redis DSN from the RABBIT_DSN and REDIS_DSN environment variable.
    let rabbit_dsn = env::var("RABBITMQ_DSN").unwrap_or_else(|_| "amqp://guest:guest@localhost:5672".to_string());
    // let redis_dsn = std::env::var("REDIS_DSN").expect("REDIS_DSN environment variable not set");

    // Open connection.
    let mut connection = Connection::insecure_open(&rabbit_dsn)?;

    // Open a channel - None says let the library choose the channel ID.
    let channel = connection.open_channel(None)?;

    // Declare the "hello" queue.
    let queue = channel.queue_declare("blockchain-data", QueueDeclareOptions {
        durable: true,
        ..QueueDeclareOptions::default()
    },)?;

    // Start a consumer.
    let consumer = queue.consume(ConsumerOptions::default())?;
    println!("Waiting for messages. Press Ctrl-C to exit.");

    // Connect to Redis
    // let client = Client::open(redis_dsn)?;
    // let mut redis_connection = client.get_connection()?;

    for (i, message) in consumer.receiver().iter().enumerate() {
        match message {
            ConsumerMessage::Delivery(delivery) => {
                let body = String::from_utf8_lossy(&delivery.body);
                println!("({:>3}) Received [{}]", i, body);
                // let _ : () = redis_connection.set("my_key", body.to_string())?;

                consumer.ack(delivery)?;
            }
            other => {
                println!("Consumer ended: {:?}", other);
                break;
            }
        }
    }

    connection.close()
}
