use t_ssh_client::Session;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sess = Session::connect("192.168.62.1:22").await?;
    if sess.auth("username", "password").await? {
        let output = sess.execute("df -h").await?;
        if output.success() {
            println!("{}", output.stdout());
        } else {
            println!("{}", output.stderr());
        }
    } else {
        println!("login failed!");
    }
    sess.close().await?;
    Ok(())
}
