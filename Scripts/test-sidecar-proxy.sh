#!/bin/bash

echo "Testing Sidecar proxy injection "

sleep 1
echo "Checking pods"
kubectl get pods -o wide -n cortexflow
echo
echo "Checking if the sidecar proxy is present"
kubectl get pods -n cortexflow -o json | jq '.items[].spec.containers[].name'

echo 
sleep 1
echo "Checking open ports in test-proxy"
kubectl get pods test-proxy -o jsonpath='{.spec.containers[*].ports}' -n cortexflow 
echo
kubectl get pods test-proxy2 -o jsonpath='{.spec.containers[*].ports}' -n cortexflow 

echo
echo
echo "Installing debugging tools in test-proxy: (PROXY-SIDECAR container)"
sleep 3
./install-debugging-tools.sh test-proxy proxy-sidecar
echo 
echo
echo "Installing debugging tools in test-proxy2: (PROXY-SIDECAR container)"
sleep 3
./install-debugging-tools.sh test-proxy2 proxy-sidecar

echo 
echo
echo "Checking network connections in test-proxy pod "
kubectl exec -it test-proxy -c proxy-sidecar -n cortexflow -- netstat -tulnp
echo 
echo "Checking network connections in test-proxy2 pod"
kubectl exec -it test-proxy2 -c proxy-sidecar -n cortexflow -- netstat -tulnp


echo
sleep 2
echo "TEST 1: Checking if test-proxy can communicate with test-proxy2"
kubectl exec -it test-proxy -c proxy-sidecar -n cortexflow -- nc -zv test-proxy2.cortexflow.svc.cluster.local 5054
echo

echo

echo "TEST 2: Checking if test-proxy can communicate with test-proxy2 (TCP)"

# 2. Send the message from test-proxy to test-proxy2
kubectl exec test-proxy -c proxy-sidecar -n cortexflow -- sh -c '
    echo "Test: Incoming Message ⏳"
    printf "{\"service\":\"test-proxy2.cortexflow\",\"direction\":\"Incoming\",\"payload\":\"eyJwYXlsb2FkIjogIkhlbGxvIGZyb20gcHJveHktc2lkZWNhciJ9\"}\n" | nc -w3 test-proxy2 5054 && echo "✅ Test completed"
'

echo
sleep 2
echo
echo "TEST 2: Sending a message from test-proxy to test-proxy2 (UDP)"

#Start the UDP listener on test-proxy2 (MUST be before sending the message)
kubectl exec test-proxy2 -c proxy-sidecar -n cortexflow -- sh -c '
    echo "Starting UDP listener on port 5053..."
    nohup sh -c "nc -lu -p 5053 > /tmp/received_message.log" >/dev/null 2>&1 &
    sleep 2  # Wait for the listener to start
'

#2. Send the message from test-proxy to test-proxy2
kubectl exec test-proxy -c proxy-sidecar -n cortexflow -- sh -c '
    echo "Test: Incoming Message ⏳"
    echo "{\"service\":\"test-proxy2.cortexflow\",\"direction\":\"Incoming\",\"payload\":\"eyJtZXNzYWdlIjogIkhlbGxvIGZyb20gcHJveHktc2lkZWNhciJ9\"}" | nc -u -w3 test-proxy2 5053 && echo "✅ Test completed"
'
