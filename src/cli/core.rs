use clap::{Command, arg, command};

use crate::node::core::{run_node, sync_node, create_new_blockchain};
use std::process as runtime;

pub async fn run_cli() {
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
                .arg(arg!(-p --port <PORT> "Sets the port to run on").required(false)),
        )
        .subcommand(Command::new("createblockchain").about("Create a new blockchain"))
        .subcommand(
            Command::new("sync")
                .about("Sync to a new blockchain")
                .arg(
                    arg!(-b --boot <NODE_ADDRESS> "Sets the boot node to sync from").required(true),
                )
                .arg(arg!(-p --port <PORT> "Sets the boot node to sync from").required(false)),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("node", _sub_matches)) => {
            if let Some(port) = _sub_matches.get_one::<String>("port") {
                run_node(port.to_string()).await.unwrap_or_else(|err| {
                    eprintln!("{:?}", err);
                    runtime::exit(1);
                });
            }
            run_node(String::from("50051")).await.unwrap_or_else(|err| {
                eprintln!("{:?}", err);
                runtime::exit(1);
            });
        }
        Some(("sync", _sub_matches)) => {
            if let Some(node_addr) = _sub_matches.get_one::<String>("boot") {
                sync_node(node_addr.to_string())
                    .await
                    .unwrap_or_else(|err| {
                        eprintln!("{:?}", err);
                        runtime::exit(1);
                    });
                if let Some(port) = _sub_matches.get_one::<String>("port") {
                    run_node(port.to_string()).await.unwrap_or_else(|err| {
                        eprintln!("{:?}", err);
                        runtime::exit(1);
                    });
                }
                run_node(String::from("50051")).await.unwrap_or_else(|err| {
                    eprintln!("{:?}", err);
                    runtime::exit(1);
                });
            }
        }
        Some(("createblockchain", _sub_matches)) => {
            create_new_blockchain().unwrap_or_else(|err| {
                eprintln!("{:?}", err);
                runtime::exit(1);
            });
            run_node(String::from("50051")).await.unwrap_or_else(|err| {
                eprintln!("{:?}", err);
                runtime::exit(1);
            });
        }
        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
    }
}
