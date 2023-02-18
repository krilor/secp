use clap::Parser;
use anyhow::{Context, Result};
use log::{info,warn};

use std::io::prelude::*;
use std::net::TcpStream;
use ssh2::Session;

#[derive(Parser)]
/// Copy files to a remote machine with sudo on the other end
struct Secp {
    /// The user to become on the remote host. Think sudo -u <sudo_user>
    #[arg(short='u', long)]
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

    // println!("{}", content);


    // SSH connect
    let tcp = TcpStream::connect("localhost:4444").unwrap();
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();

    // groke has sudo NOPASSWD
    //sess.userauth_password("groke","grokepwd").unwrap();

    // joxter requires pwd for sudo
    sess.userauth_password("joxter","joxterpwd").unwrap();

    let mut chan = sess.channel_session().unwrap();
    // chan.handle_extended_data(ssh2::ExtendedData::Merge).unwrap();
    let mut stderr = chan.stderr();


    // do sudo all the things!
    info!("firing command");
    chan.exec("echo -n 'BEGIN' && sudo -S -H -p 'GIMMEYOURPWDPLEASE' /bin/sh -c '/usr/bin/scp -tr ~/'").unwrap();

    info!("I'm gonna do it");

    // lets read stderr to see if sudo is up to something
    let mut serr = String::new();
    loop {
        let mut buferr = [0u8; 1024];
        match stderr.read(&mut buferr){
            Ok(0) => {
                info!("got EOF on the channel");
                break;
            },
            Ok(c) => {
                serr.push_str(&String::from_utf8_lossy(&buferr[..c]));
                info!("got {} bytes in stderr", c);

                if serr.ends_with("GIMMEYOURPWDPLEASE") {
                    info!("time to input password");
                    chan.write(b"joxterpwd\n").unwrap();
                    break;
                } else if serr.ends_with("WEMADEIT") {
                    info!("ready to continue");
                    break;
                }
            }
            Err(e) => {
                warn!("Failed while reading: {}", e)
            }
        }
    }

    info!("stderr is {}", serr);


    // lets pretend that the above implemenatation works nicely and that we now are the requested user
    // lets start scp in recieve mode on remote
    // SCP notes:
    // https://web.archive.org/web/20170215184048/https://blogs.oracle.com/janp/entry/how_the_scp_protocol_works
    // https://en.wikipedia.org/wiki/Secure_copy#cite_note-Pechanec-2
    // https://gist.github.com/jedy/3357393

    info!("scp started, im gonna send some data");

    let content = b"12345";

    write!(chan, "C0644 {} newfile\n", content.len()).unwrap();
    chan.write(content).unwrap();
    write!(chan, "\x00").unwrap();



    info!("should be done sending now");

    chan.send_eof().unwrap();
    chan.wait_eof().unwrap();
    chan.close().unwrap();

    info!("waiting for session to close");
    chan.wait_close().unwrap();
    info!("it closed!");
    Ok(())
}
