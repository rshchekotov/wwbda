pub mod ws;

// Shared message types parsed from LiShogi websocket JSON payloads.
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Clock {
    pub sente: u64,
    pub gote: u64,
}

#[derive(Debug, Deserialize)]
pub struct MoveData {
    pub usi: String,
    pub sfen: String,
    pub ply: u32,
    pub clock: Option<Clock>,
    pub check: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct GameStatus {
    id: u32,
    name: String,
}

#[derive(Debug, Deserialize)]
pub struct EndGameData {
    pub winner: String,
    pub status: GameStatus,
}

#[derive(Debug, Deserialize)]
pub enum MessageData {
    MoveData(MoveData),
}

#[derive(Debug, Deserialize)]
pub struct SocketMessage {
    pub t: String,
    pub v: u32,
    pub d: MessageData,
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

pub use ws::{collect_pings, listen};
