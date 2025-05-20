use interface::{InitializationPacket, NetworkPacket, Sendable};
use std::{
    io::{prelude::*, BufReader},
    net::{SocketAddr, TcpStream, UdpSocket},
    thread, time,
};

const LOOP_DELAY_TIME: u64 = 2;

pub fn run_server() -> Result<(), std::io::Error> {
    // set addresses
    let remote_server = std::env!("SERVER");
    let udp_port = std::env!("UDP_PORT");
    let tcp_port = std::env!("TCP_PORT");
    let tcp_destination = format!("{}:{}", remote_server, tcp_port);
    let udp_destination = format!("{}:{}", remote_server, udp_port);
    let local_addr = "0.0.0.0:8004";

    match initalize_connection(&tcp_destination) {
        Err(e) => {
            log::error!("Encountered Error initializing connection! {}", e);
            panic!()
        }
        Ok(_) => log::info!("successfuly connected"),
    }

    let remote_addr: SocketAddr = udp_destination.parse().unwrap();

    log::info!("Binding to Local Address: {}", local_addr);
    log::info!("Connecting to Remote Address: {}", remote_addr);

    match UdpSocket::bind(local_addr) {
        Ok(sock) => {
            log::info!("successfully bound to {}", &local_addr);
            loop {
                // sock.send_to(data, remote_addr).unwrap();
                handle_loop(&sock, remote_addr);
                thread::sleep(time::Duration::from_secs(LOOP_DELAY_TIME));
            }
        }
        Err(e) => log::error!("Unable to connect to server due to: {}", e),
    }
    Ok(())
}

fn initalize_connection<'a>(destination: &'a str) -> std::io::Result<String> {
    log::info!("Initializing connection with {}", destination);
    let mut stream = TcpStream::connect(&destination)?;
    let contents = InitializationPacket {
        version: "0.0".to_string(),
        location: "kitchen".to_string(),
        data_map: vec!["temperature".to_string(), "humidity".to_string()],
        measureands: vec!["temperature".to_string(), "humidity".to_string()],
        units: vec!["C".to_string(), "".to_string()],
    };
    stream.write_all(&contents.to_bytes().unwrap())?;
    let mut buf = String::new();
    let mut buf_reader = BufReader::new(stream);

    buf_reader.read_to_string(&mut buf)?;
    Ok(buf)
}

fn handle_loop(sock: &UdpSocket, remote_addr: SocketAddr) {
    let data = NetworkPacket {
        data: vec![32., 43.],
        ..Default::default()
    };
    match sock.send_to(&data.to_bytes().unwrap(), remote_addr) {
        Ok(len) => log::info!("{:?} bytes sent to {}", len, remote_addr),
        Err(e) => log::error!("Error in sending message to address: {}", e),
    };
}
