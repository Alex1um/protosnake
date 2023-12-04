use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};


pub fn create_sockets() -> (UdpSocket, UdpSocket) {
    let multicast_receiver = UdpSocket::bind("0.0.0.0").expect("successful bind");
    multicast_receiver
        .join_multicast_v4(&Ipv4Addr::new(239, 192, 0, 4), &Ipv4Addr::new(0, 0, 0, 0))
        .expect("Successful join");

    let sender = UdpSocket::bind("0.0.0.0").expect("successful sender bind");
    return (sender, multicast_receiver)
}