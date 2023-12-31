use crate::CACHE;
use lazy_static::lazy_static;
use std::collections::HashMap;
use tokio::sync::Mutex;

struct Entity {
    commands: Mutex<Vec<String>>,
    valid: Mutex<bool>,
}

impl Entity {
    fn new() -> Self {
        Self {
            commands: Mutex::new(Vec::new()),
            valid: Mutex::new(true),
        }
    }
    async fn set_invalid(&self) {
        let mut inner = self.valid.lock().await;
        *inner = false;
    }
}

pub struct Transaction {
    data: Mutex<HashMap<String, Entity>>,
}

impl Transaction {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(HashMap::new()),
        }
    }

    pub async fn new_transaction(&self) -> String {
        let transaction_id = uuid::Uuid::new_v4().to_string();
        let entity = Entity::new();
        self.data
            .lock()
            .await
            .insert(transaction_id.clone(), entity);
        transaction_id
    }

    pub async fn add_command(&self, transaction_id: &str, command: &str) {
        let inner = self.data.lock().await;
        let entity = inner.get(transaction_id).unwrap();
        entity.commands.lock().await.push(command.to_string());
    }

    pub async fn set_invalid(&self, transaction_id: &str) {
        let inner = self.data.lock().await;
        let entity = inner.get(transaction_id).unwrap();
        entity.set_invalid().await;
    }

    pub async fn exec(&self, transaction_id: &str) -> Option<Vec<Option<String>>> {
        let mut res = Vec::new();
        let inner = self.data.lock().await;
        let entity = inner.get(transaction_id).unwrap();
        if !*entity.valid.lock().await {
            return None;
        }
        let commands = entity.commands.lock().await;

        tracing::info!("{:?}", commands);
        let mut commands_splited: Vec<String> = Vec::new();
        let lines = commands.iter();
        lines.for_each(|x| {
            let mut line = x.split('\n');
            while let Some(value) = line.next() {
                commands_splited.push(value.to_string())
            }
        });
        let mut lines = commands_splited.iter();
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
                match cmd.as_str() {
                    "set" => {
                        let value = args[2];
                        let mut ttl = None;
                        if len == 4 {
                            let expire_at = args[3].parse::<u128>().unwrap();
                            let now = std::time::SystemTime::now()
                                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                                .unwrap()
                                .as_millis();
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
                    "get" => {
                        res.push(CACHE.get(key).await);
                    }
                    _ => {
                        panic!("unknown command");
                    }
                }
            }
        }
        Some(res)
    }
}

lazy_static! {
    pub static ref TRANSACTION: Transaction = Transaction::new();
}
