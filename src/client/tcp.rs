use std::io::{BufRead, BufReader, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::str::FromStr;

use client::AIClient;
use common;
use protocol;

pub struct TcpClient {
    stream: TcpStream,
    reader: BufReader<TcpStream>,

    last_response: String,
}

impl AIClient for TcpClient {
    fn identify_as(&mut self, name: &str) {
        self.stream.write_fmt(format_args!("NAME {}\n", name))
            .expect("Failed while sending identification message");
    }

    fn wait_response(&mut self) -> bool {
        self.last_response.clear();
        let len = self.reader.read_line(&mut self.last_response).unwrap();
        len > 0
    }

    fn response(&self) -> Result<protocol::Message, protocol::Error> {
        protocol::Message::from_str(&self.last_response)
    }

    fn send_action(&mut self, direction: &common::Direction) {
        self.stream.write_fmt(format_args!("{}\n", direction))
            .expect("Failed while sending action message");
    }
}

pub fn connect<T: ToSocketAddrs>(addr: T) -> Option<TcpClient> {
    let stream = TcpStream::connect(addr);

    stream.ok().and_then(|x| {
        x.set_nodelay(true).ok();

        let input_stream = x.try_clone().unwrap();
        Some(TcpClient {
            stream: x,
            reader: BufReader::new(input_stream),
            last_response: String::new(),
        })
    })
}
