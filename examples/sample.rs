use std::env;
use std::time::Duration;

use t_ssh_client::{AuthMethod, Client};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let password = env::var("PASSWORD")?;

    let mut client = Client::builder()
        .username("admin")
        .auth(AuthMethod::Password(password))
        .connect_timeout(Duration::from_secs(2))
        .connect("192.168.62.1:22")
        .await?;
    println!("login success");

    let output = client.output("echo 'hello, world!'").await?;
    assert_eq!(output.stdout_string(), "hello, world!\n");

    Ok(())
}
