cargo update
cargo build
cargo run --bin server master > /dev/null &
sleep 1
cargo run --bin server slave1 > /dev/null &
cargo run --bin server slave2 > /dev/null &
