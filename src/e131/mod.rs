use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use eyre::Result;

use sacn_unofficial::source::SacnSource;
use sacn_unofficial::packet::ACN_SDT_MULTICAST_PORT;

pub const E131_PORT: u16 = ACN_SDT_MULTICAST_PORT;

pub struct E131 {
    src: SacnSource,
    dst: SocketAddr,
    universe: u16,
}

impl E131 {
    pub fn new(dest: IpAddr, port: u16, universe: u16) -> Result<Self> {
        let src = SocketAddr::new(IpAddr::from_str("0.0.0.0").unwrap(), 0);
        let dst = SocketAddr::new(dest, port);

        let mut src = SacnSource::with_ip("", src).unwrap();
        src.register_universe(universe).expect("failed to register E131 universe");

        Ok(Self {
            src,
            dst,
            universe,
        })
    }

    pub fn send(&mut self, data: &[u8]) {
        if let Err(e) = self.src.send(&[self.universe], data, None, Some(self.dst), None) {
            log::error!("{e}");
        }
    }
}
