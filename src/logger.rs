use chrono::prelude::Local;

pub fn log_init() {
    // setup time loggers
    let fmt = "%Y_%m_%d_%H_%M_%S";
    let now = Local::now();
    let formatted_date = now.format(fmt).to_string();

    // Output file
    let dir = "log";
    let filename = String::from(dir) + "/" + &formatted_date + ".log";
    let _ = std::fs::create_dir(dir);

    // Terminal logger
    let console_logger = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                Local::now().format("[%Y-%m-%d %H:%M:%S]"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout());

    // File logger
    let file_logger = fern::Dispatch::new()
        .format(|out, message, _| out.finish(format_args!("{}", message)))
        .level(log::LevelFilter::Info)
        .chain(fern::log_file(filename).unwrap());

    // TODO add color to the terminal upon the release of fern 0.7
    console_logger
        .chain(file_logger)
        .apply()
        .expect("failed to initialize logger");
}
