mod block;
mod node;
mod cli;
mod signature;
mod transaction;
mod utils;

use cli::core::run_cli;

#[tokio::main]
async fn main() {
    run_cli().await;
}
