use std::net::SocketAddr;

use socket2::{Domain, Protocol, Socket, Type};
use tokio::net::{TcpListener, UdpSocket};

fn dual_stack(domain: Domain, sock_type: Type, proto: Protocol, addr: SocketAddr) -> Socket {
    let socket = Socket::new(domain, sock_type, Some(proto)).expect("create socket");
    if addr.is_ipv6() {
        let _ = socket.set_only_v6(false);
    }
    socket.set_reuse_address(true).expect("SO_REUSEADDR");
    socket.set_nonblocking(true).expect("nonblocking");
    socket.bind(&addr.into()).expect("bind");
    socket
}

pub async fn bind(addr: SocketAddr) -> TcpListener {
    let domain = if addr.is_ipv6() {
        Domain::IPV6
    } else {
        Domain::IPV4
    };
    let socket = dual_stack(domain, Type::STREAM, Protocol::TCP, addr);
    socket.listen(1024).expect("listen");
    TcpListener::from_std(socket.into()).expect("tokio TcpListener")
}

pub async fn bind_udp(addr: SocketAddr) -> UdpSocket {
    let domain = if addr.is_ipv6() {
        Domain::IPV6
    } else {
        Domain::IPV4
    };
    let socket = dual_stack(domain, Type::DGRAM, Protocol::UDP, addr);
    UdpSocket::from_std(socket.into()).expect("tokio UdpSocket")
}

pub fn ephemeral_udp_addr(target: &SocketAddr) -> SocketAddr {
    if target.is_ipv6() {
        "[::]:0".parse().unwrap()
    } else {
        "0.0.0.0:0".parse().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpStream;

    #[tokio::test]
    async fn dual_stack_accepts_ipv4() {
        let listener = bind("[::]:0".parse().unwrap()).await;
        let port = listener.local_addr().unwrap().port();
        let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
        assert!(
            TcpStream::connect(addr).await.is_ok(),
            "IPv4 must connect to dual-stack socket"
        );
    }

    #[tokio::test]
    async fn dual_stack_accepts_ipv6() {
        let listener = bind("[::]:0".parse().unwrap()).await;
        let port = listener.local_addr().unwrap().port();
        let addr: SocketAddr = format!("[::1]:{port}").parse().unwrap();
        assert!(
            TcpStream::connect(addr).await.is_ok(),
            "IPv6 must connect to dual-stack socket"
        );
    }

    #[tokio::test]
    async fn v4_only_binds() {
        let listener = bind("127.0.0.1:0".parse().unwrap()).await;
        let addr = listener.local_addr().unwrap();
        assert!(TcpStream::connect(addr).await.is_ok());
    }
}
