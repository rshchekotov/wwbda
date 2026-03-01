use ::log::{error, info};
use dotenvy::dotenv;
use libshogi::State;
use poise::serenity_prelude::{self as serenity, ChannelId, GatewayIntents};
use std::env;
use std::error::Error as StdError;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::Mutex;

use crate::command::shogi;

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

async fn on_error(error: poise::FrameworkError<'_, BotContext, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    // init logging
    if let Err(e) = log::init() {
        eprintln!("failed to init logger: {}", e);
    }

    let token = env::var("DISCORD_TOKEN").expect("Expected a Discord Token in the environment.");
    let log_channel = env::var("LOG_CHANNEL")
        .expect("Expected a Log Channel in the environment.")
        .parse()
        .expect("Expected the Log Channel ID to be a proper Discord Snowflake");
    let environment = env::var("ENV").unwrap_or("production".to_string());
    info!("Initial Environment set-up finished.");

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let framework_environment = environment.clone();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: command::COMMANDS.iter().map(|f| f()).collect::<Vec<_>>(),
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("$".into()),
                ..Default::default()
            },
            on_error: |error| Box::pin(on_error(error)),
            event_handler: |ctx, event, framework, data| {
                Box::pin(event::event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                let commands = &framework.options().commands;
                if !commands.is_empty() {
                    poise::builtins::register_globally(ctx, commands).await?;
                }
                let func = Arc::new(shogi::create_callback(ctx.http.clone()));
                Ok(BotContext {
                    log_channel: ChannelId::new(log_channel),
                    environment: framework_environment,
                    state: Arc::new(Mutex::new(State {
                        threads: vec![],
                        message_callback: Some(func),
                    })),
                })
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .expect("Error creating client");

    // TODO: Once database expands, save a pool-handle into the Bot Context and re-use that one
    // When that happens, give libshogi#run_migrations an argument

    // TODO: Discord Listener
    // Join Discord + LibShogi Listener
    let ctrl_c = signal::ctrl_c();
    tokio::select! {
        _ = ctrl_c => {
            info!("Shutting down...");
        }
        result = client.start() => {
            if let Err(why) = result {
                error!("Client error: {:?}", why);
            }
        }
    }
}
