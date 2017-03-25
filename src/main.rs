#[macro_use]
extern crate clap;
extern crate pathfinding;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use clap::{App, Arg};
use std::net::SocketAddrV4;
use std::str::FromStr;

mod ai;
mod client;
mod common;
mod game;
mod protocol;
mod traits;

use client::AIClient;
use protocol::Message;

const ARG_IP: &'static str = "ip";
const ARG_PORT: &'static str = "port";

fn main() {
    let arguments = App::new("Allie")
        .about("Bot for the AI competition at The Gathering 2017")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::with_name(ARG_IP)
            .long("ip")
            .value_name("IP")
            .help("IP address\t(default 127.0.0.1)")
            .takes_value(true))
        .arg(Arg::with_name(ARG_PORT)
            .long("port")
            .value_name("PORT")
            .help("TCP port\t(default 54321)")
            .takes_value(true))
        .get_matches();

    let host = {
        let ip = arguments.value_of(ARG_IP).unwrap_or("127.0.0.1");
        let port = arguments.value_of(ARG_PORT).unwrap_or("54321");
        SocketAddrV4::from_str(&format!("{}:{}", ip, port))
    };

    if host.is_err() {
        println!("Invalid IP address or port ({})", host.err().unwrap());
        std::process::exit(1);
    }

    let client = client::tcp::connect(host.unwrap());
    if client.is_none() {
        println!("Failed to connect to server");
        std::process::exit(1);
    }

    // We're connected, go go go
    let mut client = client.unwrap();
    client.identify_as("Allie");

    let mut bot: Option<ai::Bot> = None;

    while client.wait_response() {
        let response = client.response();

        if response.is_err() {
            println!("Response error: {:?}", response.err().unwrap());
            continue;
        }

        match response.unwrap() {
            Message::Welcome { state } => {
                println!("Received welcome message, initializing bot");
                bot = Some(ai::Bot::from_game_state(state));
            }
            Message::StartOfRound => {
                if let Some(ref mut x) = bot {
                    x.reset();
                }
            }
            Message::Update { state } => {
                match bot {
                    Some(ref mut x) => client.send_action(x.determine_action(state)),
                    None => debug_assert!(false, "Received stateupdate message while not having an initialized AI"),
                }
            }
            Message::Dead | Message::EndOfRound => {
                // Nothing special to do here
            }
        }
    }
}
