use crate::memcached::{handler, storage, timer};
use crate::protocol::{binary, binary_codec};
use futures_util::{SinkExt, StreamExt};
use std::net::ToSocketAddrs;
use std::sync::{Arc, RwLock};
use tokio::io;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs as TokioToSocketAddrs};
use tokio_util::codec::{FramedRead, FramedWrite};

pub struct TcpServer {
    timer: Arc<dyn timer::Timer + Send + Sync>,
    storage: Arc<storage::Storage>,
}

impl Default for TcpServer {
    fn default() -> Self {
        let timer: Arc<dyn timer::Timer + Send + Sync> = Arc::new(timer::SystemTimer::new());
        TcpServer {
            timer: timer.clone(),
            storage: Arc::new(storage::Storage::new(timer.clone())),
        }
    }
}

impl TcpServer {
    pub fn new() -> TcpServer {
        Default::default()
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
                        let mut reader =
                            FramedRead::new(rx, binary_codec::MemcachedBinaryCodec::new());
                        let mut writer =
                            FramedWrite::new(tx, binary_codec::MemcachedBinaryCodec::new());

                        while let Some(result) = reader.next().await {
                            match result {
                                Ok(request) => {
                                    let response = handler.handle_request(request);
                                    if let Some(response) = response {
                                        if let Err(e) = writer.send(response).await {
                                            println!("error on sending response; error = {:?}", e);
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
