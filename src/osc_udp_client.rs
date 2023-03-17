use std::io;
use std::net::{SocketAddr, UdpSocket};

use rosc::encoder::encode;
use rosc::OscPacket;

pub struct OscUdpClient {
    pub socket: UdpSocket,
    pub addr: SocketAddr,
}

impl OscUdpClient {
    pub fn new(addr: SocketAddr) -> Result<Self, io::Error> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_nonblocking(false)?;

        Ok(Self { socket, addr })
    }

    pub fn send(&self, packet: &OscPacket) -> io::Result<()> {
        let buf = encode(packet).unwrap();

        match self.socket.send_to(&buf, self.addr) {
            Err(e) => Err(e),
            Ok(_) => Ok(()),
        }
    }
}
