#!/bin/bash
cargo update
cargo build
cargo run --bin server master > /dev/null &
cargo run --bin client "127.0.0.1:19261" get key1 | grep -q "key not found"
if test $? -eq 0
then
    echo -e "[Recover Test 1] \x1b[32mok\x1b[0m"
else
    echo -e "[Recover Test 1] \x1b[31mFAILED\x1b[0m"
fi

cargo run --bin client "127.0.0.1:19261" get key2 | grep -q "value2"
if test $? -eq 0
then
    echo -e "[Recover Test 2] \x1b[32mok\x1b[0m"
else
    echo -e "[Recover Test 2] \x1b[31mFAILED\x1b[0m"
fi

cargo run --bin client "127.0.0.1:19261" set key2 value2
cargo run --bin client "127.0.0.1:19261" set key3 value3
cargo run --bin client "127.0.0.1:19261" del key2

kill $(lsof -t -i:19261)

cargo run --bin server master > /dev/null &
cargo run --bin client "127.0.0.1:19261" get key1 | grep -q "key not found"
if test $? -eq 0
then
    echo -e "[Write AOF Log Test 1] \x1b[32mok\x1b[0m"
else
    echo -e "[Write AOF Log Test 1] \x1b[31mFAILED\x1b[0m"
fi

cargo run --bin client "127.0.0.1:19261" get key2 | grep -q "key not found"
if test $? -eq 0
then
    echo -e "[Write AOF Log Test 2] \x1b[32mok\x1b[0m"
else
    echo -e "[Write AOF Log Test 2] \x1b[31mFAILED\x1b[0m"
fi

cargo run --bin client "127.0.0.1:19261" get key3 | grep -q "value3"
if test $? -eq 0
then
    echo -e "[Write AOF Log Test 3] \x1b[32mok\x1b[0m"
else
    echo -e "[Write AOF Log Test 3] \x1b[31mFAILED\x1b[0m"
fi

cargo run --bin client "127.0.0.1:19261" set key4 value4 5
cargo run --bin client "127.0.0.1:19261" get key4 | grep -q "value4"
if test $? -eq 0
then
    echo -e "[Write AOF Log Test 4] \x1b[32mok\x1b[0m"
else
    echo -e "[Write AOF Log Test 4] \x1b[31mFAILED\x1b[0m"
fi

sleep 5
cargo run --bin client "127.0.0.1:19261" get key4 | grep -q "key not found"
if test $? -eq 0
then
    echo -e "[Write AOF Log Test 5] \x1b[32mok\x1b[0m"
else
    echo -e "[Write AOF Log Test 5] \x1b[31mFAILED\x1b[0m"
fi

kill $(lsof -t -i:19261)