use dotenvy::dotenv;
use std::env;

mod log;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // init logging
    if let Err(e) = log::init() {
        eprintln!("failed to init logger: {}", e);
    }

    let game_id = env::var("GAME_ID").unwrap_or_else(|_| "ic3sSYwC".to_string()); // dP8exR8A

    ::log::info!("Starting listener for game {}", game_id);

    if let Err(e) = libshogi::listen(&game_id).await {
        ::log::error!("listener error: {}", e);
    }
}
