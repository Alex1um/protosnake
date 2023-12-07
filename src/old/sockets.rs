use std::net::{Ipv4Addr, UdpSocket};

pub struct Sockets {
    pub multicast_receiver: UdpSocket,
    pub socket: UdpSocket,
}

impl Sockets {
    pub fn new(nonblockin: bool) -> Self {
        let multicast_receiver = UdpSocket::bind("0.0.0.0:48667").expect("successful bind");
        multicast_receiver
            .join_multicast_v4(&Ipv4Addr::new(239, 192, 0, 4), &Ipv4Addr::new(0, 0, 0, 0))
            .expect("Successful join");
        multicast_receiver.set_nonblocking(true).expect("Successful non block");

        let socket = UdpSocket::bind("0.0.0.0:48666").expect("successful main socket bind");
        socket.set_nonblocking(nonblockin).expect("Successful non blocking main socket");
        Sockets {
            multicast_receiver,
            socket,
        }
    }
    pub fn new2(nonblockin: bool) -> Self {
        let multicast_receiver = UdpSocket::bind("0.0.0.0:48669").expect("successful bind");
        multicast_receiver
            .join_multicast_v4(&Ipv4Addr::new(239, 192, 0, 4), &Ipv4Addr::new(0, 0, 0, 0))
            .expect("Successful join");
        multicast_receiver.set_nonblocking(true).expect("Successful non block");

        let socket = UdpSocket::bind("0.0.0.0:48668").expect("successful main socket bind");
        socket.set_nonblocking(nonblockin).expect("Successful non blocking main socket");
        Sockets {
            multicast_receiver,
            socket,
        }
    }
 
    pub fn new3(nonblockin: bool) -> Self {
        let multicast_receiver = UdpSocket::bind("0.0.0.0:48670").expect("successful bind");
        multicast_receiver
            .join_multicast_v4(&Ipv4Addr::new(239, 192, 0, 4), &Ipv4Addr::new(0, 0, 0, 0))
            .expect("Successful join");
        multicast_receiver.set_nonblocking(true).expect("Successful non block");

        let socket = UdpSocket::bind("0.0.0.0:48671").expect("successful main socket bind");
        socket.set_nonblocking(nonblockin).expect("Successful non blocking main socket");
        Sockets {
            multicast_receiver,
            socket,
        }
    }   

}
