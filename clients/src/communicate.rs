use interface::{InitializationPacket, NetworkPacket, Sendable};
use std::{
    io::{prelude::*, BufReader},
    net::{SocketAddr, TcpStream, UdpSocket},
    thread, time,
};

const LOOP_DELAY_TIME: u64 = 2;

struct Connection {
    config: Vec<u8>,
    udp_endpoint: SocketAddr,
    tcp_endpoint: SocketAddr,
    sock: UdpSocket,
}

impl Connection {
    pub fn from(
        config: Vec<u8>,
        tcp_endpoint: SocketAddr,
        udp_endpoint: SocketAddr,
    ) -> UnconfiguredConnection {
        UnconfiguredConnection {
            config,
            tcp_endpoint,
            udp_endpoint,
        }
    }
}

trait ClientCommunication {
    fn send(self: Box<Self>) -> std::io::Result<Box<dyn ClientCommunication>>;
    fn check_connection(self: Box<Self>) -> std::io::Result<Box<dyn ClientCommunication>>;
}

struct UnconfiguredConnection {
    config: Vec<u8>,
    tcp_endpoint: SocketAddr,
    udp_endpoint: SocketAddr,
}

impl ClientCommunication for UnconfiguredConnection {
    fn send(self: Box<Self>) -> std::io::Result<Box<dyn ClientCommunication>> {
        {
            log::info!("Initializing connection with {}", self.tcp_endpoint);
            let mut stream = TcpStream::connect(self.tcp_endpoint)?;
            stream.write_all(&self.config)?;
            let mut buf = String::new();
            let mut buf_reader = BufReader::new(stream);

            buf_reader.read_to_string(&mut buf)?;
            assert_eq!(buf, "200");

            log::info!(
                "Connection with {} Successful! binding to Udp Socket: {}",
                self.tcp_endpoint,
                self.udp_endpoint
            );
        };

        let sock = UdpSocket::bind(self.udp_endpoint)?;

        log::info!("Successfully bound to {}", self.udp_endpoint);

        Ok(Box::new(Connection {
            config: self.config,
            sock,
            udp_endpoint: self.udp_endpoint,
            tcp_endpoint: self.tcp_endpoint,
        }))
    }

    fn check_connection(self: Box<Self>) -> std::io::Result<Box<dyn ClientCommunication>> {
        self.send()
    }
}

impl ClientCommunication for Connection {
    fn send(self: Box<Self>) -> std::io::Result<Box<dyn ClientCommunication>> {
        let data = NetworkPacket {
            data: vec![32., 43.],
            ..Default::default()
        };
        log::info!("Attempting to send data to {}", self.udp_endpoint);

        match self
            .sock
            .send_to(&data.to_bytes().unwrap(), self.udp_endpoint)
        {
            Ok(len) => log::info!("{:?} bytes sent to {}", len, self.udp_endpoint),
            Err(e) => log::error!("Error in sending message to address: {}", e),
        };
        Ok(self)
    }

    fn check_connection(self: Box<Self>) -> std::io::Result<Box<dyn ClientCommunication>> {
        log::info!("Checking connection with {}", self.tcp_endpoint);
        let mut stream = TcpStream::connect(&self.tcp_endpoint)?;
        stream.write_all(&self.config)?;
        let mut buf = String::new();
        let mut buf_reader = BufReader::new(stream);

        buf_reader.read_to_string(&mut buf)?;
        match buf.as_str() {
            "200" => return Ok(self),
            _ => {
                return Err(std::io::Error::other(format!(
                    "Server returned non-200: {}",
                    buf
                )))
            }
        }
    }
}

pub fn run_server() -> Result<(), std::io::Error> {
    // set addresses
    let remote_server = std::env!("SERVER");
    let udp_port = std::env!("UDP_PORT");
    let tcp_port = std::env!("TCP_PORT");
    let tcp_destination = format!("{}:{}", remote_server, tcp_port);
    let udp_destination = format!("{}:{}", remote_server, udp_port);
    let local_addr = "0.0.0.0:8004";

    let init_packet = InitializationPacket {
        version: "0.0".to_string(),
        location: "kitchen".to_string(),
        data_map: vec!["temperature".to_string(), "humidity".to_string()],
        measureands: vec!["temperature".to_string(), "humidity".to_string()],
        units: vec!["C".to_string(), "".to_string()],
    };

    let udp_addr: SocketAddr = udp_destination.parse().unwrap();
    let tcp_addr: SocketAddr = tcp_destination.parse().unwrap();

    log::info!("Binding to Local Address: {}", local_addr);
    log::info!("Connecting to Remote Address: {}", udp_addr);

    let mut connection: Box<dyn ClientCommunication> = Box::new(Connection::from(
        init_packet.to_bytes().unwrap(),
        tcp_addr,
        udp_addr,
    ));

    let mut counter = 0;
    loop {
        connection = connection.send()?;
        counter += 1;
        thread::sleep(time::Duration::from_secs(LOOP_DELAY_TIME));
        if counter > 12 {
            connection = connection.check_connection()?;
            counter = 0;
        }
    }
}
