use chrono::Local;
use fern::Dispatch;
use fern::colors::{Color, ColoredLevelConfig};
use log::LevelFilter;
use std::fs;

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    let colors = ColoredLevelConfig::new()
        .debug(Color::Blue)
        .info(Color::Green)
        .warn(Color::Yellow)
        .error(Color::Red);

    // file name with timestamp
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    fs::create_dir_all("logs")?;
    let log_file_name = format!("logs/{}.log", timestamp);

    // console dispatcher, show debug for libshogi target only
    let console_dispatch = Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                colors.color(record.level()),
                message
            ))
        })
        .level(LevelFilter::Debug)
        .filter(|metadata| {
            metadata.target().contains("libshogi") || metadata.target().contains("bot")
        })
        .chain(std::io::stdout());

    // file dispatcher: include target and timestamp
    let file_dispatch = Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(LevelFilter::Info)
        .level_for("libshogi", LevelFilter::Debug)
        .chain(fern::log_file(log_file_name)?);

    // combine and apply
    Dispatch::new()
        .chain(console_dispatch)
        .chain(file_dispatch)
        .apply()?;

    Ok(())
}
