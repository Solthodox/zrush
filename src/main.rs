mod block;
mod cli;
mod node;
mod signature;
mod transaction;
mod utils;
mod wallet;
mod p2p;
use cli::core::run_cli;


#[tokio::main]
async fn main() {
    run_cli().await;
}

// ███████╗██████╗ ██╗   ██╗███████╗██╗  ██╗
// ╚══███╔╝██╔══██╗██║   ██║██╔════╝██║  ██║
//   ███╔╝ ██████╔╝██║   ██║███████╗███████║
//  ███╔╝  ██╔══██╗██║   ██║╚════██║██╔══██║
// ███████╗██║  ██║╚██████╔╝███████║██║  ██║
// ╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝╚═╝  ╚═╝
