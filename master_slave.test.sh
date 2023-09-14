./master_slave.sh
sleep 3
cargo run --bin client "127.0.0.1:19261" set key1 value1 > /dev/null
cargo run --bin client "127.0.0.1:19262" get key1 | grep -q "value1"
if test $? -eq 0
then
    echo "[Master Slave Test 1] \x1b[32mok\x1b[0m"
else
    echo "[Master Slave Test 1] \x1b[31mFAILED\x1b[0m"
fi
cargo run --bin client "127.0.0.1:19263" get key1 | grep -q "value1"
if test $? -eq 0
then
    echo "[Master Slave Test 2] \x1b[32mok\x1b[0m"
else
    echo "[Master Slave Test 2] \x1b[31mFAILED\x1b[0m"
fi