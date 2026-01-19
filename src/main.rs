mod cli;
mod config;
mod date;
mod db;
mod transact;
use db::init_db;

#[tokio::main]
async fn main() {
    init_db().await;
    cli::run().await;
}
