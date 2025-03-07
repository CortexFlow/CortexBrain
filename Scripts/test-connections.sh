#!/bin/bash

proxy_pod_name=$(kubectl get pods -n cortexflow --no-headers -o custom-columns=":metadata.name" | grep cortexflow-proxy)
proxy_ip=$(kubectl get -o template service/proxy-service -n cortexflow --template='{{.spec.clusterIP}}')

echo "🧑🏻‍🔬 Checking cortexflow proxy inside the proxy pod: $proxy_pod_name"

sleep 1.5
echo "🔨 checking env variables"
kubectl exec -n cortexflow $proxy_pod_name -- env

sleep 1.5

if ! kubectl exec -n cortexflow $proxy_pod_name -- which netstat >/dev/null 2>&1; then
    echo "🔨 installing netstat"
    kubectl exec -n cortexflow $proxy_pod_name -- apt update
    kubectl exec -n cortexflow $proxy_pod_name -- apt install -y net-tools
else
    echo "✅ Netstat is installed."
fi

sleep 1.5

if ! kubectl exec -n cortexflow $proxy_pod_name -- which nc >/dev/null 2>&1; then
    echo "🔨 installing netcat"
    kubectl exec -n cortexflow $proxy_pod_name -- apt install -y netcat
else
    echo "✅ Netcat is installed."
fi

sleep 1.5

if ! kubectl exec -n cortexflow $proxy_pod_name -- which curl >/dev/null 2>&1; then
    echo "🔨 installing curl"
    kubectl exec -n cortexflow $proxy_pod_name -- apt install -y curl
else
    echo "✅ Curl is installed."
fi

sleep 1.5

if ! kubectl exec -n cortexflow $proxy_pod_name -- which nslookup >/dev/null 2>&1; then
    echo "🔨 installing dnsutils"
    kubectl exec -n cortexflow $proxy_pod_name -- apt install -y dnsutils
else
    echo "✅ Nslookup is installed."
fi

sleep 1.5

if ! kubectl exec -n cortexflow $proxy_pod_name -- which tcpdump >/dev/null 2>&1; then
    echo "🔨 installing tcpdump"
    kubectl exec -n cortexflow $proxy_pod_name -- apt install -y tcpdump
else
    echo "✅ tcpdump is installed."
fi

sleep 1.5

echo
echo "🔨 Testing netstat command"
kubectl exec -n cortexflow $proxy_pod_name -- netstat -tulnp | grep 9090

sleep 1.5

echo
echo "🔨 testing if the process is in execution"
kubectl exec -n cortexflow $proxy_pod_name -- ps aux | grep cortexflow-proxy

echo "🔨 testing using netcat"
kubectl exec -n cortexflow $proxy_pod_name -- nc -zv proxy-service.cortexflow.svc.cluster.local 9090

sleep 1.5
echo "🔨 Checking if the proxy is listening in the 5053 port"
kubectl exec -n cortexflow $proxy_pod_name -- netstat -ulnp

echo
sleep 1.5
echo "🔨 Sending a test package with netcat"
kubectl exec -n cortexflow $proxy_pod_name -- sh -c echo b"Hi CortexFlow" | nc -u -w5 -v 127.0.0.1 5053 

echo
sleep 1.5
echo "🔨 Testing the DNS resolution manually with nslookup"
kubectl exec -n cortexflow $proxy_pod_name -- nslookup proxy-service.cortexflow.svc.cluster.local

sleep 1.5

echo
echo "🔨 Testing curl command"
response=$(kubectl exec -n cortexflow $proxy_pod_name -- curl -s -o /dev/null -w "%{http_code}" http://localhost:9090/)
if [ "$response" -eq 200 ]; then
  echo "✅ Server is working"
  echo " Checking / endpoint"
  kubectl exec -n cortexflow $proxy_pod_name -- curl -v http://localhost:9090/
else
  echo "❌ Error in http response ERROR: $response. Service does not exists or is not exposed"
fi

echo
sleep 1.5
echo "🔨 Testing /health endpoint"
response=$(kubectl exec -n cortexflow $proxy_pod_name -- curl -s -o /dev/null -w "%{http_code}" http://localhost:9090/health)
if [ "$response" -eq 200 ]; then
  echo "✅ Server is working"
  echo " Checking /health endpoint"
  kubectl exec -n cortexflow $proxy_pod_name -- curl -v http://localhost:9090/health
else
  echo "❌ Error in http response ERROR: $response. Service does not exists or is not exposed"
fi

echo
sleep 1.5
echo "🔨 Testing /metrics endpoint"
response=$(kubectl exec -n cortexflow $proxy_pod_name -- curl -s -o /dev/null -w "%{http_code}" http://localhost:9090/metrics)
if [ "$response" -eq 200 ]; then
  echo "✅ Server is working"
  echo " Checking /metrics endpoint"
  kubectl exec -n cortexflow $proxy_pod_name -- curl -v http://localhost:9090/metrics
else
  echo "❌ Error in http response ERROR: $response. Service does not exists or is not exposed"
fi

echo
sleep 1.5
echo "🔨 Testing /status endpoint"
response=$(kubectl exec -n cortexflow $proxy_pod_name -- curl -s -o /dev/null -w "%{http_code}" http://localhost:9090/status)
if [ "$response" -eq 200 ]; then
  echo "✅ Server is working"
  echo " Checking /status endpoint"
  kubectl exec -n cortexflow $proxy_pod_name -- curl -v http://localhost:9090/status
else
  echo "❌ Error in http response ERROR: $response. Service does not exists or is not exposed"
fi

echo
echo
echo "🧑🏻‍🔬 Testing outside the proxy pod using a test pod"
#echo "🔨 Testing using a temporary test pod and nslookup"
#kubectl run -it --rm --image=busybox test-pod --restart=Never -n cortexflow -- nslookup proxy-service.cortexflow.svc.cluster.local
#kubectl delete pod test-pod -n cortexflow

echo
sleep 1.5
echo "🔨 Sending a test message using netcat and a temporary test pod"
kubectl run -it --image=busybox test-pod --restart=Never -n cortexflow -- sh -c "echo -n Hi CortexFlow | nc -u -v 10.108.156.183 5053"
kubectl delete pod -n cortexflow test-pod 