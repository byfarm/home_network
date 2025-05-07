use std::net::UdpSocket;

// #[tokio::main]
fn main() {
    dotenv::dotenv().ok();

    // let listener = TcpListener::bind().unwrap();
    let sock = UdpSocket::bind(format!("0.0.0.0:{}", std::env::var("PORT").unwrap())).unwrap();
    println!(
        "opened udp socket at port: {:?}",
        std::env::var("PORT").unwrap()
    );
    let mut buf = [0; 1024];

    loop {
        let (len, addr) = sock.recv_from(&mut buf).unwrap();
        println!("Recieved message of length: {}, from: {}", len, addr);
        println!("Message: {:?}", buf);
        match sock.send_to(&buf[..len], addr) {
            Ok(v) => println!("num_bytes_sent: {}", v),
            Err(e) => eprintln!("Error in sending thru socket: {}", e),
        }
    }
}
