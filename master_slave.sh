cargo update
cargo build
cargo run --bin server master &
cargo run --bin server slave1 &
cargo run --bin server slave2 &
