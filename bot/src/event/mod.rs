use log::info;
use poise::serenity_prelude::{self as serenity, CreateMessage};
use serenity::FullEvent;

use crate::{BotContext, Error};

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &FullEvent,
    _framework: poise::FrameworkContext<'_, BotContext, Error>,
    data: &BotContext,
) -> Result<(), Error> {
    // I will *likely* add new features which will require this match.
    // Hence it's fine to leave this here for now...
    #[allow(clippy::single_match)]
    match event {
        FullEvent::Ready { data_about_bot, .. } => {
            let mut rebooted = false;
            if tokio::fs::try_exists("logs/.reboot")
                .await
                .expect("Should be able to check whether reboot flag exists")
            {
                tokio::fs::remove_file("logs/.reboot")
                    .await
                    .expect("Should be able to clear reboot flag");

                rebooted = true;
                info!("Reboot Successful.");
            }

            info!("Logged in as {} on Discord", data_about_bot.user.tag());
            if data.environment != "develop" {
                let msg = CreateMessage::new().content(if rebooted {
                    "> *initialized*"
                } else {
                    "> *rebooted*"
                });
                data.log_channel
                    .send_message(&ctx.http, msg)
                    .await
                    .expect("Could not send initial message to logging channel.");
            }

            if libshogi::run_migrations().is_ok() {
                let shared_state = data.state.clone();
                tokio::spawn(async move {
                    let mut state = shared_state.lock().await;
                    let callback = state.message_callback.clone();
                    info!("Starting LiShogi listener...");
                    libshogi::listen(&mut state.threads, callback).await;
                    info!("LiShogi listener has processed all running games...");
                });
            }
        }
        _ => {}
    }
    Ok(())
}
