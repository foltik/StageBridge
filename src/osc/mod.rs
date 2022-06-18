use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use tokio::net::UdpSocket;

use futures::Future;
use tokio::sync::broadcast;
use tokio::task;

use rosc::{OscMessage, OscPacket};

pub use rosc::OscType as Type;
#[derive(Clone, Debug)]
pub struct Message {
    pub addr: String,
    pub args: Vec<Type>,
}

#[derive(Clone)]
pub struct Osc {
    tx: broadcast::Sender<Message>,
}

impl Osc {
    pub async fn new(port: u16) -> Self {
        let sock = UdpSocket::bind(SocketAddr::new(IpAddr::from_str("0.0.0.0").unwrap(), port))
            .await
            .expect("Failed to bind OSC port");

        let (tx, _) = broadcast::channel(64);

        let _tx = tx.clone();
        task::spawn(async move {
            let mut buf = [0u8; rosc::decoder::MTU];
            loop {
                let (size, addr) = sock.recv_from(&mut buf).await.unwrap();
                log::trace!("{} bytes from {:?}", size, addr);
                let packet = rosc::decoder::decode(&buf[..size]).unwrap();
                match packet {
                    OscPacket::Message(OscMessage { addr, args }) => {
                        log::trace!("{}: {:?}", addr, args);
                        let msg = Message { addr, args };
                        match _tx.send(msg.clone()) {
                            Err(_) => log::warn!("Dropped message {:?}", msg),
                            _ => {}
                        }
                    }
                    _ => log::debug!("Unknown packet {:?}", packet),
                }
            }
        });

        Self {
            tx
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Message> {
        self.tx.subscribe()
    }

    pub fn spawn<F, Fut>(&self, f: F)
    where
        F: FnOnce(Osc) -> Fut + Send + Sync,
        Fut: Future<Output = ()> + Send + 'static,
    {
        task::spawn(f((*self).clone()));
    }
}
