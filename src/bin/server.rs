#![feature(impl_trait_in_assoc_type)]

mod middleware;
use middleware::{CheckLayer, LogLayer};
use miniredis::CONFIG;
use miniredis::S;
use std::net::SocketAddr;

#[volo::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let addr: SocketAddr = CONFIG.get_my_addr().parse().unwrap();
    let addr = volo::net::Address::from(addr);

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
    volo_gen::volo::redis::RedisServer::new(S)
        .layer_front(LogLayer)
        .layer_front(CheckLayer)
        .run(addr)
        .await
        .unwrap();
    let _ = tx.send(());
}
