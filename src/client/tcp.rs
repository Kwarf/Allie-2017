use std::io::{BufRead, BufReader, Write};
use std::net::{TcpStream, ToSocketAddrs};

use client::AIClient;

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
        let len = self.reader.read_line(&mut self.last_response).unwrap();
        len > 0
    }

    fn response(&self) -> &str {
        &self.last_response
    }
}

pub fn connect<T: ToSocketAddrs>(addr: T) -> Option<TcpClient> {
    let stream = TcpStream::connect(addr);

    stream.ok().and_then(|x| {
        let input_stream = x.try_clone().unwrap();
        Some(TcpClient {
            stream: x,
            reader: BufReader::new(input_stream),
            last_response: String::new(),
        })
    })
}
