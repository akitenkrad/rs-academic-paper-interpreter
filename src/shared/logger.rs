use crate::shared::errors::AppResult;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logger<T: AsRef<str>>(log_level: T) -> AppResult<()> {
    let log_level = log_level.as_ref();
    assert!(
        log_level == "error"
            || log_level == "warn"
            || log_level == "info"
            || log_level == "debug"
            || log_level == "trace"
            || log_level == "off",
        "Invalid log level: {}",
        log_level
    );

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| log_level.into());
    let subscriber = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_target(false);
    tracing_subscriber::registry()
        .with(subscriber)
        .with(env_filter)
        .try_init()?;

    Ok(())
}
