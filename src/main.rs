extern crate clap;

use clap::{App, Arg};
use std::net::SocketAddrV4;
use std::str::FromStr;

const ARG_IP: &'static str = "ip";
const ARG_PORT: &'static str = "port";

fn main() {
    let arguments = App::new("Allie")
        .version("0.1.0")
        .about("Bot for the AI competition at The Gathering 2017")
        .author("Jimmy 'Kwarf' Bergström <thekwarf@gmail.com>")
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
        println!("Invalid IP address or port");
        std::process::exit(1);
    }
}
