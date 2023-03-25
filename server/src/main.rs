use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt, TryStreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
#[cfg(feature = "tls")]
use tokio_native_tls::{native_tls, TlsAcceptor, TlsStream};
use tokio_tungstenite::{tungstenite, WebSocketStream};

use core_server::prelude::*;

#[cfg(feature = "tls")]
use std::{fs::File, io::Read};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration};//, SystemTime, UNIX_EPOCH};
/*

/// Gets the current time in milliseconds
fn get_time() -> u128 {
    let stop = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
        stop.as_millis()
}*/

#[cfg(feature = "tls")]
type Stream = WebSocketStream<TlsStream<TcpStream>>;

#[cfg(not(feature = "tls"))]
type Stream = WebSocketStream<TcpStream>;

type UuidPeerMap = FxHashMap<
    Uuid,
    SplitSink<Stream, tungstenite::Message>
>;

async fn handle_client_messages(
    ws_stream: Stream,
    server: Arc<Mutex<Server<'_>>>,
    uuid_endpoint: Arc<Mutex<UuidPeerMap>>,
) {
    let (sink, mut stream) = ws_stream.split();

    if !wait_for_login(&mut stream).await {
        return;
    }

    let uuid = server.lock().await.create_player_instance();
    println!("logged in anonymous {:?}", uuid);
    uuid_endpoint.lock().await.insert(uuid, sink);

    loop {
        if !uuid_endpoint.lock().await.contains_key(&uuid) {
            break;
        }

        let msg = stream.try_next().await;
        if msg.is_err() {
            server.lock().await.destroy_player_instance(uuid);
            uuid_endpoint.lock().await.remove(&uuid);
            println!("Client disconnected");
            break;
        }

        if let Some(msg) = msg.unwrap() {
            match msg {
                tungstenite::Message::Binary(bin) => {
                    let cmd : ServerCmd = ServerCmd::from_bin(&bin)
                        .unwrap_or(ServerCmd::NoOp);

                    match cmd {
                        ServerCmd::GameCmd(action) => {
                            server
                                .lock()
                                .await
                                .execute_packed_player_action(uuid, action)
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }
    }
}

async fn handle_server_messages(
    server: Arc<Mutex<Server<'_>>>,
    uuid_endpoint: Arc<Mutex<UuidPeerMap>>,
) {
    loop {
        tokio::time::sleep(Duration::from_millis(10)).await;

        let messages = server.lock().await.check_for_messages();

        for message in messages {
            match message {
                Message::PlayerUpdate(uuid, update) => {
                    let mut uuid_endpoint = uuid_endpoint.lock().await;

                    if let Some(sink) = uuid_endpoint.get_mut(&update.id) {
                        let cmd = ServerCmd::GameUpdate(update);

                        if let Some(bin) = cmd.to_bin() {
                            if sink
                                .send(tungstenite::Message::binary(bin))
                                .await
                                .is_err() {
                                    println!("Client disconnected");
                                    server.lock().await.destroy_player_instance(uuid);
                                    uuid_endpoint.remove(&uuid);
                                    break;
                                }
                        }
                    }
                },
                _ => {}
            }
        }
    }
}

#[cfg(feature = "tls")]
fn read_tls_acceptor(file_path: &PathBuf, password: &str) -> TlsAcceptor {
    let mut file = File::open(file_path).unwrap();

    let mut identity = vec![];
    file.read_to_end(&mut identity).unwrap();

    let identity = native_tls::Identity::from_pkcs12(&identity, password).unwrap();

    TlsAcceptor::from(native_tls::TlsAcceptor::new(identity).unwrap())
}

async fn wait_for_login(stream: &mut SplitStream<Stream>) -> bool {
    let msg = stream.try_next().await;

    if msg.is_err() {
        println!("Client disconnected");
        return false;
    }

    if let Some(msg) = msg.unwrap() {
        match msg {
            tungstenite::Message::Binary(bin) => {
                let cmd : ServerCmd = ServerCmd::from_bin(&bin)
                    .unwrap_or(ServerCmd::NoOp);

                match cmd {
                    ServerCmd::LoginAnonymous => {
                        return true;
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    }

    false
}

#[tokio::main]
async fn main() {
    env_logger::init();

    // Init server
    let game_data = GameData::load_from_path(PathBuf::from(".."));

    let mut server = Server::new();
    server.collect_data(&game_data);

    // Start the server with a maximum of 10 thread pools
    _ = server.start( Some(10) );

    let server = Arc::new(Mutex::new(server));

    // let mut timer : u128 = 0;
    // let mut game_tick_timer : u128 = 0;

    let uuid_endpoint : Arc<Mutex<UuidPeerMap>> = Arc::new(Mutex::new(FxHashMap::default()));

    tokio::spawn(handle_server_messages(server.clone(), uuid_endpoint.clone()));

    // Init network

    let tcp_listener = TcpListener::bind("0.0.0.0:3042").await.unwrap();

    while let Ok((stream, _)) = tcp_listener.accept().await {
        #[cfg(feature = "tls")]
        {
            let tls_acceptor = Arc::new(read_tls_acceptor(&PathBuf::from("keyStore.p12"), "eldiron"));

            let tls_stream = tls_acceptor.accept(stream).await.unwrap();

            tokio::spawn(handle_client_messages(
                tokio_tungstenite::accept_async(tls_stream).await.unwrap(),
                server.clone(),
                uuid_endpoint.clone()
            ));
        }

        #[cfg(not(feature = "tls"))]
        {
            tokio::spawn(handle_client_messages(
                tokio_tungstenite::accept_async(stream).await.unwrap(),
                server.clone(),
                uuid_endpoint.clone()
            ));
        }
    }
}
