use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use async_sqlite::{JournalMode, Pool, PoolBuilder};
use interface::{BUFFER_SIZE, InitializationPacket, NetworkPacket, Sendable};
use tokio::{
    io::{AsyncReadExt as _, BufReader},
    net::{TcpListener, TcpStream, UdpSocket},
    sync::Mutex,
};

type AddressLookup = Arc<Mutex<HashMap<SocketAddr, InitializationPacket>>>;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv::dotenv().ok();
    let udp_port = std::env::var("UDP_PORT").expect("Need to set UDP_PORT env variable.");
    let tcp_port = std::env::var("TCP_PORT").expect("Need to set TCP_PORT env variable.");

    let tcp_listener = TcpListener::bind(format!("0.0.0.0:{}", tcp_port)).await?;

    let udp_sock = UdpSocket::bind(format!("0.0.0.0:{}", udp_port)).await?;
    println!("opened udp socket at port: {:?}", udp_port);

    let mut udp_buf = [0; BUFFER_SIZE];

    let address_lookup: AddressLookup = Arc::new(Mutex::new(HashMap::new()));

    let pool = Arc::new(
        PoolBuilder::new()
            .path("db.sqlite3")
            .journal_mode(JournalMode::Delete)
            .open()
            .await
            .expect("Unable to open new database pool."),
    );

    loop {
        // accept incoming config requests
        let (stream, socket_address) = tcp_listener.accept().await?;

        let lookup_clone = address_lookup.clone();

        // initialize the new connection
        tokio::spawn(
            async move { handle_initialization(socket_address, stream, lookup_clone).await },
        );

        // accept new udp packets
        let (len, addr) = udp_sock.recv_from(&mut udp_buf).await?;
        println!("Recieved message of length: {}, from: {}", len, addr);

        let recieved_data = NetworkPacket::from_bytes(&udp_buf)
            .expect("Unable to decode bytes into a readable state.");

        let pool_clone = pool.clone();
        let lookup_clone = address_lookup.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_data(&addr, pool_clone, recieved_data, lookup_clone).await {
                eprintln!("Could not handle data from {}: {}", addr, e);
            };
        });
    }
}

async fn handle_initialization(
    socket_addr: SocketAddr,
    stream: TcpStream,
    address_lookup: AddressLookup,
) -> std::io::Result<()> {
    let mut address_lookup = address_lookup.lock().await;

    if address_lookup.contains_key(&socket_addr) {
        println!("{:?} already exists in config!", socket_addr)
    }

    let mut tcp_buf = String::new();
    let mut buf_reader = BufReader::new(stream);

    buf_reader.read_to_string(&mut tcp_buf).await?;
    // not the most efficient but works
    let init_packet = InitializationPacket::from_bytes(tcp_buf.as_bytes()).unwrap();

    address_lookup.insert(socket_addr, init_packet);

    Ok(())
}

async fn handle_data(
    socket_addr: &SocketAddr,
    pool: std::sync::Arc<Pool>,
    packet: NetworkPacket,
    address_lookup: AddressLookup,
) -> Result<(), async_sqlite::Error> {
    let address_lookup = address_lookup.lock().await;
    match address_lookup.get(socket_addr) {
        Some(v) => {
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
                            v.units.first().unwrap(),
                        ],
                    )
                })
                .await
            {
                Ok(rows) => {
                    println!("Uploaded data to database! Rows affected: {}", rows);
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        None => {
            eprintln!("{} not found in hashmap!", socket_addr);
            Ok(())
        }
    }
}
