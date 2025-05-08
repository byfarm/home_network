use async_sqlite::{JournalMode, Pool, PoolBuilder};
use interface::{BUFFER_SIZE, NetworkPacket, UdpAble};
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv::dotenv().ok();
    let port = std::env::var("PORT").expect("Need to set PORT env variable.");

    let sock = UdpSocket::bind(format!("0.0.0.0:{}", port)).await?;
    println!("opened udp socket at port: {:?}", port);

    let mut buf = [0; BUFFER_SIZE];

    let pool = Box::new(
        PoolBuilder::new()
            .path("db.sqlite3")
            .journal_mode(JournalMode::Wal)
            .open()
            .await
            .unwrap(),
    );

    loop {
        let (len, addr) = sock.recv_from(&mut buf).await?;
        println!("Recieved message of length: {}, from: {}", len, addr);
        let recieved_data = NetworkPacket::from_bytes(&buf).unwrap();

        handle_data(pool.clone(), recieved_data).await.unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

async fn handle_data(pool: Box<Pool>, packet: NetworkPacket) -> Result<(), std::io::Error> {
    pool.conn(move |conn| {
        conn.execute(
            "INSERT INTO location (location, temperature) VALUES (?1, ?2)",
            (packet.location.clone(), *packet.data.first().unwrap()),
        )
    })
    .await
    .unwrap();

    Ok(())
}
