use interface::{NetworkPacket, UdpAble};
use std::net::UdpSocket;
// use tokio::net::UdpSocket;

// #[tokio::main]
fn main() -> Result<(), std::io::Error> {
    dotenv::dotenv().ok();
    let port = std::env::var("PORT").expect("Need to set PORT env variable.");

    let sock = UdpSocket::bind(format!("0.0.0.0:{}", port))?;
    println!("opened udp socket at port: {:?}", port);

    let mut buf = [0; 1024];

    loop {
        let (len, addr) = sock.recv_from(&mut buf)?;
        println!("Recieved message of length: {}, from: {}", len, addr);
        let recieved_data = NetworkPacket::from_bytes(&buf);
        println!("{:?}", recieved_data);
    }
}
