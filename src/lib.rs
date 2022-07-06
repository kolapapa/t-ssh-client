use std::future;
use std::io::Write;
use std::sync::Arc;

use thrussh_keys::key;

#[derive(Default)]
pub struct Output {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    code: Option<u32>,
}

impl Output {
    pub fn stdout(&self) -> String {
        String::from_utf8_lossy(&self.stdout).into()
    }

    pub fn stderr(&self) -> String {
        String::from_utf8_lossy(&self.stderr).into()
    }

    pub fn success(&self) -> bool {
        self.code == Some(0)
    }
}

#[derive(Default)]
struct Client {}

impl thrussh::client::Handler for Client {
    type Error = thrussh::Error;
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

pub struct Session {
    inner: thrussh::client::Handle<Client>,
}

impl Session {
    pub async fn connect(addr: &str) -> Result<Session, thrussh::Error> {
        let config = Arc::new(thrussh::client::Config::default());
        let client = Client::default();
        let ssh = thrussh::client::connect(config, addr, client).await?;
        Ok(Self { inner: ssh })
    }

    pub async fn auth_with_password(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<(), thrussh::Error> {
        let success = self.inner.authenticate_password(username, password).await?;
        if !success {
            return Err(thrussh::Error::NotAuthenticated);
        }
        Ok(())
    }

    #[allow(unused_variables)]
    pub async fn execute(&mut self, command: &str) -> Result<Output, thrussh::Error> {
        let mut channel = self.inner.channel_open_session().await?;
        channel.exec(true, command).await?;
        let mut res = Output::default();
        while let Some(msg) = channel.wait().await {
            match msg {
                thrussh::ChannelMsg::Data { ref data } => {
                    res.stdout.write_all(&data)?;
                }
                thrussh::ChannelMsg::ExitStatus { exit_status } => {
                    res.code = Some(exit_status);
                }
                thrussh::ChannelMsg::ExtendedData { ref data, ext } => {
                    res.stderr.write_all(&data)?;
                }

                _ => {}
            }
        }
        Ok(res)
    }

    pub async fn close(&mut self) -> Result<(), thrussh::Error> {
        self.inner
            .disconnect(thrussh::Disconnect::ByApplication, "", "English")
            .await?;
        Ok(())
    }
}
