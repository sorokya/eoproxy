use chrono::{DateTime, Local};
use eo::{
    data::{encode_number, EOByte, StreamBuilder},
    net::PacketProcessor,
    protocol::{PacketAction, PacketFamily},
};
use tokio::net::TcpStream;

const PACKET_HEADER_SIZE: usize = 2;
const PACKET_LENGTH_SIZE: usize = 2;

use crate::PacketBuf;

pub struct Bus {
    socket: TcpStream,
    pub packet_processor: PacketProcessor,
    timestamp: DateTime<Local>,
    name: String,
    packet_length: usize,
    amount_read: usize,
    buf: Option<Vec<u8>>,
}

impl Bus {
    pub fn new(socket: TcpStream, name: String) -> Self {
        Self {
            socket,
            packet_processor: PacketProcessor::new(),
            timestamp: Local::now(),
            name,
            packet_length: 0,
            amount_read: 0,
            buf: None,
        }
    }

    pub async fn send_raw(&mut self, mut data: PacketBuf) -> std::io::Result<()> {
        let packet_size = data.len();

        self.timestamp = Local::now();
        trace!(
            "{} Send: [{}] {:?}",
            self.name,
            self.timestamp.format("%M:%S.%f"),
            data
        );

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
        action: PacketAction,
        family: PacketFamily,
        mut data: PacketBuf,
    ) -> std::io::Result<()> {
        let packet_size = PACKET_HEADER_SIZE + data.len();
        let mut builder = StreamBuilder::with_capacity(PACKET_LENGTH_SIZE + packet_size);

        builder.add_byte(action.to_byte());
        builder.add_byte(family.to_byte());
        builder.append(&mut data);

        let mut buf = builder.get();
        self.timestamp = Local::now();
        debug!(
            "{} Send: [{}] {:?}",
            self.name,
            self.timestamp.format("%M:%S.%f"),
            buf
        );
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
                    self.packet_length = packet_length;
                    match self.read(packet_length).await {
                        Some(Ok(_)) => {
                            let mut data_buf = self.buf.as_ref().unwrap().clone();

                            debug!("{} Receive Raw: [{}] {:?}", self.name, self.timestamp.format("%M:%S.%f"), data_buf);
                            self.packet_processor.decode(&mut data_buf);

                            self.timestamp = Local::now();
                            debug!(
                                "{} Receive: [{}] {:?}",
                                self.name,
                                self.timestamp.format("%M:%S.%f"),
                                data_buf
                            );
                            self.buf = None;
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
        if self.packet_length > 0 {
            return Some(Ok(self.packet_length));
        }

        match self.read(2).await {
            Some(Ok(_)) => {
                let buf = self.buf.as_ref().unwrap().clone();
                self.buf = None;
                Some(Ok(eo::data::decode_number(&buf) as usize))
            }
            Some(Err(e)) => Some(Err(e)),
            None => None,
        }
    }

    async fn read(&mut self, length: usize) -> Option<std::io::Result<()>> {
        if self.buf.is_none() {
            self.buf = Some(vec![0; length]);
        }

        self.socket.readable().await.unwrap();
        if let Some(buf) = self.buf.as_mut() {
            let mut recv_buf = vec![0; length - self.amount_read];

            match self.socket.try_read(&mut recv_buf) {
                Ok(0) => {
                    return Some(Err(std::io::Error::new(
                        std::io::ErrorKind::BrokenPipe,
                        "Connection closed",
                    )));
                }
                Ok(bytes_read) => {
                    buf.splice(self.amount_read..self.amount_read + bytes_read, recv_buf);

                    self.amount_read += bytes_read;

                    if self.amount_read == length {
                        self.amount_read = 0;
                        self.packet_length = 0;
                        return Some(Ok(()));
                    }
                }
                Err(_) => {
                    return None;
                }
            }
        }

        None
    }
}
