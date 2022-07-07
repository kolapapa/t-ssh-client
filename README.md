# t-ssh-client
Rust async `ssh client` wrapped by [thrussh](https://pijul.org/thrussh).

[![Crates.io](https://img.shields.io/crates/v/t-ssh-client.svg)](https://crates.io/crates/t-ssh-client)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/kolapapa/t-ssh-client/blob/main/LICENSE)
[![API docs](https://docs.rs/t-ssh-client/badge.svg)](http://docs.rs/t-ssh-client)

## Example
```rust
use std::env;
use std::time::Duration;

use t_ssh_client::{AuthMethod, Client, PasswordAuth};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pa = PasswordAuth::new("admin", &env::var("PASSWORD")?);

    let mut client = Client::builder()
        .auth(AuthMethod::Password(pa))
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

This project is licensed under the [MIT license].

[MIT license]: https://github.com/kolapapa/t-ssh-client/blob/main/LICENSE
