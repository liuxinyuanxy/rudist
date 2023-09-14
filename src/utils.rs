pub async fn set_to_string(key: &str, value: &str, ttl: Option<i32>) -> String {
    match ttl {
        Some(ttl) => {
            let expire_at = ttl as u128 * 1000
                + std::time::SystemTime::now()
                    .duration_since(std::time::SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
            let mut buf = format!(
                "*4\n$3\nset\n${}\n{}\n${}\n{}\n",
                key.len(),
                key,
                value.len(),
                value
            );
            if ttl > 0 {
                buf.push_str(&format!(
                    "${}\n{}\n",
                    expire_at.to_string().len(),
                    expire_at
                ));
            }
            buf
        }
        None => {
            format!(
                "*3\n$3\nset\n${}\n{}\n${}\n{}\n",
                key.len(),
                key,
                value.len(),
                value
            )
        }
    }
}

pub async fn del_to_string(key: &str) -> String {
    format!("*2\n$3\ndel\n${}\n{}\n", key.len(), key)
}
