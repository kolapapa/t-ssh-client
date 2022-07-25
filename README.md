# t-ssh-client
Rust async `ssh client` wrapped by [thrussh](https://pijul.org/thrussh).

[![Crates.io](https://img.shields.io/crates/v/t-ssh-client.svg)](https://crates.io/crates/t-ssh-client)
[![Apache-2.0 licensed](https://img.shields.io/crates/l/t-ssh-client/0.1.2)](https://github.com/kolapapa/t-ssh-client/blob/main/LICENSE)
[![API docs](https://docs.rs/t-ssh-client/badge.svg)](http://docs.rs/t-ssh-client)

## Example
```rust
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
```

## License

This project is licensed under the [Apache-2.0 license].

[Apache-2.0 license]: https://github.com/kolapapa/t-ssh-client/blob/main/LICENSE
