use poise::Command;

use crate::{BotContext, Error};

pub mod changelog;
pub mod reboot;
pub mod shogi;

pub static COMMANDS: &[fn() -> Command<BotContext, Error>] =
    &[shogi::shogi, shogi::debug, changelog::changelog];
