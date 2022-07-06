use t_ssh_client::Session;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ssh = Session::connect("192.168.62.1:22").await?;
    match ssh.auth("username", "password").await? {
        true => {
            let output = ssh.execute("df -h").await?;
            if output.success() {
                println!("{}", output.stdout());
            } else {
                println!("{}", output.stderr());
            }
        }
        false => {
            println!("login failed!");
        }
    }
    Ok(())
}
