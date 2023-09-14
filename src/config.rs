use clap::Parser;
use serde_derive::Deserialize;
use std::{cell::RefCell, sync::Mutex};
/// a simple tcp proxy service
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct ProxyArgs {
    /// proxy.toml file
    #[clap(short, long)]
    pub config: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ClusterProxyConfig {
    pub proxy: Option<Vec<ProxyConfig>>,
}

#[derive(Deserialize, Debug)]
pub struct ProxyConfig {
    //for debug
    pub enable: Option<bool>,
    pub name: String,
    pub listen: u16,
    pub target: String,
}
pub struct Cluster {
    pub slots: RefCell<Slots>,
    pub nodes: RefCell<String>,
    pub proxy: ProxyConfig,
    pub read_from_slave: bool,
}
impl Cluster {
    pub fn new(proxy_config: ProxyConfig) -> Self {
        Cluster {
            slots: RefCell::new(Slots::new()),
            nodes: RefCell::new(String::new()),
            proxy: proxy_config,
            read_from_slave: false,
        }
    }
    pub fn get_addr(&self) -> String {
        if self.read_from_slave {
            self.slots.borrow().get_slave()
        } else {
            self.slots.borrow().get_master()
        }
    }
    //dispatch the key using crc16 mod 16384
    pub fn dispatch(&self, key: &str) -> String {
        const X25: crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_IBM_SDLC);
        let mut hasher = X25.digest();
        hasher.update(key.as_bytes());
        let hash = hasher.finalize();
        let slot = hash as usize % 16384;
        let slots = self.slots.borrow_mut();
        if slots.masters.len() == 0 {
            return String::new();
        }
        if slots.masters.len() == 1 {
            return slots.masters[0].clone();
        }
        let mut i = 0;
        while i < slots.masters.len() - 1 {
            if slot >= i && slot < i + 1 {
                return slots.masters[i].clone();
            }
            i += 1;
        }
        slots.masters[slots.masters.len() - 1].clone()
    }
}
pub struct Slots {
    pub masters: Vec<String>,
    pub slaves: Vec<String>,
}
impl Slots {
    pub fn new() -> Self {
        Slots {
            masters: Vec::new(),
            slaves: Vec::new(),
        }
    }
    pub fn add_master(&mut self, addr: String) {
        self.masters.push(addr);
    }
    pub fn add_slave(&mut self, addr: String) {
        self.slaves.push(addr);
    }
    pub fn get_master(&self) -> String {
        self.masters[0].clone()
    }
    pub fn get_slave(&self) -> String {
        self.slaves[0].clone()
    }
}
