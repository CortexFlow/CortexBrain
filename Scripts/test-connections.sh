#!/bin/bash

proxy_pod_name=$(kubectl get pods -n cortexflow --no-headers -o custom-columns=":metadata.name" | grep cortexflow-proxy)
proxy_ip=$(kubectl get -o template service/proxy-service -n cortexflow --template='{{.spec.clusterIP}}')
proxy_udp_port=5053
proxy_metrics_port=9090

echo "🧑🏻‍🔬 Checking cortexflow proxy inside the proxy pod: $proxy_pod_name"

sleep 1.5
echo "🔨 checking env variables"
kubectl exec -n cortexflow $proxy_pod_name -- env

sleep 1.5

./install-debugging-tools.sh $proxy_pod_name
echo
./test-proxy-ports $proxy_pod_name $proxy_metrics_port
echo
sleep 1.5
echo "🔨 Sending a test package with netcat from proxy pod -> proxy pod"
kubectl exec -n cortexflow $proxy_pod_name -- sh -c echo b"Hi CortexFlow" | nc -u -w5 -v 127.0.0.1 $proxy_udp_port 

echo
sleep 1.5
echo "🔨 Testing the DNS resolution manually with nslookup"
kubectl exec -n cortexflow $proxy_pod_name -- nslookup proxy-service.cortexflow.svc.cluster.local

sleep 1.5
echo
./test-proxy-endpoints.sh $proxy_pod_name
echo
echo
echo "🧑🏻‍🔬 Testing outside the proxy pod using a test pod"
echo "🔨 Testing using a temporary test pod and nslookup"
kubectl run -it --rm --image=busybox test-pod --restart=Never -n cortexflow -- nslookup proxy-service.cortexflow.svc.cluster.local

echo
sleep 1.5
echo "🔨 Sending a test message using netcat and a temporary test pod"
kubectl run -it --rm --image=busybox test-pod --restart=Never -n cortexflow -- sh -c "echo -n Hi CortexFlow | nc -u -w 3 -v $proxy_ip $proxy_udp_port"
