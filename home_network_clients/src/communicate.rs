use std::io::prelude::*;
use std::io::BufReader;

use std::net::TcpStream;

const END_MESSAGE: &str = "\r\n\r\n";

pub fn run_server() -> Result<(), std::io::Error> {
    loop {
        if let Ok(mut stream) = TcpStream::connect(std::env!("SERVER")) {
            let mut message = String::from("Hello from esp");
            message.push_str(END_MESSAGE);

            stream.write_all(message.as_bytes()).unwrap();

            let mut response = String::new();

            let mut buf_reader = BufReader::new(&stream);

            while let Ok(bytes_read) = buf_reader.read_line(&mut response) {
                if bytes_read == 0 || response.ends_with(END_MESSAGE) {
                    break;
                }
            }
        } else {
            log::error!("Unable to connect to server!")
        }

        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}
