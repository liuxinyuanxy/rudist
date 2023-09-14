#!/bin/bash
cargo update
cargo build
cargo run --bin server master
cargo run --bin client "127.0.0.1:19261" get key1
cargo run --bin client "127.0.0.1:19261" set key2 value2
cargo run --bin client "127.0.0.1:19261" set key3 value3
cargo run --bin client "127.0.0.1:19261" del key2
kill $(lsof -t -i:19261)
cargo run --bin server master
cargo run --bin client "127.0.0.1:19261" get key1
cargo run --bin client "127.0.0.1:19261" get key2
cargo run --bin client "127.0.0.1:19261" get key3