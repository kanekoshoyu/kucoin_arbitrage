use chrono::prelude::Local;
use colored::Colorize;
use eyre::Result;
use fern;

// TODO find a way to obtain the running binary name for
// - terminal log whitelisting
// - console log filename

fn dispatch_console() -> fern::Dispatch {
    fern::Dispatch::new()
        .level(tracing::LevelFilter::Info)
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                match record.level() {
                    tracing::Level::Error => "ERROR".red(),
                    tracing::Level::Warn => "WARN".yellow(),
                    tracing::Level::Info => "INFO".green(),
                    tracing::Level::Debug => "DEBUG".cyan(),
                    tracing::Level::Trace => "TRACE".blue(),
                },
                record.target(),
                message
            ))
        })
        .chain(std::io::stdout())
}

fn dispatch_file(filename: &str) -> fern::Dispatch {
    fern::Dispatch::new()
        .level(tracing::LevelFilter::Info)
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                match record.level() {
                    tracing::Level::Error => "ERROR",
                    tracing::Level::Warn => "WARN",
                    tracing::Level::Info => "INFO",
                    tracing::Level::Debug => "DEBUG",
                    tracing::Level::Trace => "TRACE",
                },
                record.target(),
                message
            ))
        })
        .chain(fern::log_file(filename).expect("Failed to create log file"))
}

pub fn log_init() -> Result<()> {
    // setup time loggers
    let fmt = "%Y_%m_%d_%H_%M_%S";
    let formatted_date = Local::now().format(fmt).to_string();
    // Output file
    let dir = "log";
    let filename = format!("{dir}/{formatted_date}.log");
    if std::fs::metadata(dir).is_err() {
        std::fs::create_dir(dir)?;
    }
    fern::Dispatch::new()
        .chain(dispatch_file(&filename))
        .chain(dispatch_console())
        .apply()?;
    Ok(())
}
