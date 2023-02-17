use clap::Parser;

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

fn main() {
    let args = Secp::parse();

    let content = std::fs::read_to_string(&args.source).expect("not able to read file");
    println!("{}", content);

}
