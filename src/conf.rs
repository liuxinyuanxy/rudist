#![allow(dead_code)]
#![allow(unused_variables)]

use std::sync::Mutex;

struct Inner {
    name: String,
    is_master: bool,
    master_addr: Option<String>,
    slave_addr_myself: Option<String>,
}

pub struct Config {
    inner: Mutex<Inner>,
}

lazy_static::lazy_static! {
    pub static ref CONFIG: Config = Config::new();
}

fn load_config(name: &str) -> Inner {
    let settings = config::Config::builder()
        .add_source(config::File::with_name("src/redis.conf"))
        .build()
        .unwrap();
    let names: Vec<String> = settings.get("names").unwrap();
    let addrs: Vec<String> = settings.get("addrs").unwrap();
    let master_name: String = settings.get("master").unwrap();

    if master_name == name {
        Inner {
            name: name.to_string(),
            is_master: true,
            master_addr: Some(addrs[names.iter().position(|x| x == name).unwrap()].clone()),
            slave_addr_myself: None,
        }
    } else {
        Inner {
            name: name.to_string(),
            is_master: false,
            master_addr: Some(addrs[names.iter().position(|x| x == &master_name).unwrap()].clone()),
            slave_addr_myself: Some(addrs[names.iter().position(|x| x == name).unwrap()].clone()),
        }
    }
}

impl Config {
    fn new() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let name = args[1].clone();
        Config {
            inner: Mutex::new(load_config(name.as_str())),
        }
    }

    pub fn get_name(&self) -> String {
        let inner = self.inner.lock().unwrap();
        inner.name.clone()
    }

    pub fn is_master(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.is_master
    }

    pub fn get_my_addr(&self) -> String {
        let inner = self.inner.lock().unwrap();
        match inner.is_master {
            true => inner.master_addr.clone().unwrap(),
            false => inner.slave_addr_myself.clone().unwrap(),
        }
    }

    pub fn get_master_addr(&self) -> String {
        let inner = self.inner.lock().unwrap();
        inner.master_addr.clone().unwrap()
    }
}
