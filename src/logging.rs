use crate::types::LogError;
use env_logger;
use std::fs::{File, OpenOptions};

pub fn initialize_logger() -> Result<(), LogError> {
    env_logger::init();
    let log_dir = "./logs";

    match std::fs::metadata(&log_dir) {
        Ok(meta) if meta.is_dir() => println!("log dir exists already"),
        _ => std::fs::create_dir(&log_dir).expect("failed to create dir"),
    };

    let mut log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(format!("{}/grammarbot.log", &log_dir))
        .expect("coulnd't open log file");

    let logger = |record: &log::Record| {
        writeln!(
            &mut log_file,
            "[{}] - {} - {}",
            record.level(),
            chrono::Local::now(),
            record.args()
        )
        .expect("Filed to write to log file");

        // print log message to console as well
        println!("[{}] - {} - {}", chrono::Local::now(), record.args());
    };

    log::set_boxed_logger(Box::new(logger))
        .map(|()| log::set_max_level(log::LevelFilter::Info))
        .expect("failed to setup logger");

    Ok(())
}
