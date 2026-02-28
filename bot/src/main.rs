use ::log::info;
use dotenvy::dotenv;
use poise::serenity_prelude::ChannelId;
use std::env;

mod log;

pub struct BotContext {
    log_channel: ChannelId,
    environment: String,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    // init logging
    if let Err(e) = log::init() {
        eprintln!("failed to init logger: {}", e);
    }

    let token = env::var("DISCORD_TOKEN").expect("Expected a Discord Token in the environment.");
    let log_channel = env::var("LOG_CHANNEL").expect("Expected a Log Channel in the environment.");
    let environment = env::var("ENV").unwrap_or("production".to_string());

    let mut state = libshogi::State {
        threads: vec![],
        message_callback: None,
    };
    libshogi::listen(&mut state).await;

    // TODO: Discord Listener
    // Join Discord + LibShogi Listener

    info!("Environment set-up finished.");
}
