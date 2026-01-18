mod db;
use db::init_db;

#[tokio::main]
async fn main() {
    init_db().await;
}

