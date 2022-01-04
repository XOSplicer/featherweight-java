use thiserror::Error;

#[derive(Debug, Error)]
pub enum TypingError {
    #[error(transparent)]
    Other(#[from] anyhow::Error)
}

#[derive(Debug, Error)]
pub enum EvalError {
    #[error(transparent)]
    Other(#[from] anyhow::Error)
}