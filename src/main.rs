const VERSION: &str = "0.0.0";

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

use std::{cell::RefCell, collections::VecDeque};

use chrono::Local;
use eo::{
    data::{EOByte, EOShort, Serializeable, StreamReader},
    protocol::{
        server::{
            init::{Init, InitData},
            welcome,
        },
        PacketAction, PacketFamily,
    },
};
use lazy_static::lazy_static;

pub type PacketBuf = Vec<EOByte>;

mod settings;
use futures_util::SinkExt;
use settings::Settings;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::broadcast,
};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};

// mod player;
// use player::Player;

mod bus;
use bus::Bus;

lazy_static! {
    static ref SETTINGS: Settings = Settings::new().expect("Failed to load settings!");
}

#[derive(Debug, Clone, serde_derive::Serialize)]
enum WSMessage {
    AddPlayer,
    RemovePlayer(u32),
    SetPlayerId(u32),
    Packet {
        player_id: u32,
        from: String,
        buf: Vec<u8>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "console")]
    console_subscriber::init();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();
    println!(
        "'||''''|   ..|''||   '||''|.
||  .    .|'    ||   ||   || ... ..    ...   ... ... .... ...
||''|    ||      ||  ||...|'  ||' '' .|  '|.  '|..'   '|.  |
||       '|.     ||  ||       ||     ||   ||   .|.     '|.|
.||.....|  ''|...|'  .||.     .||.     '|..|' .|  ||.    '|
                                                       .. |
                                                        ''      \nThe rusty endless online proxy: v{}\n",
        VERSION
    );

    let tcp_listener =
        TcpListener::bind(format!("{}:{}", SETTINGS.proxy.host, SETTINGS.proxy.port))
            .await
            .unwrap();

    let websocket_listener = TcpListener::bind("127.0.0.1:9001").await.unwrap();

    info!(
        "listening at {}:{}",
        SETTINGS.proxy.host, SETTINGS.proxy.port
    );

    let (tx, _) = broadcast::channel(32);

    let tx2 = tx.clone();
    tokio::spawn(async move {
        loop {
            let (client_socket, addr) = websocket_listener.accept().await.unwrap();
            info!("New websocket connection from {}", addr);

            let mut websocket = match accept_async(client_socket).await {
                Ok(ws) => ws,
                Err(e) => {
                    error!("Failed to accept websocket connection: {}", e);
                    continue;
                }
            };

            let mut rx = tx2.subscribe();

            tokio::spawn(async move {
                while let Ok(msg) = rx.recv().await {
                    let msg = serde_json::to_string(&msg).unwrap();
                    websocket.send(Message::text(msg)).await.unwrap();
                }
            });
        }
    });

    let tx = tx.clone();
    loop {
        let (client_socket, addr) = tcp_listener.accept().await.unwrap();
        info!("connection accepted ({})", addr);

        let tx = tx.clone();

        tokio::spawn(async move {
            let server_socket =
                TcpStream::connect(format!("{}:{}", SETTINGS.server.host, SETTINGS.server.port))
                    .await
                    .unwrap();

            let mut client_bus = Bus::new(client_socket, "Client".to_string());
            let mut server_bus = Bus::new(server_socket, "Server".to_string());
            let mut client_queue: RefCell<VecDeque<PacketBuf>> = RefCell::new(VecDeque::new());
            let mut server_queue: RefCell<VecDeque<PacketBuf>> = RefCell::new(VecDeque::new());
            let mut player_id: EOShort = 0;
            let mut character_name: Option<String> = None;

            let _timestamp = Local::now();

            tx.send(WSMessage::AddPlayer).unwrap();

            loop {
                tokio::select! {
                    result = client_bus.recv() => match result {
                        Some(Ok(packet)) => {
                            client_queue.get_mut().push_back(packet);
                        },
                        Some(Err(e)) => {
                            match e.kind() {
                                std::io::ErrorKind::BrokenPipe => {
                                    info!("Closed by peer");
                                    tx.send(WSMessage::RemovePlayer(player_id.into())).unwrap();
                                    break;
                                },
                                _ => {
                                    error!("Unknown error");
                                    tx.send(WSMessage::RemovePlayer(player_id.into())).unwrap();
                                    break;
                                }
                            }
                        },
                        None => {
                        }
                    },
                    result = server_bus.recv() => match result {
                        Some(Ok(packet)) => {
                            server_queue.get_mut().push_back(packet);
                        },
                        Some(Err(e)) => {
                            match e.kind() {
                                std::io::ErrorKind::BrokenPipe => {
                                    info!("Closed by peer");
                                    tx.send(WSMessage::RemovePlayer(player_id.into())).unwrap();
                                    break;
                                },
                                _ => {
                                    error!("Unknown error");
                                    tx.send(WSMessage::RemovePlayer(player_id.into())).unwrap();
                                    break;
                                }
                            }
                        },
                        None => {
                        }
                    },
                }

                if let Some(packet) = client_queue.get_mut().pop_front() {
                    tx.send(WSMessage::Packet {
                        player_id: player_id as u32,
                        from: "Client".to_string(),
                        buf: packet.clone(),
                    })
                    .unwrap();
                    let action = PacketAction::from_byte(packet[0]).unwrap();
                    let family = PacketFamily::from_byte(packet[1]).unwrap();

                    debug!(
                        "{}({}) From client: {:?}_{:?}\n{:?}\n",
                        character_name.as_ref().unwrap_or(&String::new()),
                        player_id,
                        family,
                        action,
                        packet
                    );

                    let reader = StreamReader::new(&packet[2..]);
                    let buf = reader.get_vec(reader.remaining());
                    reader.reset();
                    reader.seek(2);
                    server_bus.send(action, family, buf).await.unwrap();
                }

                if let Some(packet) = server_queue.get_mut().pop_front() {
                    tx.send(WSMessage::Packet {
                        player_id: player_id as u32,
                        from: "Server".to_string(),
                        buf: packet.clone(),
                    })
                    .unwrap();
                    let action = PacketAction::from_byte(packet[0]);
                    if let Some(action) = action {
                        let family = PacketFamily::from_byte(packet[1]).unwrap();

                        debug!(
                            "{}({}) From server: {:?}_{:?}\n{:?}\n",
                            character_name.as_ref().unwrap_or(&String::new()),
                            player_id,
                            family,
                            action,
                            packet
                        );

                        let reader = StreamReader::new(&packet[2..]);
                        let buf = reader.get_vec(reader.remaining());
                        reader.reset();

                        match family {
                            PacketFamily::Init => match action {
                                PacketAction::Init => {
                                    let mut reply = Init::new();
                                    reply.deserialize(&reader);
                                    debug!("{:?}", reply);

                                    match reply.data {
                                        InitData::Ok(reply_ok) => {
                                            player_id = reply_ok.player_id;

                                            tx.send(WSMessage::SetPlayerId(player_id.into()));

                                            server_bus.packet_processor.set_multiples(
                                                reply_ok.encode_multiple,
                                                reply_ok.decode_multiple,
                                            );
                                            client_bus.packet_processor.set_multiples(
                                                reply_ok.decode_multiple,
                                                reply_ok.encode_multiple,
                                            );
                                        }
                                        _ => {}
                                    }
                                }
                                _ => {}
                            },
                            PacketFamily::Welcome => match action {
                                PacketAction::Reply => {
                                    let mut reply = welcome::Reply::new();
                                    reply.deserialize(&reader);

                                    match reply.data {
                                        welcome::ReplyData::SelectCharacter(
                                            reply_select_character,
                                        ) => {
                                            character_name =
                                                Some(reply_select_character.name.to_string());
                                        }
                                        _ => {}
                                    }
                                }
                                _ => {}
                            },
                            _ => {}
                        }

                        client_bus.send(action, family, buf).await.unwrap();
                    } else {
                        client_bus.send_raw(packet).await.unwrap();
                    }
                }
            }
        });
    }

    Ok(())
}
