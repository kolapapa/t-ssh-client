mod client;
mod error;

use std::future;
use thrussh_keys::key;

pub use client::*;
pub use error::ClientError;

#[derive(Default)]
pub struct Output {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub code: Option<u32>,
}

impl Output {
    pub fn stdout_string(&self) -> String {
        String::from_utf8_lossy(&self.stdout).into()
    }

    pub fn stderr_string(&self) -> String {
        String::from_utf8_lossy(&self.stderr).into()
    }

    pub fn success(&self) -> bool {
        self.code == Some(0)
    }
}

#[derive(Default)]
struct Handler {}

impl thrussh::client::Handler for Handler {
    type Error = ClientError;
    type FutureBool = future::Ready<Result<(Self, bool), Self::Error>>;
    type FutureUnit = future::Ready<Result<(Self, thrussh::client::Session), Self::Error>>;

    fn finished_bool(self, b: bool) -> Self::FutureBool {
        future::ready(Ok((self, b)))
    }

    fn finished(self, session: thrussh::client::Session) -> Self::FutureUnit {
        future::ready(Ok((self, session)))
    }

    #[allow(unused_variables)]
    fn check_server_key(self, server_public_key: &key::PublicKey) -> Self::FutureBool {
        self.finished_bool(true)
    }
}
