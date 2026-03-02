use std::sync::Arc;

use crate::{Context, Error};
use libshogi::{
    EndGameData, MessageData, MoveData, SocketMessage, SocketMessageCallback,
    persistence::models::DetailedShogiGame,
};
use poise::serenity_prelude::{
    self as serenity, ChannelType, CreateEmbed, CreateMessage, CreateThread, EditThread,
    Mentionable, UserId,
};

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
    if let Some(target_user) = user {
        libshogi::add_player(target_user.id.get() as i64, lishogi_tag.clone());
        ctx.reply(format!(
            "Associated Lishogi ID '{}' with {}",
            lishogi_tag,
            target_user.mention(),
        ))
        .await
        .expect("Third-Person User Creation Response should succeed.");
    } else {
        let target_user = ctx.author();
        libshogi::add_player(target_user.id.get() as i64, lishogi_tag.clone());
        ctx.reply(format!(
            "Associated Lishogi ID '{}' with {}",
            lishogi_tag,
            target_user.mention(),
        ))
        .await
        .expect("User Creation Response should succeed.");
    }
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
    libshogi::add_game(game_id.clone(), &mut state.threads, callback);
    ctx.reply(format!("Now tracking game with ID '{}'.", game_id))
        .await
        .expect("Game Tracking Reponse should succeed.");
    Ok(())
}

#[poise::command(prefix_command, slash_command, owners_only)]
pub async fn debug(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Nothing is being debugged at the moment.")
        .await
        .expect("Debug command should succeed.");
    Ok(())
}

pub fn create_callback(client: Arc<serenity::Http>) -> SocketMessageCallback {
    Box::new(
        move |game_id: &str, game: DetailedShogiGame, msg: SocketMessage| {
            let http = Arc::clone(&client);
            let owned_game_id = game_id.to_string();

            Box::pin(async move {
                let guilds = http
                    .get_guilds(None, None)
                    .await
                    .expect("GuildInfo should be safely retrieved.");
                let guild = guilds
                    .first()
                    .expect("The bot should be present in exactly one guild.");

                let channels = http
                    .get_channels(guild.id)
                    .await
                    .expect("Channels should be safely retrievable.");

                let mut thread = if let Some(thread) =
                    channels.iter().find(|c| c.name == owned_game_id.as_str())
                {
                    thread.clone()
                } else {
                    let channel = channels
                        .iter()
                        .find(|c| c.name.as_str() == "将棋")
                        .expect("There should exist a channel called 将棋.");

                    channel
                        .create_thread(
                            &http,
                            CreateThread::new(owned_game_id.as_str())
                                .kind(ChannelType::PublicThread)
                                .auto_archive_duration(serenity::AutoArchiveDuration::OneWeek)
                                .audit_log_reason(
                                    format!("Game Thread #{}", owned_game_id).as_str(),
                                ),
                        )
                        .await
                        .expect("Should be able to create a game thread.")
                };

                if let Some(data) = msg.d {
                    match data {
                        MessageData::MoveData(MoveData {
                            sfen, clock, check, ..
                        }) => {
                            if let Some(game_move) = &game.latest_move {
                                // Determine whose turn it is now (opposite of who just moved)
                                // Odd turn = Sente moved, so ping Gote. Even turn = Gote moved, so ping Sente
                                let is_sente_turn = game_move.turn % 2 == 0;

                                let mut embed = CreateEmbed::new()
                                    .title(format!("Game #{}", game.game.id))
                                    .description(format!("Turn #{}\n{}", game_move.turn, sfen))
                                    .timestamp(serenity::Timestamp::now());

                                if let Some(check_val) = check {
                                    if check_val {
                                        embed = embed.field("Status", "王手", false);
                                    }
                                }

                                if let Some(clock_val) = clock {
                                    embed = embed.field(
                                        "Clock",
                                        format!(
                                            "☗ Sente: {}s | ☖ Gote: {}s",
                                            clock_val.sente, clock_val.gote
                                        ),
                                        false,
                                    );
                                }

                                let mut message = CreateMessage::new().embed(embed);

                                // Ping the player whose turn it is
                                if is_sente_turn {
                                    if let Some(sente_player) = &game.sente {
                                        message = message.content(
                                            UserId::new(sente_player.id as u64)
                                                .mention()
                                                .to_string(),
                                        );
                                    }
                                } else {
                                    if let Some(gote_player) = &game.gote {
                                        message = message.content(
                                            UserId::new(gote_player.id as u64)
                                                .mention()
                                                .to_string(),
                                        );
                                    }
                                }

                                thread
                                    .send_message(&http, message)
                                    .await
                                    .expect("Should be able to send message to thread.");
                            }
                        }
                        MessageData::EndGameData(EndGameData { winner, status }) => {
                            let embed = CreateEmbed::new()
                                .title(format!("Game #{}", game.game.id))
                                .description(format!(
                                    "Game Ended!\nWinner: {}\nWin Condition: {}",
                                    winner, status.name
                                ))
                                .timestamp(serenity::Timestamp::now())
                                .color(0x00FF00);

                            thread
                                .send_message(&http, CreateMessage::new().embed(embed))
                                .await
                                .expect("Should be able to send message to thread.");

                            // Archive the thread
                            thread
                                .edit_thread(&http, EditThread::new().archived(true))
                                .await
                                .expect("Should be able to archive the thread.");
                        }
                    }
                }
            })
        },
    )
}
