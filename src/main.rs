use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use clap::Parser;
use crate::message::Message;

mod message;
mod component;

mod DFGM;
pub use DFGM::dfgm_interface::*;
pub use DFGM::dfgm_handler::*;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional IP address to use
    host: Option<String>,

    /// Port to listen on
    #[arg(short, long)]
    port: i32,
}

fn main() {
    let cli = Cli::parse();

    let host = match &cli.host {
        Some(hostname) => hostname,
        None => "localhost",
    };

    let port = cli.port;
    let ipaddr = host.to_owned() + ":" + &port.to_string();
    let listener = TcpListener::bind(ipaddr).unwrap();
    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on {host}:{port}");

    let mut components = component::init();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                handle_client(stream, &mut components);
            }
            Err(e) => {
                println!("Error: {}", e);
                /* connection failed */
            }
        }
    }
    // close the socket server
    drop(listener);
}

fn handle_client(mut stream: TcpStream, components: &mut [component::Component]) {
    let mut data = [0; message::MSG_LEN];
    while match stream.read(&mut data) {
        Ok(size) => {
            // echo everything!
            match size {
                0 => { println!("EOF");
                       false
                }
                _ => {
                    println!("read {size} bytes");
                    let response = handle_message(&data, components);
                    match stream.write(&response) {
                        Ok(wlen) => if size < message::MSG_LEN {
                            println!("short write: {wlen}");
                            false
                        }
                        else {
                            true
                        },
                        Err(e) => {
                            println!("write error: {e}");
                            false
                        }
                    }
                }
            }
        },
        Err(_) => {
            println!("An error occurred, terminating connection");
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}
}

fn handle_message(msg: &Message, components: &mut [component::Component]) -> Message {
    let cmd = message::Command::deserialize(msg);

    println!("handle_message: payload {0}, opcode {1}", cmd.payload, cmd.opcode);

    let target = &mut components[cmd.payload.id];
    match component::dispatch_cmd(target, &cmd) {
        Ok(msg) => msg,
        Err(_) => cmd.status_msg(255)
    }
}