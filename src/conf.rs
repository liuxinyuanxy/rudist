#![allow(dead_code)]
#![allow(unused_variables)]

use std::sync::Mutex;

struct Inner {
    name: String,
    is_master: bool,
    my_addr: String,
    slaves: Option<Vec<String>>,
}

pub struct Config {
    inner: Mutex<Inner>,
}

lazy_static::lazy_static! {
    pub static ref CONFIG: Config = Config::new();
}

fn load_config(name: &str) -> Inner {
    let settings = config::Config::builder()
        .add_source(config::File::with_name("src/redis.toml"))
        .build()
        .unwrap();
    let names: Vec<String> = settings.get("names").unwrap();
    let addrs: Vec<String> = settings.get("addrs").unwrap();
    let master_name: String = settings.get("master").unwrap();

    if master_name == name {
        Inner {
            name: name.to_string(),
            is_master: true,
            my_addr: addrs[names.iter().position(|x| x == name).unwrap()].clone(),
            slaves: Some(
                names
                    .iter()
                    .filter(|x| *x != name)
                    .map(|x| addrs[names.iter().position(|y| y == x).unwrap()].clone())
                    .collect(),
            ),
            // master_addr: Some(addrs[names.iter().position(|x| x == name).unwrap()].clone()),
            // slave_addr_myself: None,
        }
    } else {
        Inner {
            name: name.to_string(),
            is_master: false,
            my_addr: addrs[names.iter().position(|x| x == name).unwrap()].clone(),
            slaves: None,
            // master_addr: Some(addrs[names.iter().position(|x| x == &master_name).unwrap()].clone()),
            // slave_addr_myself: Some(addrs[names.iter().position(|x| x == name).unwrap()].clone()),
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
        inner.my_addr.clone()
    }

    pub fn get_slave_addrs(&self) -> Option<Vec<String>> {
        let inner = self.inner.lock().unwrap();
        inner.slaves.clone()
    }
}
