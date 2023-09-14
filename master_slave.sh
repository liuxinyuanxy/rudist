kill $(lsof -t -i:19261) &> /dev/null
kill $(lsof -t -i:19262) &> /dev/null
kill $(lsof -t -i:19263) &> /dev/null
kill $(lsof -t -i:19290) &> /dev/null

./server master > /dev/null &
sleep 1
./server slave1 > /dev/null &
./server slave2 > /dev/null &
