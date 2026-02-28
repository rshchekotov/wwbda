pub mod persistence;
pub mod ws;

// Shared message types parsed from LiShogi websocket JSON payloads.
use serde::Deserialize;

#[derive(Debug)]
pub struct State {
    pub threads: Vec<tokio::task::JoinHandle<()>>,
    pub message_callback: Option<fn(SocketMessage)>,
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

pub use ws::listen;
