use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error(transparent)]
    IOError(#[from] io::Error),
    #[error("connect server failed: {0}")]
    ClientFailed(#[from] thrussh::Error),
    #[error("connect server timeout.")]
    Timeout,
    #[error("auth failed: {0}")]
    AuthFailed(String),
}
