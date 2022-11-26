const VERSION: &str = "0.0.0";

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

use std::{cell::RefCell, collections::VecDeque};

use chrono::Local;
use eo::{
    data::{EOByte, EOShort, Serializeable, StreamReader, EOChar},
    net::{
        packets::{self, server::init::ReplyOk},
        replies::{InitReply, WelcomeReply},
        Action, Family, ItemMapInfo,
    }, character::PaperdollIcon, world::{Coords, TinyCoords},
};
use lazy_static::lazy_static;

pub type PacketBuf = Vec<EOByte>;

mod settings;
use num_traits::FromPrimitive;
use settings::Settings;
use tokio::net::{TcpListener, TcpStream};

// mod player;
// use player::Player;

mod bus;
use bus::Bus;

lazy_static! {
    static ref SETTINGS: Settings = Settings::new().expect("Failed to load settings!");
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
    info!(
        "listening at {}:{}",
        SETTINGS.proxy.host, SETTINGS.proxy.port
    );

    loop {
        let (client_socket, addr) = tcp_listener.accept().await.unwrap();
        info!("connection accepted ({})", addr,);

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

            let mut timestamp = Local::now();

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
                                    break;
                                },
                                _ => {
                                    error!("Unknown error");
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
                                    break;
                                },
                                _ => {
                                    error!("Unknown error");
                                    break;
                                }
                            }
                        },
                        None => {
                        }
                    },
                }

                if let Some(packet) = client_queue.get_mut().pop_front() {
                    let action = Action::from_u8(packet[0]).unwrap();
                    let family = Family::from_u8(packet[1]).unwrap();

                    debug!("{}({}) From client: {:?}_{:?}\n{:?}\n", character_name.as_ref().unwrap_or(&String::new()), player_id, family, action, packet);

                    let reader = StreamReader::new(&packet[2..]);
                    let buf = reader.get_vec(reader.remaining());
                    reader.reset();
                    reader.seek(2);

                    // match family {
                    //     Family::Init => match action {
                    //         Action::Init => {
                    //             let mut request = packets::client::init::Request::new();
                    //             request.deserialize(&reader);
                    //             debug!("{:?}", request);
                    //         }
                    //         _ => {}
                    //     },
                    //     Family::NpcMapInfo => match action {
                    //         Action::Request => {
                    //             let mut request = packets::client::npc_map_info::Request::new();
                    //             request.deserialize(&reader);
                    //             debug!("Requesting NPC Info\n{:?}", request);
                    //         }
                    //         _ => {}
                    //     },
                    //     _ => {}
                    // }

                    server_bus.send(action, family, buf).await.unwrap();
                }

                if let Some(packet) = server_queue.get_mut().pop_front() {
                    let action = Action::from_u8(packet[0]);

                    if let Some(action) = action {
                        let family = Family::from_u8(packet[1]).unwrap();

                        debug!("{}({}) From server: {:?}_{:?}\n{:?}\n", character_name.as_ref().unwrap_or(&String::new()), player_id, family, action, packet);

                        let reader = StreamReader::new(&packet[2..]);
                        let mut buf = reader.get_vec(reader.remaining());
                        reader.reset();

                        match family {
                            Family::Init => match action {
                                Action::Init => {
                                    let mut reply = packets::server::init::Reply::new();
                                    reply.deserialize(&reader);
                                    debug!("{:?}", reply);

                                    match reply.reply_code {
                                        InitReply::OK => {
                                            let reply_buf = reply.reply.serialize();
                                            let mut reply_ok = packets::server::init::ReplyOk::new();
                                            let ok_reader = StreamReader::new(&reply_buf);
                                            reply_ok.deserialize(&ok_reader);

                                            player_id = reply_ok.player_id;
                                            server_bus.packet_processor.set_multiples(
                                                reply_ok.encoding_multiples[0],
                                                reply_ok.encoding_multiples[1],
                                            );
                                            client_bus.packet_processor.set_multiples(
                                                reply_ok.encoding_multiples[1],
                                                reply_ok.encoding_multiples[0],
                                            );
                                        }
                                        _ => {}
                                    }
                                }
                                Action::Request => {
                                    // blah
                                }
                                _ => {}
                            },
                            Family::MapInfo => match action {
                                Action::Reply => {
                                    let mut reply = packets::server::map_info::Reply::new();
                                    reply.deserialize(&reader);
                                    reply.nearby.items.push(ItemMapInfo {
                                        uid: 0, id: 1, coords: TinyCoords { x: 11, y: 8}, amount: 10000, 
                                    });

                                    debug!("Injecting gold! {:?}", reply);
                                    buf = reply.serialize();
                                }
                                _ => {}
                            }
                            Family::Welcome => match action {
                                Action::Reply => {
                                    let mut reply = packets::server::welcome::Reply::new();
                                    reply.deserialize(&reader);

                                    if let Some(select_character_reply) = reply.select_character {
                                        character_name = Some(select_character_reply.name.to_string());
                                    }
                                }
                                _ => {}
                            }
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
