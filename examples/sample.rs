use std::env;
use std::time::Duration;

use t_ssh_client::{AuthMethod, Client, Password};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let password = Password::new("admin", &env::var("PASSWORD")?);

    let mut client = Client::builder()
        .auth(AuthMethod::Password(password))
        .connect_timeout(Duration::from_secs(2))
        .connect("192.168.62.1:22")
        .await?;
    println!("login success");

    let output = client.output("echo 'hello, world!'").await?;
    assert_eq!(output.stdout_string(), "hello, world!\n");

    Ok(())
}
