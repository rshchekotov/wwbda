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
            info!("Logged in as {} on Discord", data_about_bot.user.tag());
            if data.environment != "develop" {
                let msg = CreateMessage::new().content("Logging Channel Found :white_check_mark:");
                data.log_channel
                    .send_message(&ctx.http, msg)
                    .await
                    .expect("Could not send initial message to logging channel.");
            }
        }
        _ => {}
    }
    Ok(())
}
