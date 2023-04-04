use std::io;
use std::net::{SocketAddr, UdpSocket};

use rosc::encoder::encode;
use rosc::OscPacket;
use tracing::info;

pub struct OscUdpClient {
    pub socket: UdpSocket,
    pub to: SocketAddr,
}

impl OscUdpClient {
    pub fn new(from: SocketAddr, to: SocketAddr) -> Result<Self, io::Error> {
        let socket = UdpSocket::bind(from)?;
        socket.set_nonblocking(false)?;

        info!("new client: socket: {:?}; addr: {:?}", &socket, &to);
        Ok(Self { socket, to })
    }

    pub fn send(&self, packet: &OscPacket) -> io::Result<()> {
        let buf = encode(packet).unwrap();

        match self.socket.send_to(&buf, self.to) {
            Err(e) => Err(e),
            Ok(_) => Ok(()),
        }
    }
}
