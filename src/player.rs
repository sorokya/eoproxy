use std::{collections::VecDeque, cell::RefCell};

use eo::{data::EOShort, net::PacketProcessor};
use tokio::{net::TcpStream, sync::mpsc::{self, UnboundedReceiver}};

use crate::PacketBuf;

#[derive(Debug)]
pub enum Command {
    Close(String),
}

pub struct Player {
    pub id: Option<EOShort>,
    pub rx: UnboundedReceiver<Command>,
    pub packet_processor: PacketProcessor,
    pub bus: PacketBus,
    pub queue: RefCell<VecDeque<PacketBuf>>,
    pub busy: bool,
}

impl Player {
    pub fn new(socket: TcpStream, rx: UnboundedReceiver<Command>) -> Self {
        Self {
            id: None,
            rx,
            packet_processor: PacketProcessor::new(),
            bus: PacketBus::new(),
            queue: RefCell::new(VecDeque::new()),
            busy: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlayerHandle {
    tx: mpsc::UnboundedSender<Command>,
}

impl PlayerHandle {
    pub fn new(socket: TcpStream) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let player = Player::new(socket, rx);
        tokio::task::Builder::new()
            .name(&format!("Player"))
            .spawn(run_player(player, PlayerHandle::for_tx(tx.clone())));

        Self { tx }
    }
}

async fn run_player(mut player: Player, player_handle: PlayerHandle) {
    loop {
        tokio::select! {
            result = player.bus.recv() => match result {
                Some(Ok(packet)) => {
                    trace!("Recv: {:?}", packet);
                    player.queue.get_mut().push_back(packet);
                },
                Some(Err(e)) => {
                    match e.kind() {
                        std::io::ErrorKind::BrokenPipe => {
                            player_handle.close("Closed by peer".to_string());
                        },
                        _ => {
                            player_handle.close(format!("Due to unknown error: {:?}", e));
                        }
                    }
                },
                None => {
                }
            },
            Some(command) = player.rx.recv() => {
                // TODO: really don't like how this reads.. maybe a better way to do this?
                if !player.handle_command(command).await {
                    break;
                }
            }
        }

        if player.busy {
            continue;
        }

        if let Some(packet) = player.queue.get_mut().pop_front() {
            tokio::task::Builder::new()
                .name("handle_packet")
                .spawn(handle_packet(
                    packet,
                    player_handle.clone(),
                    player.world.clone(),
                ));
        }
    }
}
