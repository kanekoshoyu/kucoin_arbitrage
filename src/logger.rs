use core::fmt::Result as FmtResult;
use eyre::*;
use serde::*;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use tracing::level_filters::LevelFilter;
use tracing::{Event, Level, Subscriber};
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_subscriber::fmt::{format, FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{EnvFilter, Layer};

#[derive(Debug, Clone, Deserialize)]
pub struct LogFileConfig {
    pub directory: String,
    pub file_name_prefix: String,
    pub log_level: Level,
}

pub fn env_filter_level(log_level: LogLevel) -> EnvFilter {
    EnvFilter::from_default_env().add_directive(Level::from(log_level).into())
}

// custom formatter for file log
struct MyFormatter;
impl<S, N> FormatEvent<S, N> for MyFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: format::Writer<'_>,
        event: &Event<'_>,
    ) -> FmtResult {
        // Format values from the event's's metadata:
        let metadata = event.metadata();
        let level = *metadata.level();
        let datetime = chrono::Utc::now().timestamp() as u64;
        let target = metadata.target();
        let thread = std::thread::current();
        let thread = thread.name().unwrap_or_else(|| "unnamed");
        let line = metadata.line().unwrap_or_default();
        write!(
            writer,
            "\n[{datetime}][{level}][{thread}][{target}][{line}]"
        )
        .unwrap();
        ctx.format_fields(writer, event)?;
        Ok(())
    }
}
/// daily rolling log file
pub fn non_blocking_make_writer_file(
    directory: impl AsRef<Path>,
    file_name_prefix: impl AsRef<Path>,
) -> (NonBlocking, WorkerGuard) {
    let file_appender = tracing_appender::rolling::daily(directory, file_name_prefix);
    tracing_appender::non_blocking(file_appender)
}

pub fn setup_composite_logs(
    log_level: Level,
    log_file_config: &LogFileConfig,
) -> Result<WorkerGuard> {
    use tracing_subscriber::fmt;
    use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
    use tracing_subscriber::Registry;

    let (nb, wg) = non_blocking_make_writer_file(
        &log_file_config.directory,
        &log_file_config.file_name_prefix,
    );
    // file layer with custom formatter
    let file_layer = fmt::layer()
        .with_writer(nb)
        .event_format(MyFormatter)
        .with_ansi(false)
        .with_filter(env_filter_level(log_file_config.log_level));
    // terminal layer
    let stdout_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_thread_names(true)
        .with_line_number(true)
        .without_time()
        .with_filter(env_filter_level(log_level));

    // Combine layers into a single subscriber and set global default
    let subscriber = Registry::default().with(stdout_layer).with(file_layer);
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default subscriber");

    Ok(wg)
}
