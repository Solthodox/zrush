mod block;
mod cli;
mod node;
mod signature;
mod transaction;
mod utils;
mod wallet;
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
