#![allow(dead_code)]
#![allow(unused_variables)]

use std::sync::Mutex;
use tokio::io::AsyncWriteExt;
struct Inner {
    name: String,
    is_master: bool,
    my_addr: String,
    slaves: Option<Vec<String>>,
}

pub struct Config {
    inner: Mutex<Inner>,
}

pub struct File {
    file: tokio::sync::Mutex<Option<tokio::fs::File>>,
}

impl File {
    fn new() -> Self {
        Self {
            file: tokio::sync::Mutex::new(None),
        }
    }

    pub async fn set_file(&self, filename: String) {
        let file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(filename)
            .await
            .unwrap();
        let mut inner = self.file.lock().await;
        *inner = Some(file);
    }

    pub async fn write_to_file_new_thread(
        &self,
        buf: String,
        sender: Option<tokio::sync::mpsc::Sender<()>>,
    ) {
        let inner = self.file.lock().await;
        let file = inner.as_ref();
        if let Some(file) = file.as_ref() {
            let mut file = file.try_clone().await.unwrap();
            let _ = tokio::spawn(async move {
                file.write_all(buf.as_bytes()).await.unwrap();
                drop(sender)
            });
        }
    }
}

lazy_static::lazy_static! {
    pub static ref CONFIG: Config = Config::new();
    pub static ref FILE: File = File::new();
}

fn load_config(name: &str) -> Inner {
    let settings = config::Config::builder()
        .add_source(config::File::with_name("src/redis.toml"))
        .build()
        .unwrap();
    let names: Vec<String> = settings.get("names").unwrap();
    let addrs: Vec<String> = settings.get("addrs").unwrap();
    let master_name: String = settings.get("master").unwrap();
    let master_slave_on = false;
    // let master_slave_on: bool = settings.get("master_slave_on").unwrap();
    let cluster_on: bool = settings.get("cluster_on").unwrap();
    let file_name = "log/aof.log".to_string();
    if master_slave_on {
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
    } else {
        Inner {
            name: name.to_string(),
            is_master: true,
            my_addr: addrs[names.iter().position(|x| x == name).unwrap()].clone(),
            slaves: None,
            // master_addr: None,
            // slave_addr_myself: None,
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
