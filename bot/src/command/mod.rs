use poise::Command;

use crate::{BotContext, Error};

pub mod shogi;

pub static COMMANDS: &[fn() -> Command<BotContext, Error>] = &[shogi::shogi];
