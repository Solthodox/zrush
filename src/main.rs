use core::sync;

use clap::{arg, command, Command};
mod block;
mod node;
mod signature;
mod transaction;
use node::{core::run_node, node::{create_new_blockchain, sync_node}};

#[tokio::main]
async fn main() {
    let matches = command!()
        .name("zrush")
        .author("mrwojack@proton.me")
        .about("A ZKP-based public distributed ledger")
        .propagate_version(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("node")
                .about("Run a node")
                .arg(arg!(-p <PORT> "Sets the port to run on").required(false))
        )
        .subcommand(
            Command::new("createblockchain")
                .about("Create a new blockchain")
        )
        .subcommand(
            Command::new("sync")
                .about("Sync to a new blockchain")
                .arg(arg!(-n <NODE_ADDRESS> "Sets the boot node to sync from").required(true))
        )
        .get_matches();

    match matches.subcommand() {
        Some(("node", _sub_matches)) => {
            run_node("50051").await;
        }
        Some(("sync", _sub_matches)) => {
            sync_node(String::from("127.0.0.1:50051")).await.expect("failed to sync");
            run_node("50052").await;
        }
        Some(("createblockchain", _sub_matches)) => {
            create_new_blockchain();
            run_node("50051").await;
        }
        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
    }
}
