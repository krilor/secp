use clap::Parser;
use anyhow::{Context, Result};
use log::{info,warn};

#[derive(Parser)]
/// Copy files to a remote machine with sudo on the other end
struct Secp {
    /// The user to become on the remote host. Think sudo -u <sudo_user>
    sudo_user: String,
    /// File source to copy
    source: String,
    /// File destination. Format <user>@<host>:<path>
    destination: String,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Secp::parse();

    info!("got file {}",&args.source);

    let content =  std::fs::read_to_string(&args.source)
        .with_context(|| format!("not able to read file `{}`", &args.source))?;

    println!("{}", content);
    Ok(())

}
