use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use async_sqlite::{JournalMode, Pool, PoolBuilder};
use interface::{BUFFER_SIZE, InitializationPacket, NetworkPacket, Sendable};
use thiserror::Error;
use tokio::{
    io::{AsyncBufReadExt as _, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream, UdpSocket},
    sync::Mutex,
};

type AddressLookup = Arc<Mutex<HashMap<IpAddr, InitializationPacket>>>;

#[derive(Error, Debug)]
enum ConfigError {
    #[error("Client not properly Configured in server: {0}")]
    NotConfigured(String),
}

enum SentDataResult<T, E, ER> {
    Ok(T),
    Err(E),
    CfgErr(ER),
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv::dotenv().ok();
    let udp_port = std::env::var("UDP_PORT").expect("Need to set UDP_PORT env variable.");
    let tcp_port = std::env::var("TCP_PORT").expect("Need to set TCP_PORT env variable.");

    let tcp_listener = TcpListener::bind(format!("0.0.0.0:{}", tcp_port)).await?;
    println!("opened tcp listener at port: {:?}", tcp_port);

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
        tokio::select! {
            tcp_result = tcp_listener.accept() => {
                // accept incoming config requests
                let (stream, socket_address) = tcp_result?;

                let lookup_clone = address_lookup.clone();

                // initialize the new connection
                tokio::spawn(
                    async move { handle_initialization(socket_address, stream, lookup_clone).await },
                );
            },
            udp_result = udp_sock.recv_from(&mut udp_buf) => {
                // accept new udp packets
                let (len, addr) = udp_result?;
                println!("Recieved message of length: {}, from: {}", len, addr);

                let pool_clone = pool.clone();
                let lookup_clone = address_lookup.clone();

                let recieved_data = NetworkPacket::from_bytes(&udp_buf)
                    .expect("Unable to decode bytes into a readable state.");

                tokio::spawn(async move {
                    match handle_data(&addr, pool_clone, recieved_data, lookup_clone).await {
                        SentDataResult::Ok(_) => {},
                        SentDataResult::Err(e) => {eprintln!("unable to recieve data due to {}", e)}
                        SentDataResult::CfgErr(e) => {eprintln!("Client not properly configured {}", e); }
                    }
                });
            }
        }
    }
}

async fn handle_initialization(
    socket_addr: SocketAddr,
    stream: TcpStream,
    address_lookup: AddressLookup,
) -> std::io::Result<()> {
    println!("recieving message from {}", socket_addr);
    let mut address_lookup = address_lookup.lock().await;

    let mut tcp_buf = String::new();
    let mut buf_reader = BufReader::new(stream);

    if address_lookup.contains_key(&socket_addr.ip()) {
        println!("{:?} already exists in config!", socket_addr);
        buf_reader.write_all("200".as_bytes()).await?;
        return Ok(());
    }

    // buf_reader.read_to_string(&mut tcp_buf).await?;
    while let Ok(bytes_read) = buf_reader.read_line(&mut tcp_buf).await {
        if bytes_read == 0 || tcp_buf.ends_with("\n") {
            break;
        }
    }
    println!("message recieved: {}", tcp_buf);
    // not the most efficient but works
    let init_packet = InitializationPacket::from_bytes(tcp_buf.as_bytes()).unwrap();

    println!("recieved metadata: {:?}", init_packet);

    address_lookup.insert(socket_addr.ip(), init_packet);

    buf_reader.write_all("200".as_bytes()).await?;

    Ok(())
}

async fn handle_data(
    socket_addr: &SocketAddr,
    pool: std::sync::Arc<Pool>,
    packet: NetworkPacket,
    address_lookup: AddressLookup,
) -> SentDataResult<(), async_sqlite::Error, ConfigError> {
    let init_packet_option = {
        let address_lookup_guard = address_lookup.lock().await;
        address_lookup_guard.get(&socket_addr.ip()).cloned()
    };

    if let Some(metadata) = init_packet_option {
        // Clone the location *once* outside the loop to be the "base" for cloning
        let base_location = metadata.location.to_lowercase(); // metadata.location is moved here

        // Iterate over `packet.data` (owned), `metadata.units` (references),
        // and `metadata.measurands` (references).
        // Using `.zip()` repeatedly.
        for ((unit_ref, measureand_ref), value) in metadata
            .units
            .iter()
            .zip(metadata.measureands.iter())
            .zip(packet.data.into_iter())
        {
            // Clone the specific data needed for *this closure's parameters*
            // This is the point where the `String`s are truly prepared for the query.
            let location_param = base_location.clone(); // Clone for this specific query
            let unit_param = unit_ref.clone(); // Clone the `&String` to an owned `String`
            let measureand = measureand_ref.clone();

            match pool
                .conn(move |conn| {
                    // All variables captured by this `move` closure are now owned by it.
                    // `location_param`, `value`, `measurand_param`, `unit_param`
                    // are now independent copies, owned by this particular closure invocation.
                    let mut stmt = conn.prepare_cached(
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
                    )?;

                    println!(
                        "Uploaded data to database! Rows affected: {}, {}, {}",
                        value, unit_param, measureand
                    );

                    stmt.execute(async_sqlite::rusqlite::params![
                        location_param, // Now owned by the params! call
                        value,          // Copy (f64)
                        measureand,
                        unit_param, // Now owned by the params! call
                    ])
                })
                .await
            {
                Ok(rows) => {
                    if rows != 1 {
                        eprintln!("Warning: {} rows affected", rows)
                    }
                }
                Err(e) => {
                    eprintln!("Error uploading data to database: {:?}", e);
                    return SentDataResult::Err(e);
                }
            }
        }
        SentDataResult::Ok(())
    } else {
        eprintln!("{} not found in hashmap!", socket_addr);
        return SentDataResult::CfgErr(ConfigError::NotConfigured(socket_addr.to_string()));
    }
}
