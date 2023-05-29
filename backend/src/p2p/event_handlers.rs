use libp2p::swarm::Swarm;
use log::info;
use std::collections::HashSet;
use super::app_behaviour::AppBehaviour;

pub fn get_list_peers(swarm: &Swarm<AppBehaviour>) -> Vec<String> {
    info!("Discovered Peers:");
    let nodes = swarm.behaviour().mdns.discovered_nodes();
    let mut unique_peers = HashSet::new();
    for peer in nodes {
        unique_peers.insert(peer);
    }
    unique_peers.iter().map(|p| p.to_string()).collect()
}

// pub fn handle_print_peers(swarm: &Swarm<AppBehaviour>) {
//     let peers = get_list_peers(swarm);
//     peers.iter().for_each(|p| info!("{}", p));
// }

// pub fn handle_print_chain(swarm: &Swarm<AppBehaviour>) {
//     info!("Local Blockchain:");
//     let pretty_json =
//         serde_json::to_string_pretty(&swarm.behaviour().app.blocks).expect("can jsonify blocks");
//     info!("{}", pretty_json);
// }

// //TODO broadcast records so everyone can mine it
// pub fn handle_create_block(cmd: &str, swarm: &mut Swarm<AppBehaviour>) {
//     if let Some(data) = cmd.strip_prefix("create b") {
//         let behaviour = swarm.behaviour_mut();
//         let latest_block = behaviour
//             .app
//             .blocks
//             .last()
//             .expect("there is at least one block");
//         let block = Block::new(
//             latest_block.id + 1,
//             latest_block.hash.clone(),
//             data.to_owned(),
//         );
//         let json = serde_json::to_string(&block).expect("can jsonify request");
//         behaviour.app.blocks.push(block);
//         info!("broadcasting new block");
//         behaviour
//             .floodsub
//             .publish(BLOCK_TOPIC.clone(), json.as_bytes());
//     }
// }
