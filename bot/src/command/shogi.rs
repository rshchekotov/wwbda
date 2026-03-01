use crate::{Context, Error};
use poise::serenity_prelude as serenity;

#[poise::command(prefix_command, slash_command, subcommands("user", "track"))]
pub async fn shogi_root(_ctx: Context<'_>) -> Result<(), Error> {
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
    libshogi::add_game(game_id, state);
    Ok(())
}
