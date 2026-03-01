pub mod persistence;
pub mod ws;

use std::{pin::Pin, sync::Arc};

// Shared message types parsed from LiShogi websocket JSON payloads.
use serde::Deserialize;

pub type SocketMessageCallback =
    Box<dyn Fn(SocketMessage) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

#[derive(Default)]
pub struct State {
    pub threads: Vec<tokio::task::JoinHandle<()>>,
    pub message_callback: Option<Arc<SocketMessageCallback>>,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct Clock {
    pub sente: u64,
    pub gote: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MoveData {
    pub usi: String,
    pub sfen: String,
    pub ply: u32,
    pub clock: Option<Clock>,
    pub check: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct GameStatus {
    id: u32,
    name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EndGameData {
    pub winner: String,
    pub status: GameStatus,
}

#[derive(Debug, Deserialize, Clone)]
pub enum MessageData {
    MoveData(MoveData),
    EndGameData(EndGameData),
}

#[derive(Debug, Deserialize, Clone)]
pub struct SocketMessage {
    pub t: String,
    pub v: u32,
    pub d: Option<MessageData>,
}

#[derive(Debug, Deserialize)]
pub struct Watchers {
    pub nb: u32,
}

#[derive(Debug, Deserialize)]
pub struct CrowdData {
    pub sente: bool,
    pub gote: bool,
    pub watchers: Watchers,
}

#[derive(Debug, Deserialize)]
pub struct CrowdMessage {
    pub d: CrowdData,
}

pub use persistence::{add_game, add_player, run_migrations};
pub use ws::listen;
