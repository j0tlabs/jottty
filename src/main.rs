mod cli;
mod db;
mod date;
mod config;
use db::init_db;

#[tokio::main]
async fn main() {
    init_db().await;
    cli::run().await;
}
