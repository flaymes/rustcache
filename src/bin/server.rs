use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use tokio::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 11211);
    let mut tcp_server = rustcache::memcached::server::TcpServer::new();
    tcp_server.run(addr).await
}
