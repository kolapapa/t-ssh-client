use std::io::Write;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use std::time::Duration;

use tokio::{fs, time};

use crate::error::ClientError;
use crate::{Handler, Output};

pub enum AuthMethod {
    Password(String),
    Key(String),
}

pub struct ClientBuilder {
    username: String,
    auth: Option<AuthMethod>,
    connect_timeout: Duration,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self {
            username: String::default(),
            auth: None,
            connect_timeout: Duration::from_secs(10),
        }
    }

    pub fn username<S: ToString>(&mut self, username: S) -> &mut Self {
        self.username = username.to_string();
        self
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
                let mut client = Client {
                    inner: handle,
                    username: self.username.clone(),
                };
                match &self.auth {
                    Some(AuthMethod::Password(pass)) => client.auth_with_password(&pass).await?,
                    Some(AuthMethod::Key(path)) => {
                        let secret: String = fs::read_to_string(path).await?;
                        let key_pair = thrussh_keys::decode_secret_key(&secret, None)?;
                        client.auth_with_key_pair(Arc::new(key_pair)).await?
                    }
                    None => {}
                }
                Ok(client)
            }
            Ok(Err(err)) => return Err(err),
            Err(_) => return Err(ClientError::Timeout),
        }
    }
}

pub struct Client {
    username: String,
    inner: thrussh::client::Handle<Handler>,
}

impl Client {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    pub(crate) async fn auth_with_password(&mut self, password: &str) -> Result<(), ClientError> {
        match self
            .inner
            .authenticate_password(&self.username, password)
            .await
        {
            Ok(true) => Ok(()),
            Ok(false) => Err(ClientError::AuthFailed(String::from(
                "username or password is wrong!",
            ))),
            Err(e) => Err(ClientError::ClientFailed(e)),
        }
    }

    pub(crate) async fn auth_with_key_pair(
        &mut self,
        key: Arc<thrussh_keys::key::KeyPair>,
    ) -> Result<(), ClientError> {
        match self.inner.authenticate_publickey(&self.username, key).await {
            Ok(true) => Ok(()),
            Ok(false) => Err(ClientError::AuthFailed(String::from(
                "username or key is wrong!",
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
