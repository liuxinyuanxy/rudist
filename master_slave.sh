kill $(lsof -t -i:19261) &> /dev/null
kill $(lsof -t -i:19262) &> /dev/null
kill $(lsof -t -i:19263) &> /dev/null
kill $(lsof -t -i:19290) &> /dev/null

cargo run --bin server master > /dev/null &
sleep 1
cargo run --bin server slave1 > /dev/null &
cargo run --bin server slave2 > /dev/null &
