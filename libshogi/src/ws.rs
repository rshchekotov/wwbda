use diesel::prelude::*;
use futures::stream::SplitSink;
use futures::{SinkExt, StreamExt};
use log::{debug, info};
use rand::{RngExt, distr::Alphanumeric};
use std::sync::Arc;
use std::{error::Error, time::Duration};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, MutexGuard};
use tokio::task::JoinHandle;
use tokio::time;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};

use crate::persistence::{
    add_move, end_game, establish_connection, get_game_details, get_last_move, sqlite_pool_handler,
};
use crate::{CrowdMessage, MessageData, SocketMessage, SocketMessageCallback};

type StreamMessageMutex<'a> =
    MutexGuard<'a, SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>;

pub fn gen_sri(n: usize) -> String {
    rand::rng()
        .sample_iter(Alphanumeric)
        .take(n)
        .map(char::from)
        .collect()
}

async fn connect_url(
    game_id: &str,
) -> Result<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    Box<dyn Error>,
> {
    let sri = gen_sri(12);
    let url = format!(
        "wss://socket1.lishogi.org/watch/{}/sente/v6?sri={}",
        game_id, sri
    );
    // Let the websocket client construct its own handshake headers (sec-websocket-key etc.).
    // Passing a full Request previously caused the server to reject the handshake.
    let (ws_stream, _resp) = connect_async(url).await?;
    Ok(ws_stream)
}

/// Connect and collect `max_pings` server ping messages (server sends plain `0`).
/// Returns number of pings observed (may be less on timeout).
pub async fn collect_pings(
    game_id: &str,
    max_pings: usize,
    timeout_secs: u64,
) -> Result<usize, Box<dyn Error>> {
    let ws = connect_url(game_id).await?;

    let (write, mut read) = ws.split();
    let write = Arc::new(Mutex::new(write));

    // keepalive task: send 'null' every 3s and occasionally a version check
    let write_clone = write.clone();
    let ka = tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(3));
        let mut version_counter: i32 = 18; // follow python ref
        loop {
            interval.tick().await;
            let mut w: StreamMessageMutex = write_clone.lock().await;
            if version_counter == 19 {
                let _ = w
                    .send(Message::Text(r#"{ "t": "version_check" }"#.into()))
                    .await;
                version_counter = -1;
            }
            version_counter += 1;
            // send literal `null` as text like the python ref
            let _ = w.send(Message::Text("null".into())).await;
        }
    });

    let mut ping_count = 0usize;

    let timeout = Duration::from_secs(timeout_secs);
    let start = time::Instant::now();

    while let Some(msg) = read.next().await {
        let msg = msg?;
        if let Message::Text(t) = msg
            && t == "0"
        {
            ping_count += 1;
            if ping_count >= max_pings {
                break;
            }
        }

        if start.elapsed() > timeout {
            break;
        }
    }

    // cancel keepalive task
    ka.abort();
    Ok(ping_count)
}

pub async fn listen(
    threads: &mut Vec<JoinHandle<()>>,
    callback: Option<Arc<SocketMessageCallback>>,
) {
    use crate::persistence::schema::shogi_game::dsl::*;

    let pool = establish_connection();
    let connection =
        &mut sqlite_pool_handler(&pool).expect("Pooled Connection should be established.");

    let game_ids = shogi_game
        .filter(win_condition.is_null())
        .select(id)
        .load::<String>(connection)
        .expect("Failed to fetch games.");

    for game_id in game_ids {
        threads.push(tokio::spawn({
            let local_callback = callback.clone();
            async move {
                let _ = listen_to_game(game_id.as_str(), local_callback).await;
            }
        }));
    }
}

/// Long-running game listener.
pub async fn listen_to_game(
    game_id: &str,
    callback: Option<Arc<SocketMessageCallback>>,
) -> Result<(), Box<dyn Error>> {
    let ws = connect_url(game_id).await?;

    let (write, mut read) = ws.split();
    let write = Arc::new(Mutex::new(write));

    // keepalive
    let write_clone = write.clone();
    let ka = tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(3));
        let mut version_counter: i32 = 18;
        loop {
            interval.tick().await;
            let mut w: StreamMessageMutex = write_clone.lock().await;
            if version_counter == 19 {
                let _ = w
                    .send(Message::Text(r#"{ "t": "version_check" }"#.into()))
                    .await;
                version_counter = -1;
            }
            version_counter += 1;
            let _ = w.send(Message::Text("null".into())).await;
        }
    });

    // track pings silently (don't print)
    let mut _ping_count = 0usize;

    while let Some(msg) = read.next().await {
        let msg = msg?;
        match msg {
            Message::Text(t) => {
                if t == "0" {
                    // heartbeat ping from server -> increment silently
                    _ping_count += 1;
                    continue;
                }

                if t.starts_with('{') && t.contains("versionCheck") {
                    // ignore
                    continue;
                }

                if let Ok(ws_msg) = serde_json::from_str::<SocketMessage>(&t) {
                    let mut should_end_game = false;

                    debug!("[{}]: {:?}", game_id, ws_msg);
                    let last_turn = get_last_move(game_id).await;

                    let mut update = true;
                    if let Some(data) = ws_msg.clone().d {
                        match data {
                            MessageData::AnnouncementData(d) => {
                                info!("[{}] LiShogi Announcement: {}", game_id, d.msg);
                            }
                            MessageData::MoveData(d) => {
                                if !(add_move(game_id, d).await) {
                                    update = false;
                                }
                            }
                            MessageData::EndGameData(d) => {
                                end_game(game_id, d).await;
                                info!("[{}] Game Ended", game_id);
                                should_end_game = true;
                            }
                        }
                    }

                    if update && let Some(func) = callback.clone() {
                        let game = get_game_details(game_id).await.expect(
                            "There should be an existing Shogi Game at this point in the program.",
                        );

                        func(game_id, game, last_turn, ws_msg).await;
                    }

                    if should_end_game {
                        break;
                    } else {
                        continue;
                    }
                }

                // Try parse crowd message
                if let Ok(crowd_msg) = serde_json::from_str::<CrowdMessage>(&t) {
                    info!("[{}]: {:?}", game_id, crowd_msg.d);
                    continue;
                }

                // fallback: debug log the raw message
                debug!("[{}]: raw: {}", game_id, t);
            }
            Message::Binary(_) => {}
            Message::Ping(_) | Message::Pong(_) => {}
            Message::Close(_) => break,
            _ => {}
        }
    }

    ka.abort();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_move_deserialization() {
        let data = r#"{"t":"usi","v":1,"d":{"usi":"7g7f","sfen":"lnsgkgsnl/1r5b1/ppppppppp/9/9/2P6/PP1PPPPPP/1B5R1/LNSGKGSNL w -","ply":1,"clock":{"sente":1209600,"gote":1209600}}}"#;
        let deserialized = serde_json::from_str::<SocketMessage>(data);
        if deserialized.is_err() {
            println!("{:?}", &deserialized);
        }
        assert!(deserialized.is_ok());
    }
}
