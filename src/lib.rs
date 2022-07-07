mod error;

use std::future;
use std::io::Write;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use std::time::Duration;

use error::ClientError;
use thrussh_keys::key;
use tokio::time;

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

pub enum AuthMethod {
    Password(String),
}

pub struct ClientBuilder {
    username: String,
    auth: Option<AuthMethod>,
    connect_timeout: Duration,
}

impl ClientBuilder {
    pub fn new<S: ToString>(username: S) -> Self {
        Self {
            username: username.to_string(),
            auth: None,
            connect_timeout: Duration::from_secs(10),
        }
    }

    pub fn auth(&mut self, auth: AuthMethod) -> &mut Self {
        self.auth = Some(auth);
        self
    }

    pub fn connect_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.connect_timeout = timeout;
        self
    }

    pub async fn connect<T: ToSocketAddrs>(&self, addr: T) -> Result<Client, ClientError> {
        let config = Arc::new(thrussh::client::Config::default());
        match time::timeout(
            self.connect_timeout,
            thrussh::client::connect(config, addr, Handler::default()),
        )
        .await
        {
            Ok(Ok(handle)) => {
                let mut client = Client { inner: handle };
                match &self.auth {
                    Some(AuthMethod::Password(password)) => {
                        client.auth_with_password(&self.username, password).await?
                    }
                    None => {}
                }
                Ok(client)
            }
            Ok(Err(err)) => return Err(ClientError::ClientFailed(err)),
            Err(_) => return Err(ClientError::Timeout),
        }
    }
}

pub struct Client {
    inner: thrussh::client::Handle<Handler>,
}

impl Client {
    pub fn builder(username: impl ToString) -> ClientBuilder {
        ClientBuilder::new(username)
    }

    pub(crate) async fn auth_with_password(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<(), ClientError> {
        match self.inner.authenticate_password(username, password).await {
            Ok(true) => Ok(()),
            Ok(false) => Err(ClientError::AuthFailed(String::from(
                "username or password is wrong!",
            ))),
            Err(e) => Err(ClientError::ClientFailed(e)),
        }
    }

    #[allow(unused_variables)]
    pub async fn output(&mut self, command: &str) -> Result<Output, ClientError> {
        let mut channel = self.inner.channel_open_session().await?;
        channel.exec(true, command).await?;
        let mut res = Output::default();
        while let Some(msg) = channel.wait().await {
            match msg {
                thrussh::ChannelMsg::Data { ref data } => {
                    res.stdout.write_all(&data)?;
                }
                thrussh::ChannelMsg::ExtendedData { ref data, ext } => {
                    res.stderr.write_all(&data)?;
                }
                thrussh::ChannelMsg::ExitStatus { exit_status } => {
                    res.code = Some(exit_status);
                }
                _ => {}
            }
        }
        Ok(res)
    }

    pub async fn close(&mut self) -> Result<(), ClientError> {
        self.inner
            .disconnect(thrussh::Disconnect::ByApplication, "", "English")
            .await?;
        Ok(())
    }
}
