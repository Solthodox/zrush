use clap::{arg, command, Command};
mod block;
mod node;
mod signature;
mod transaction;
use node::{core::run_node, node::create_new_blockchain};

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
                .arg(arg!([OPTION1])),
        )
        .subcommand(
            Command::new("createblockchain")
                .about("Create a new blockchain")
                .arg(arg!([OPTION1])),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("node", _sub_matches)) => {
            run_node("50051").await;
        }
        Some(("createblockchain", _sub_matches)) => {
            create_new_blockchain();
            run_node("50051").await;
        }
        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
    }
}
