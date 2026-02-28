use diesel::prelude::*;
use futures::stream::SplitSink;
use futures::{SinkExt, StreamExt};
use log::{debug, info};
use rand::{RngExt, distr::Alphanumeric};
use std::sync::Arc;
use std::{error::Error, time::Duration};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, MutexGuard};
use tokio::time;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};

use crate::persistence::{establish_connection, sqlite_pool_handler};
use crate::{CrowdMessage, MessageData, SocketMessage, State};

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

pub async fn listen(state: &mut State) {
    use crate::persistence::schema::{
        shogi_game::dsl::*,
        shogi_game_move::dsl::{id as sgm_id, *},
    };

    let pool = establish_connection();
    let connection =
        &mut sqlite_pool_handler(&pool).expect("Pooled Connection should be established.");

    let game_ids = shogi_game_move
        .left_outer_join(shogi_game)
        .filter(winner.is_null())
        .select(sgm_id)
        .distinct()
        .load::<String>(connection)
        .expect("Failed to fetch games.");

    for game_id in game_ids {
        state.threads.push(tokio::spawn({
            let callback = state.message_callback;
            async move {
                let _ = listen_to_game(game_id.as_str(), callback).await;
            }
        }));
    }
}

/// Long-running game listener.
pub async fn listen_to_game(
    game_id: &str,
    callback: Option<fn(SocketMessage)>,
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
                    info!("[{}]: {:?}", game_id, ws_msg);
                    if let Some(func) = callback {
                        func(ws_msg.clone());
                    }

                    if let Some(data) = ws_msg.d
                        && let MessageData::EndGameData(_) = data
                    {
                        info!("[{}] Game Ended", game_id);
                        break;
                    }
                    continue;
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
    async fn test_collect_pings() {
        // use the same hard-coded game id as the python ref
        let game_id = "dP8exR8A";
        let pings = collect_pings(game_id, 1, 30)
            .await
            .expect("collect pings failed");
        assert!(pings >= 1, "expected at least one server ping");
    }
}
