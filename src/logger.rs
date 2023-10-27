use chrono::prelude::Local;
use colored::Colorize;
use fern;

// TODO find a way to obtain the running binary name for
// - terminal log whitelisting
// - console log filename

fn dispatch_console() -> fern::Dispatch {
    fern::Dispatch::new()
        .level(log::LevelFilter::Info)
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                match record.level() {
                    log::Level::Error => "ERROR".red(),
                    log::Level::Warn => "WARN".yellow(),
                    log::Level::Info => "INFO".green(),
                    log::Level::Debug => "DEBUG".cyan(),
                    log::Level::Trace => "TRACE".blue(),
                },
                record.target(),
                message
            ))
        })
        .chain(std::io::stdout())
}

fn dispatch_file(filename: &str) -> fern::Dispatch {
    fern::Dispatch::new()
        .level(log::LevelFilter::Info)
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                match record.level() {
                    log::Level::Error => "ERROR",
                    log::Level::Warn => "WARN",
                    log::Level::Info => "INFO",
                    log::Level::Debug => "DEBUG",
                    log::Level::Trace => "TRACE",
                },
                record.target(),
                message
            ))
        })
        .chain(fern::log_file(filename).expect("Failed to create log file"))
}

pub fn log_init() -> Result<(), failure::Error> {
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
