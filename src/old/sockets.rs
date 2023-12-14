use std::net::{UdpSocket};

pub struct Sockets {
    pub socket: UdpSocket,
}

impl Sockets {
    pub fn new(nonblockin: bool) -> Self {
        let socket = UdpSocket::bind("0.0.0.0:0").expect("successful main socket bind");
        socket.set_nonblocking(nonblockin).expect("Successful non blocking main socket");
        Sockets {
            socket,
        }
    }
    pub fn new2(nonblockin: bool) -> Self {

        let socket = UdpSocket::bind("0.0.0.0:0").expect("successful main socket bind");
        socket.set_nonblocking(nonblockin).expect("Successful non blocking main socket");
        Sockets {
            socket,
        }
    }
 
    pub fn new3(nonblockin: bool) -> Self {
        let socket = UdpSocket::bind("0.0.0.0:0").expect("successful main socket bind");
        socket.set_nonblocking(nonblockin).expect("Successful non blocking main socket");
        Sockets {
            socket,
        }
    }   

}
