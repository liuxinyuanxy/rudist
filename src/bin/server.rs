#![feature(impl_trait_in_assoc_type)]

mod middleware;
use middleware::{CheckLayer, LogLayer};
use miniredis::S;
use miniredis::cache::CACHE;
use core::panic;
use std::io::Read;
use std::net::SocketAddr;
use std::fs::{OpenOptions, File};
#[volo::main]
async fn main() {
    // set the log mod as debug
    tracing_subscriber::fmt::init();

    let addr: SocketAddr = "[::]:19260".parse().unwrap();
    let addr = volo::net::Address::from(addr);

    if !std::path::Path::new("log").exists() {
        let _ = std::fs::create_dir("log").unwrap();
    }

    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .append(true)
        .open("log/aof.log")
        .unwrap();
    init_cache(&mut file).await;

    volo_gen::volo::redis::RedisServer::new(S{
        file: std::sync::Mutex::new(file),
    })
        .layer_front(LogLayer)
        .layer_front(CheckLayer)
        .run(addr)
        .await
        .unwrap();
}

async fn init_cache(file: &mut File) {
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap(); // *3\n$3\nset\n$3\nkey\n$5\nvalue\n
    let mut lines = buf.split("\n");
    while let Some(line) = lines.next() {
        if line.starts_with("*") {
            let mut line = line.chars();
            line.next();
            let len = line.as_str().parse::<usize>().unwrap();
            let mut args = Vec::new();
            for _ in 0..len {
                lines.next();
                let arg = lines.next().unwrap();
                args.push(arg);
            }
            let cmd = args[0];
            let key = args[1];
            match cmd {
                "set" => {
                    let value = args[2];
                    let mut ttl = None;
                    if len == 4 {
                        let expire_at = args[3].parse::<u128>().unwrap();
                        let now = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_millis();
                        if expire_at <= now {
                            continue;
                        }
                        let expire_seconds = (args[3].parse::<u128>().unwrap() - now) / 1000;
                        ttl = Some(expire_seconds as i32);
                    }
                    CACHE.insert(key.to_string(), value.to_string(), ttl).await;
                }
                "del" => {
                    CACHE.del(key).await;
                }
                _ => {
                    panic!("unknown command");
                }
            }
        }
    }
}