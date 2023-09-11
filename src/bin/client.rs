#![feature(try_blocks)]
use lazy_static::lazy_static;
use std::io::Write;
use std::{net::SocketAddr, str::SplitWhitespace};
use volo::FastStr;
lazy_static! {
    static ref CLIENT: volo_gen::volo::redis::RedisClient = {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        volo_gen::volo::redis::RedisClientBuilder::new("redis-client")
            .address(addr)
            .build()
    };
}

async fn get_cache(key: &str) -> Option<String> {
    let req = volo_gen::volo::redis::GetRequest {
        key: FastStr::new(key),
    };
    let resp = CLIENT.get(req).await;
    match resp {
        Ok(resp) => resp.value.map(|s| s.as_str().to_string()),
        Err(e) => {
            tracing::error!("{:?}", e);
            None
        }
    }
}

async fn set_cache(key: &str, value: &str, ttl: Option<i32>) {
    let req = volo_gen::volo::redis::SetRequest {
        key: FastStr::new(key),
        value: FastStr::new(value),
        ttl,
    };
    let resp = CLIENT.set(req).await;
    match resp {
        Ok(resp) => {
            if !resp.success {
                tracing::error!("set failed");
            }
        }
        Err(e) => tracing::error!("{:?}", e),
    }
}

async fn del_cache(key: &str) {
    let req = volo_gen::volo::redis::DelRequest {
        key: FastStr::new(key),
    };
    let resp = CLIENT.del(req).await;
    match resp {
        Ok(resp) => {
            if !resp.success {
                tracing::error!("del failed");
            }
        }
        Err(e) => tracing::error!("{:?}", e),
    }
}

async fn ping() {
    let resp = CLIENT.ping().await;
    match resp {
        Ok(resp) => {
            tracing::info!("{:?}", resp.as_str());
        }
        Err(e) => tracing::error!("{:?}", e),
    }
}

fn print_help_message() {
    println!("Commands:");
    println!("get <key>");
    println!("set <key> <value> <ttl>(optional)");
    println!("del <key>");
    println!("ping");
    println!("help");
    println!("exit");
}

fn try_get_next<'a>(iter: &mut SplitWhitespace<'a>) -> Result<&'a str, Box<dyn std::error::Error>> {
    match iter.next() {
        Some(s) => Ok(s),
        None => Err("wrong args".into()),
    }
}

async fn handle_cmd(cmd: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut iter = cmd.split_whitespace();
    match iter.next() {
        Some("get") => {
            let key = try_get_next(iter.by_ref())?;
            match get_cache(key).await {
                Some(value) => {
                    println!("the value in {} is \x1b[34m{}\x1b[0m", key, value);
                }
                None => {
                    tracing::error!("key not found");
                }
            }
        }
        Some("set") => {
            let key = try_get_next(iter.by_ref())?;
            let value = try_get_next(iter.by_ref())?;
            let ttl = iter.next().map(|s| s.parse::<i32>().unwrap());
            set_cache(key, value, ttl).await;
        }
        Some("del") => {
            let key = try_get_next(iter.by_ref())?;
            del_cache(key).await;
        }
        Some("ping") => {
            ping().await;
        }
        Some("exit") => {
            std::process::exit(0);
        }
        Some("help") => {
            print_help_message();
        }
        _ => {
            tracing::error!("unknown cmd");
            print_help_message();
        }
    }
    Ok(())
}

async fn little_cli() {
    print_help_message();
    loop {
        let mut cmd = String::new();
        print!("> ");
        let _ = std::io::stdout().flush();
        std::io::stdin().read_line(&mut cmd).unwrap();
        match handle_cmd(cmd.trim()).await {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("{:?}", e);
                print_help_message();
            }
        }
    }
}

#[volo::main]
async fn main() {
    tracing_subscriber::fmt::init();
    little_cli().await;
}
