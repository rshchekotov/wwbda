use std::sync::Arc;

use crate::{Context, Error};
use libshogi::{SocketMessage, SocketMessageCallback};
use poise::serenity_prelude as serenity;

#[poise::command(
    rename = "shogi",
    prefix_command,
    slash_command,
    subcommands("user", "track")
)]
pub async fn shogi(_ctx: Context<'_>) -> Result<(), Error> {
    // TODO: "Database Dump":
    //   - how many/which registered users
    //   - how many games
    //     - currently tracking
    //     - already finished
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn user(
    ctx: Context<'_>,
    #[description = "Lishogi Username"] lishogi_tag: String,
    #[description = "Discord User (default: you)"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let id = if let Some(indicated_user) = user {
        indicated_user.id
    } else {
        ctx.author().id
    };
    libshogi::add_player(id.get() as i64, lishogi_tag);
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn track(
    ctx: Context<'_>,
    #[description = "Lishogi Game ID (8 characters, i.e. dP8exR8A)"]
    #[min_length = 8]
    #[max_length = 8]
    game_id: String,
) -> Result<(), Error> {
    let state = &mut ctx.data().state.lock().await;
    let callback = state.message_callback.as_ref().map(|arc| arc.clone());
    libshogi::add_game(game_id, &mut state.threads, callback);
    Ok(())
}

pub fn create_callback(client: Arc<serenity::Http>) -> SocketMessageCallback {
    Box::new(move |_msg: SocketMessage| {
        let http = Arc::clone(&client);
        Box::pin(async move {
            let guilds = http
                .get_guilds(None, None)
                .await
                .expect("GuildInfo should be safely retrieved.");
            for guild in guilds {
                println!("Guild: {:?}", guild);
            }
            // let channels = client.get_channels("");
            // client.send_message().await;
        })
    })
}
