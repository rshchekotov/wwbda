use ::log::info;
use dotenvy::dotenv;
use poise::serenity_prelude::ChannelId;
use std::env;
use std::error::Error as StdError;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::Mutex;

mod command;
mod event;
mod log;

pub struct BotContext {
    log_channel: ChannelId,
    environment: String,
    state: Arc<Mutex<libshogi::State>>,
}

pub(crate) type Error = Box<dyn StdError + Send + Sync>;
pub(crate) type Context<'a> = poise::Context<'a, BotContext, Error>;

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

    // TODO: Once database expands, save a pool-handle into the Bot Context and re-use that one
    // When that happens, give libshogi#run_migrations an argument
    if libshogi::run_migrations().is_ok() {
        let mut state = libshogi::State {
            threads: vec![],
            message_callback: None,
        };
        libshogi::listen(&mut state).await;
    }

    // TODO: Discord Listener
    // Join Discord + LibShogi Listener
    let ctrl_c = signal::ctrl_c();
    tokio::select! {
        _ = ctrl_c => {
            info!("Shutting down...")
        }
    }

    info!("Environment set-up finished.");
}
