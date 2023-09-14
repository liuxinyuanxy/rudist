./master_slave.sh
cargo run --bin proxy proxy &
sleep 3
cargo run --bin client "127.0.0.1:19290" set key1 value1 > /dev/null
sleep 3
cargo run --bin client "127.0.0.1:19290" get key1 | grep value1
if test $? -eq 0
then
    echo -e "[Master Slave Test 1] \x1b[32mok\x1b[0m"
else
    echo -e "[Master Slave Test 1] \x1b[31mFAILED\x1b[0m"
fi

cargo run --bin proxy proxy & 
sleep 3
cargo run --bin client "127.0.0.1:19290" set key2 value2 > /dev/null
cargo run --bin client "127.0.0.1:19290" set key3 value3 > /dev/null
sleep 3 
cargo run --bin client "127.0.0.1:19290" get key3 | grep value3
cargo run --bin client "127.0.0.1:19290" get key2 | grep value2
if test $? -eq 0
then
    echo -e "[Master Slave Test 2] \x1b[32mok\x1b[0m"
else
    echo -e "[Master Slave Test 2] \x1b[31mFAILED\x1b[0m"
fi
cargo run --bin proxy proxy &
for i in {1..10}
do 
  cargo run --bin client "127.0.0.1:19290" set key2 value2 > /dev/null
done
sleep 3
cargo run --bin client "127.0.0.1:19290" set key2 value3 > /dev/null
cargo run --bin client "127.0.0.1:19290" set key3 value3 > /dev/null
cargo run --bin client "127.0.0.1:19290" get key3 | grep value3
cargo run --bin client "127.0.0.1:19290" get key2 | grep value3
if test $? -eq 0 
then 
  echo -e "[Master Slave Test 2] \x1b[32mok\x1b[0m"
else
    echo -e "[Master Slave Test 2] \x1b[31mFAILED\x1b[0m"
fi