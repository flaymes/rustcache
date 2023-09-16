use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
#[tokio::main]
async fn main() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 6142);
    let mut stream = TcpStream::connect(addr).await.unwrap();
    println!("created stream");

    let result = stream.write(b"hello world\n").await;
    println!("wrote to stream; success={:?}", result.is_ok());
}
