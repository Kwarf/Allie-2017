#![cfg_attr(feature = "benchmarking", feature(test))]

#[macro_use]
extern crate clap;
extern crate itertools;
extern crate pathfinding;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use clap::{App, Arg};
use std::net::SocketAddrV4;
use std::str::FromStr;
use std::time::{Duration, Instant};

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

const V_MAJOR: &'static str = env!("CARGO_PKG_VERSION_MAJOR");
const V_MINOR: &'static str = env!("CARGO_PKG_VERSION_MINOR");

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
    if cfg!(debug_assertions) {
        client.identify_as("Allie DBG");
    }
    else {
        client.identify_as(&format!("Allie {}.{}", V_MAJOR, V_MINOR));
    }


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
                    Some(ref mut x) => {
                        let instant = Instant::now();
                        let action = x.determine_action(state);
                        // println!("Time to determine action: {:>10.3} ms", duration_in_ms(&instant.elapsed()));
                        client.send_action(&action);
                    },
                    None => debug_assert!(false, "Received stateupdate message while not having an initialized AI"),
                }
            }
            Message::Dead | Message::EndOfRound => {
                // Nothing special to do here
            }
        }
    }
}

fn duration_in_ms(duration: &Duration) -> f32 {
    (duration.as_secs() as f32 * 1000.0) + (duration.subsec_nanos() as f32 / 1_000_000.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_convert_duration_to_ms() {
        assert_eq!(2050.0, duration_in_ms(&Duration::new(2, 50_000_000)));
    }
}
