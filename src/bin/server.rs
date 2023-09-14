#![feature(impl_trait_in_assoc_type)]

mod middleware;
use middleware::{CheckLayer, LogLayer};
use miniredis::CONFIG;
use miniredis::S;
use miniredis::cache::CACHE;
use core::panic;
use std::io::Read;
use std::net::SocketAddr;
use std::fs::{OpenOptions, File};

#[volo::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let addr: SocketAddr = CONFIG.get_my_addr().parse().unwrap();
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

    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    // start a new thread to subscribe to master
    let _ = tokio::spawn(async move {
        let addr: SocketAddr = CONFIG.get_master_addr().parse().unwrap();
        let master = volo_gen::volo::redis::RedisClientBuilder::new(CONFIG.get_name())
            .address(addr)
            .build();
        let myselfaddr = CONFIG.get_my_addr();

        let myself = volo_gen::volo::redis::RedisClientBuilder::new(CONFIG.get_name())
            .address(myselfaddr.parse().unwrap())
            .build();
        let mut offset = -1;
        loop {
            if rx.try_recv().is_ok() {
                break;
            }
            let mut request = volo_gen::volo::redis::SyncRequest::default();
            request.offset = offset;
            let response = master.dump_to(request).await.unwrap();
            offset = response.offset;
            let aof = response.aof;
            let mut request = volo_gen::volo::redis::SyncRequest::default();
            request.aof = aof;
            myself.load_from(request).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });

    volo_gen::volo::redis::RedisServer::new(S{
        file: std::sync::Mutex::new(file),
    })
        .layer_front(LogLayer)
        .layer_front(CheckLayer)
        .run(addr)
        .await
        .unwrap();
    let _ = tx.send(());
}

async fn init_cache(file: &mut File) {
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();
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