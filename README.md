# t-ssh-client
Rust async `ssh client` wrapped by [thrussh](https://pijul.org/thrussh).

## Example
```rust
use std::time::Duration;

use t_ssh_client::Session;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sess = Session::connect("192.168.65.1:22", Duration::from_secs(2)).await?;
    sess.auth_with_password("username", "password").await?;
    let output = sess.execute("df -h").await?;
    if output.success() {
        println!("[√] {}", output.stdout());
    } else {
        println!("[x] {}", output.stderr());
    }
    sess.close().await?;
    Ok(())
}
```

## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/kolapapa/t-ssh-client/blob/main/LICENSE
