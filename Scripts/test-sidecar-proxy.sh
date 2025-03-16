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
echo "Installing debugging tools in test-proxy: (NGINX container)"
sleep 3
./install-debugging-tools.sh test-proxy nginx
echo 
echo
echo "Installing debugging tools in test-proxy2: (NGINX container)"
sleep 3
./install-debugging-tools.sh test-proxy2 nginx

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
kubectl exec -it test-proxy -n cortexflow -- netstat -tulnp
echo 
echo "Checking network connections in test-proxy2 pod"
kubectl exec -it test-proxy2 -n cortexflow -- netstat -tulnp


echo
sleep 2
echo "TEST 1: Checking if test-proxy can communicate with test-proxy2"
kubectl exec -it test-proxy -c proxy-sidecar -n cortexflow -- nc -zv test-proxy2.cortexflow.svc.cluster.local 5054
echo
echo "TEST 2: Sending a message from test-proxy to test-proxy2"

# Start Netcat listening on test-proxy2 (background)
kubectl exec test-proxy2 -c proxy-sidecar -n cortexflow -- sh -c \
  'nc -l -p 5054 > /tmp/proxy_test_output' &
sleep 2

#Send a message from test proxt to test proxy 2 using the 5054 TCP port 
kubectl exec test-proxy -c proxy-sidecar -n cortexflow -- sh -c \
  'echo "Hello from proxy-sidecar!" | nc test-proxy2.cortexflow.svc.cluster.local 5054'
sleep 2

# Recupera l'output direttamente dal pod test-proxy2
# Check the output from test proxy 2 
kubectl exec test-proxy2 -c proxy-sidecar -n cortexflow -- sh -c \
  'cat /tmp/proxy_test_output' | grep -q "Hello from proxy-sidecar!" && \
  echo "✅ Test PASSED: Message received !" || \
  echo "❌ Test FAILED: Message Unavailable"

#remove temporary files
kubectl exec test-proxy2 -c proxy-sidecar -n cortexflow -- rm -f /tmp/proxy_test_output
