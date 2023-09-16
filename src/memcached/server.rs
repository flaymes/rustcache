use std::net::ToSocketAddrs;
use std::sync::Arc;
use futures_util::{SinkExt, StreamExt};
use tokio::io;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs as TokioToSocketAddrs};
use tokio_util::codec::{FramedRead, FramedWrite};
use crate::memcached::{handler, storage};
use crate::protocol::{binary, binary_codec};

pub struct TcpServer {
    storage: Arc<storage::Storage>,
}

impl TcpServer {
    pub fn new() -> TcpServer {
        TcpServer {
            storage: Arc::new(storage::Storage::new())
        }
    }

    pub async fn run<A: ToSocketAddrs + TokioToSocketAddrs>(&mut self, addr: A) -> io::Result<()> {
        let mut listener = TcpListener::bind(addr).await?;
        loop {
            match listener.accept().await {
                Ok((mut socket, peer_addr)) => {
                    let db = self.storage.clone();
                    println!("Incoming connection: {}", peer_addr);

                    tokio::spawn(async move {
                        let mut handler = handler::BinaryHandler::new(db);

                        let (rx, tx) = socket.split();
                        let mut reader
                            = FramedRead::new(rx, binary_codec::MemcachedBinaryCodec::new());
                        let mut writer =
                            FramedWrite::new(tx, binary_codec::MemcachedBinaryCodec::new());

                        while let Some(result) = reader.next().await {
                            match result {
                                Ok(request) => {
                                    let response = handler.handle_request(request);
                                    match response {
                                        None => {}
                                        Some(response) => {
                                            if let Err(e) = writer.send(response).await {
                                                println!("error on sending response; error = {:?}", e);
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("error on decoding from socket; error = {:?}", e);
                                }
                            }
                        }
                    });
                }
                Err(e) => {}
            }
        }
    }
}
