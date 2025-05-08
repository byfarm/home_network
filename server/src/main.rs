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

    let pool = PoolBuilder::new()
        .path("db.sqlite3")
        .journal_mode(JournalMode::Wal)
        .open()
        .await
        .expect("Unable to open new database pool.");

    let shared_pool = std::sync::Arc::new(pool);

    loop {
        let (len, addr) = sock.recv_from(&mut buf).await?;
        println!("Recieved message of length: {}, from: {}", len, addr);

        let recieved_data =
            NetworkPacket::from_bytes(&buf).expect("Unable to decode bytes into a readable state.");

        let pool_clone = shared_pool.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_data(pool_clone, recieved_data).await {
                eprintln!("Could not handle data from {}: {}", addr, e);
            };
        });
    }
}

async fn handle_data(
    pool: std::sync::Arc<Pool>,
    packet: NetworkPacket,
) -> Result<(), async_sqlite::Error> {
    match pool
        .conn(move |conn| {
            conn.execute(
                "INSERT INTO
                data (location_id, value, measurand, units)
            SELECT
                location.id,
                ?2,
                ?3,
                ?4
            FROM location
            WHERE
                location.name = ?1
            LIMIT 1",
                async_sqlite::rusqlite::params![
                    "kitchen",
                    *packet.data.first().unwrap(),
                    "temperature",
                    packet.units,
                ],
            )
        })
        .await
    {
        Ok(v) => {
            println!("uploaded data to database! rows affected: {}", v);
            Ok(())
        }
        Err(e) => {
            Err(e)
        }
    }
}
