use chrono::{DateTime, Utc, Local};
use eo::{net::{PacketProcessor, Action, Family, PACKET_HEADER_SIZE, PACKET_LENGTH_SIZE}, data::{StreamBuilder, EOByte, encode_number}};
use tokio::net::TcpStream;

use crate::PacketBuf;

pub struct Bus {
    socket: TcpStream,
    pub packet_processor: PacketProcessor,
    timestamp: DateTime<Local>,
    name: String,
}

impl Bus {
    pub fn new(socket: TcpStream, name: String) -> Self {
        Self {
            socket,
            packet_processor: PacketProcessor::new(),
            timestamp: Local::now(),
            name,
        }
    }

    pub async fn send_raw(
        &mut self,
        mut data: PacketBuf,
    ) -> std::io::Result<()> {
        let packet_size = data.len();

        self.timestamp = Local::now();
        trace!("{} Send: [{}] {:?}", self.name, self.timestamp.format("%M:%S.%f"), data);

        let length_bytes = encode_number(packet_size as u32);
        data.insert(0, length_bytes[1]);
        data.insert(0, length_bytes[0]);

        match self.socket.try_write(&data) {
            Ok(num_of_bytes_written) => {
                if num_of_bytes_written != packet_size + PACKET_LENGTH_SIZE {
                    error!(
                        "Written bytes ({}) doesn't match packet size ({})",
                        num_of_bytes_written, packet_size
                    );
                }
            }
            _ => {
                error!("Error writing to socket");
            }
        }

        Ok(())
    }

    pub async fn send(
        &mut self,
        action: Action,
        family: Family,
        mut data: PacketBuf,
    ) -> std::io::Result<()> {
        let packet_size = PACKET_HEADER_SIZE + data.len();
        let mut builder = StreamBuilder::with_capacity(PACKET_LENGTH_SIZE + packet_size);

        builder.add_byte(action as EOByte);
        builder.add_byte(family as EOByte);
        builder.append(&mut data);


        let mut buf = builder.get();
        self.timestamp = Local::now();
        trace!("{} Send: [{}] {:?}", self.name, self.timestamp.format("%M:%S.%f"), buf);
        self.packet_processor.encode(&mut buf);

        let length_bytes = encode_number(packet_size as u32);
        buf.insert(0, length_bytes[1]);
        buf.insert(0, length_bytes[0]);

        match self.socket.try_write(&buf) {
            Ok(num_of_bytes_written) => {
                if num_of_bytes_written != packet_size + PACKET_LENGTH_SIZE {
                    error!(
                        "Written bytes ({}) doesn't match packet size ({})",
                        num_of_bytes_written, packet_size
                    );
                }
            }
            _ => {
                error!("Error writing to socket");
            }
        }

        Ok(())
    }

    pub async fn recv(&mut self) -> Option<std::io::Result<PacketBuf>> {
        match self.get_packet_length().await {
            Some(Ok(packet_length)) => {
                if packet_length > 0 {
                    match self.read(packet_length).await {
                        Some(Ok(buf)) => {
                            let mut data_buf = buf;
                            self.packet_processor.decode(&mut data_buf);

                            self.timestamp = Local::now();
                            trace!("{} Receive: [{}] {:?}", self.name, self.timestamp.format("%M:%S.%f"), data_buf);
                            Some(Ok(data_buf))
                        }
                        Some(Err(e)) => Some(Err(e)),
                        None => None,
                    }
                } else {
                    None
                }
            }
            Some(Err(e)) => Some(Err(e)),
            None => None,
        }
    }

    async fn get_packet_length(&mut self) -> Option<std::io::Result<usize>> {
        match self.read(2).await {
            Some(Ok(buf)) => Some(Ok(eo::data::decode_number(&buf) as usize)),
            Some(Err(e)) => Some(Err(e)),
            None => None,
        }
    }

    async fn read(&mut self, length: usize) -> Option<std::io::Result<Vec<EOByte>>> {
        let mut buf: Vec<EOByte> = vec![0; length];
        self.socket.readable().await.unwrap();
        match self.socket.try_read(&mut buf) {
            Ok(0) => {
                return Some(Err(std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    "Connection closed",
                )));
            }
            Ok(_) => {}
            Err(_) => {
                return None;
            }
        }
        Some(Ok(buf))
    }
}