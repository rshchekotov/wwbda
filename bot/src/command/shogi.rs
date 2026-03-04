use std::sync::Arc;

use crate::{Context, Error, util::time::format_duration};
use libshogi::{
    EndGameData, MessageData, MoveData, SocketMessage, SocketMessageCallback,
    persistence::models::{DetailedShogiGame, ShogiGameMove},
};
use log::warn;
use poise::serenity_prelude::{
    self as serenity, ChannelType, CreateEmbed, CreateMessage, CreateThread, EditThread,
    GuildChannel, Mentionable, UserId,
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
        let success = libshogi::add_player(target_user.id.get() as i64, lishogi_tag.clone());
        if success {
            ctx.reply(format!(
                "Associated Lishogi ID '{}' with {}",
                lishogi_tag,
                target_user.mention(),
            ))
            .await
            .expect("Third-Person User Creation Response should succeed.");
        } else {
            ctx.reply(format!(
                "Couldn't associate Lishogi ID '{}' with {} (maybe already associated w/ something else).",
                lishogi_tag,
                target_user.mention(),
            ))
            .await
            .expect("Negative Third-Person User Creation Response should succeed.");
        }
    } else {
        let target_user = ctx.author();
        let success = libshogi::add_player(target_user.id.get() as i64, lishogi_tag.clone());
        if success {
            ctx.reply(format!(
                "Associated Lishogi ID '{}' with {}",
                lishogi_tag,
                target_user.mention(),
            ))
            .await
            .expect("Positive User Creation Response should succeed.");
        } else {
            ctx.reply(format!(
                "Couldn't associate Lishogi ID '{}' with {} (maybe already associated w/ something else).",
                lishogi_tag,
                target_user.mention(),
            ))
            .await
            .expect("Negative User Creation Response should succeed.");
        }
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
    #[description = "Lishogi Tag of the Sente Player"] sente_lishogi: Option<String>,
    #[description = "Lishogi Tag of the Gote Player"] gote_lishogi: Option<String>,
) -> Result<(), Error> {
    let state = &mut ctx.data().state.lock().await;
    let callback = state.message_callback.as_ref().map(|arc| arc.clone());
    let added_game = libshogi::add_game(
        game_id.clone(),
        sente_lishogi,
        gote_lishogi,
        &mut state.threads,
        callback,
    );
    if added_game {
        ctx.reply(format!("Now tracking game with ID '{}'.", game_id))
            .await
            .expect("Positive Game Tracking Reponse should succeed.");
    } else {
        ctx.reply(format!(
            "Failed to add game '{}'.\nMaybe it already exists?",
            game_id
        ))
        .await
        .expect("Negative Game Tracking Reponse should succeed.");
    }
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
        move |game_id: &str,
              game: DetailedShogiGame,
              last_move: Option<ShogiGameMove>,
              msg: SocketMessage| {
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

                let shogi_channel = channels
                    .iter()
                    .find(|c| c.name.as_str() == "将棋")
                    .expect("There should exist a channel called 将棋.");

                let result = http
                    .get_guild_active_threads(guild.id)
                    .await
                    .expect("Should be able to fetch active guild threads.");
                let threads: Vec<&GuildChannel> = result
                    .threads
                    .iter()
                    .filter(|&c| c.parent_id == Some(shogi_channel.id))
                    .collect();

                let mut new_thread = false;
                let mut thread =
                    if let Some(&thread) = threads.iter().find(|&&c| c.name == owned_game_id) {
                        thread.clone()
                    } else {
                        new_thread = true;
                        shogi_channel
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
                                if new_thread {
                                    let sente = if let Some(sente_player) = &game.sente {
                                        UserId::new(sente_player.id as u64).mention().to_string()
                                    } else {
                                        "<unknown>".to_string()
                                    };

                                    let gote = if let Some(gote_player) = &game.gote {
                                        UserId::new(gote_player.id as u64).mention().to_string()
                                    } else {
                                        "<unknown>".to_string()
                                    };

                                    thread
                                        .send_message(
                                            &http,
                                            CreateMessage::new().content(format!(
                                                "Tracking Game: {} vs {}",
                                                sente, gote
                                            )),
                                        )
                                        .await
                                        .expect(
                                            "Should be able to send first Game Tracking Message.",
                                        );
                                }

                                // Determine whose turn it is now (opposite of who just moved)
                                // Odd turn = Sente moved, so ping Gote. Even turn = Gote moved, so ping Sente
                                let is_sente_turn = game_move.turn % 2 == 0;

                                let mut embed = CreateEmbed::new()
                                    .title(format!("Game #{}", game.game.id))
                                    .url(format!("https://lishogi.org/{}", game.game.id))
                                    .description(format!("Turn #{}\n{}", game_move.turn, sfen))
                                    .timestamp(serenity::Timestamp::now());

                                if let Some(check_val) = check
                                    && check_val
                                {
                                    embed = embed.field("Status", "王手", false);
                                }

                                if let Some(clock_val) = clock
                                    && let Some(prev_move) = last_move
                                {
                                    let current_move = game.latest_move
                                        .expect("The current move must exist for there to be a GameMove event.");

                                    let (player_clock, emoji) = if is_sente_turn {
                                        (clock_val.gote, "☖ ")
                                    } else {
                                        (clock_val.sente, "☗ ")
                                    };

                                    let formatted_clock_time = format_duration(player_clock);
                                    let mut delta = current_move.ts.and_utc().timestamp()
                                        - prev_move.ts.and_utc().timestamp();
                                    if delta < 0 {
                                        delta = -delta;
                                        warn!(
                                            "Negative Time Delta found in {} ({} -> {})",
                                            owned_game_id, prev_move.turn, current_move.turn
                                        );
                                    }
                                    let formatted_delta_time = format_duration(delta as u64);

                                    embed = embed.field(
                                        "Time",
                                        format!(
                                            "{}{} / {} ({:.2}%)",
                                            emoji,
                                            formatted_delta_time,
                                            formatted_clock_time,
                                            (delta as f32 / player_clock as f32) * 100f32
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
                                } else if let Some(gote_player) = &game.gote {
                                    message = message.content(
                                        UserId::new(gote_player.id as u64).mention().to_string(),
                                    );
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
