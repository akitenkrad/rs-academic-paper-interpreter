use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    InternalAppError(String),

    // from anyhow
    #[error("Error: {0}")]
    AnyhowError(#[from] anyhow::Error),

    // from tracing
    #[error("Tracing Error: {0}")]
    TracingTryInitError(#[from] tracing_subscriber::util::TryInitError),
}

pub type AppResult<T> = Result<T, AppError>;
