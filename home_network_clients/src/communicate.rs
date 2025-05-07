use std::net::{SocketAddr, UdpSocket};

pub fn run_server() -> Result<(), std::io::Error> {
    loop {
        let destination = format!("{}:{}", std::env!("SERVER"), std::env!("PORT"));

        // let mut buf = [0; 1024];
        let message = "Hello via udp!";

        let addr: SocketAddr = destination.parse().unwrap();
        let local_addr = "0.0.0.0:8004";
        log::warn!("Remote Address: {}", addr);
        log::warn!("Local Address: {}", local_addr);

        match UdpSocket::bind(local_addr) {
            Ok(sock) => {
                log::info!("successfully bound to {}", &local_addr);
                loop {
                    match sock.send_to(message.as_bytes(), addr) {
                        Ok(len) => log::info!("{:?} bytes sent to {}", len, addr),
                        Err(e) => log::error!("Error in sending message to address: {}", e),
                    };

                    std::thread::sleep(std::time::Duration::from_secs(2));
                }
            }
            Err(e) => log::error!("Unable to connect to server due to: {}", e),
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}
