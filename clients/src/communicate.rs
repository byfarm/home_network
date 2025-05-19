use std::{
    net::{SocketAddr, UdpSocket},
    thread, time,
};
use interface::{NetworkPacket, UdpAble};

const LOOP_DELAY_TIME: u64 = 2;

pub fn run_server() -> Result<(), std::io::Error> {
    // let mut buf = [0; 1024];

    // set addresses
    let destination = format!("{}:{}", std::env!("SERVER"), std::env!("PORT"));
    let remote_addr: SocketAddr = destination.parse().unwrap();
    let local_addr = "0.0.0.0:8004";

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

fn handle_loop(sock: &UdpSocket, remote_addr: SocketAddr) {
    let data = NetworkPacket {
        units: "C".to_string(),
        data: vec![32., 43., 54., 66.],
        location: "kitchen".to_string(),
        ..Default::default()
    };
    match sock.send_to(&data.to_bytes().unwrap(), remote_addr) {
        Ok(len) => log::info!("{:?} bytes sent to {}", len, remote_addr),
        Err(e) => log::error!("Error in sending message to address: {}", e),
    };
}
