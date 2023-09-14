./master_slave.sh
sleep 3
./client "127.0.0.1:19261" set key1 value2 > /dev/null
for i in {1..1000}
do
    ./client "127.0.0.1:19261" set key1 value2 > /dev/null
done
./client  "127.0.0.1:19261" set key1 value1 > /dev/null
./client  "127.0.0.1:19262" get key1 | grep -q "value1"
if test $? -eq 0
then
    echo -e "[Master Slave Test 1] \x1b[32mok\x1b[0m"
else
    echo -e "[Master Slave Test 1] \x1b[31mFAILED\x1b[0m"
fi
./client  "127.0.0.1:19263" get key1 | grep -q "value1"
if test $? -eq 0
then
    echo -e "[Master Slave Test 2] \x1b[32mok\x1b[0m"
else
    echo -e "[Master Slave Test 2] \x1b[31mFAILED\x1b[0m"
fi
./client "127.0.0.1:19261" del key1 > /dev/null
./client "127.0.0.1:19262" get key1 | grep -q "value1"
if test $? -eq 1
then
    echo -e "[Master Slave Test 3] \x1b[32mok\x1b[0m"
else
    echo -e "[Master Slave Test 3] \x1b[31mFAILED\x1b[0m"
fi

./client "127.0.0.1:19261" set key1 value1 > /dev/null
./client "127.0.0.1:19263" get key1 | grep -q "value1"
if test $? -eq 0
then
    echo -e "[Master Slave Test 4] \x1b[32mok\x1b[0m"
else
    echo -e "[Master Slave Test 4] \x1b[31mFAILED\x1b[0m"
fi

./client "127.0.0.1:19261" set key1 value2 > /dev/null
./client "127.0.0.1:19262" get key1 | grep -q "value2"
if test $? -eq 0
then
    echo -e "[Master Slave Test 5] \x1b[32mok\x1b[0m"
else
    echo -e "[Master Slave Test 5] \x1b[31mFAILED\x1b[0m"
fi
./client "127.0.0.1:19262" set key1 value2
