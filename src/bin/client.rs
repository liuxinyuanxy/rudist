#![feature(try_blocks)]
use lazy_static::lazy_static;
use miniredis::CONFIG;
use std::io::Write;
use std::{net::SocketAddr, str::SplitWhitespace};
use volo::FastStr;
lazy_static! {
    static ref CLIENT: volo_gen::volo::redis::RedisClient = {
        let args: Vec<String> = std::env::args().collect();
        let addr = args[1].clone();
        let addr: SocketAddr = addr.parse().unwrap();
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
    println!("subscribe <topic1> <topic2> ...");
    println!("publish <topic> <message>");
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

async fn subscribe(topics: &Vec<String>) -> Vec<i32> {
    let req = volo_gen::volo::redis::SubscribeRequest {
        topics: topics.iter().map(|s| FastStr::new(s)).collect(),
    };
    let resp = CLIENT.subscribe(req).await;
    match resp {
        Ok(resp) => resp.offsets,
        Err(e) => {
            tracing::error!("{:?}", e);
            Vec::new()
        }
    }
}

async fn keep_polling(
    offsets: Vec<i32>,
    topics: &Vec<String>,
    mut signal: tokio::sync::oneshot::Receiver<()>,
) {
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    let mut offsets = offsets;
    loop {
        let req = volo_gen::volo::redis::PollRequest {
            topics: topics.iter().map(|s| FastStr::new(s)).collect(),
            offsets: offsets,
        };
        let resp = CLIENT.poll(req).await;
        match resp {
            Ok(resp) => {
                for message in resp.messages.iter() {
                    println!(
                        "get message from \x1b[32m{}\x1b[0m : \x1b[34m{}\x1b[0m",
                        message.topic,
                        message.data.as_str()
                    );
                }
                offsets = resp.offsets;
            }
            Err(e) => {
                tracing::error!("{:?}", e);
                break;
            }
        }
        // if receive signal, break
        if signal.try_recv().is_ok() {
            break;
        }
    }
}

async fn unsubscribe(topics: &Vec<String>) {
    let req = volo_gen::volo::redis::UnsubscribeRequest {
        topics: topics.iter().map(|s| FastStr::new(s)).collect(),
    };
    let resp = CLIENT.unsubscribe(req).await;
    match resp {
        Ok(resp) => {
            if !resp.success {
                tracing::error!("unsubscribe failed");
            }
        }
        Err(e) => tracing::error!("{:?}", e),
    }
}

async fn publish(topic: &str, message: &str) {
    let req = volo_gen::volo::redis::PublishRequest {
        topic: FastStr::new(topic),
        data: FastStr::new(message),
    };
    let resp = CLIENT.publish(req).await;
    match resp {
        Ok(resp) => {
            if !resp.success {
                tracing::error!("publish failed");
            }
        }
        Err(e) => tracing::error!("{:?}", e),
    }
}

async fn sub_mode(topics: Vec<String>) {
    println!("Entered sub mode, press Ctrl-C to exit");
    let offsets = subscribe(&topics).await;
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let topics_clone = topics.clone();
    let _ = tokio::spawn(async move {
        keep_polling(offsets, &topics_clone, rx).await;
    });
    let _ = tokio::signal::ctrl_c().await;
    let _ = tx.send(());
    unsubscribe(&topics).await;
    println!("Exited sub mode");
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
        Some("subscribe") => {
            let topics: Vec<String> = iter.map(|s| s.to_string()).collect();
            sub_mode(topics).await;
        }
        Some("publish") => {
            let topic = try_get_next(iter.by_ref())?;
            let message = try_get_next(iter.by_ref())?;
            publish(topic, message).await;
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
        // let _ = tokio::spawn(async move {
        match handle_cmd(cmd.trim()).await {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("{:?}", e);
                print_help_message();
            }
        };
        // });
    }
}

#[volo::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = std::env::args().collect();
    // the command is all args after 1
    let cmd = args[1..].join(" ");
    handle_cmd(cmd.as_str()).await.unwrap();
    // little_cli().await;
}
