use clap::Parser;
use anyhow::{Context, Result};
use log::{info,warn};

use std::io::prelude::*;
use std::net::TcpStream;
use ssh2::Session;

use std::sync::mpsc;
use std::thread;

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser)]
/// Copy files to a remote machine with sudo on the other end
struct Secp {
    /// The user to become on the remote host. Think sudo -u <sudo_user>
    #[arg(short='u', long)]
    sudo_user: String,
    /// SSH and sudo password for connection user
    #[arg(short='p', long)]
    password: String,
    /// File source to copy
    source: String,
    /// File destination. Format <user>@<host>:<path>
    destination: String,
}

/* Our magic messages that we are looking for in stdin and stderr */

const PASSWORDPROMPT: &str = "GIMMEYOURPASSWORD";
const ESCALATIONSUCCEDED: &str = "LETSGODOOURTHING";

fn main() -> Result<()> {
    env_logger::init();
    let args = Secp::parse();

    info!("got file {}",&args.source);

    let content =  std::fs::read_to_string(&args.source)
        .with_context(|| format!("not able to read file `{}`", &args.source))?;

    let iat = args.destination.find('@');
    //let icolon = args.destination.find(':');
    let user = &args.destination[..iat.unwrap()];


    // SSH connect
    let tcp = TcpStream::connect("localhost:4444").unwrap();
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();


    // joxter requires pwd for sudo
    sess.userauth_password(user,&args.password).unwrap();

    let mut chan = sess.channel_session().unwrap();
    chan.handle_extended_data(ssh2::ExtendedData::Merge).unwrap();

    info!("firing command");
    chan.exec(&format!("echo -n 'BEGIN' && sudo -S -H -p '{}' /bin/sh -c 'echo -n {} && /usr/bin/scp -tr ~/'", PASSWORDPROMPT, ESCALATIONSUCCEDED)).unwrap();
    info!("I'm gonna do it");


    let mut out = String::new();
    loop {
        let mut buf = [0u8; 1024];
        match chan.read(&mut buf){
            Ok(0) => {
                info!("eof in session");
                break;
            },
            Ok(c) => {

                let s = String::from_utf8_lossy(&buf[..c]);
                out.push_str(&s);

                info!("got {} bytes from session: {}", c, s);

                if out.ends_with(ESCALATIONSUCCEDED) {
                    info!("Im done!");
                    break;
                } else if out.ends_with(PASSWORDPROMPT) {
                    info!("writing password");
                    chan.write_fmt(format_args!("{}\n",&args.password)).unwrap();
                }
            }
            Err(e) => {
                warn!("Failed while reading: {}", e)
            }
        }
    }

    // lets pretend that the above implemenatation works nicely and that we now are the requested user
    // lets start scp in recieve mode on remote
    // SCP notes:
    // https://web.archive.org/web/20170215184048/https://blogs.oracle.com/janp/entry/how_the_scp_protocol_works
    // https://en.wikipedia.org/wiki/Secure_copy#cite_note-Pechanec-2
    // https://gist.github.com/jedy/3357393

    info!("scp started, im gonna send some data");


    let filename: String = format!("newfile-{}", SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs());

    write!(chan, "C0644 {} {}\n", content.len(), filename).unwrap();
    chan.write_fmt(format_args!("{}",content)).unwrap();
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
