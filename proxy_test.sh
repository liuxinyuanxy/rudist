./master_slave.sh
sleep 3
cargo run --bin client "127.0.0.1:19261" set key1 value1 > /dev/null
cargo run --bin client "127.0.0.1:19262" get key1 | grep value1
cargo run --bin proxy