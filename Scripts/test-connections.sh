proxy_pod_name=$(kubectl get pods -n cortexflow --no-headers -o custom-columns=":metadata.name" | grep cortexflow-proxy)
echo "Checking cortexflow proxy inside the pod"

sleep 1.5
echo "🔨 checking env variables"
kubectl exec -it -n cortexflow $proxy_pod_name -- env

sleep 1.5

if ! kubectl exec -it -n cortexflow $proxy_pod_name -- which netstat >/dev/null 2>&1; then
    echo "🔨 installing netstat"
    kubectl exec -it -n cortexflow $proxy_pod_name -- apt update
    kubectl exec -it -n cortexflow $proxy_pod_name -- apt install net-tools
else
    echo "✅ Netstat is installed."
fi

sleep 1.5

if ! kubectl exec -it -n cortexflow $proxy_pod_name -- which netcat-traditional >/dev/null 2>&1; then
    echo "🔨 installing netcat"
    kubectl exec -it -n cortexflow $proxy_pod_name -- apt install netcat-traditional
else
    echo "✅ Netcat is installed."
fi
echo 
sleep 1.5

echo
if ! kubectl exec -it -n cortexflow $proxy_pod_name -- which curl >/dev/null 2>&1; then
    echo "🔨 installing curl"   
    kubectl exec -it -n cortexflow $proxy_pod_name -- apt install curl
else
    echo "✅ Curl is installed."
fi

sleep 1.5
echo
if ! kubectl exec -it -n cortexflow $proxy_pod_name -- which dnsutils >/dev/null 2>&1; then
    echo "🔨 installing nslookup"   
    kubectl exec -it -n cortexflow $proxy_pod_name -- apt install dnsutils
else
    echo "✅ Nslookup is installed."
fi

sleep 1.5

echo
echo "🔨 Testing netstat command"
kubectl exec -it -n cortexflow $proxy_pod_name -- netstat -tulnp | grep 9090

sleep 1.5

echo
echo "🔨 testing if the process is in execution"
kubectl exec -it -n cortexflow $proxy_pod_name -- ps aux | grep cortexflow-proxy

echo "🔨 testing using netcat"
kubectl exec -it -n cortexflow $proxy_pod_name -- nc -zv proxy-service.cortexflow.svc.cluster.local 9090


sleep 1.5
echo "🔨 Checking if the proxy is listening in the 5053 port"
kubectl exec -it -n cortexflow $proxy_pod_name -- netstat -ulnp

echo
sleep 1.5
echo "🔨 Sending a test package"
kubectl exec -it -n cortexflow $proxy_pod_name -- echo "test" | nc -u -w1 proxy-service.cortexflow.svc.cluster.local 5053

echo 
sleep 1.5
echo "🔨 Testing the DNS resolution manually"
kubectl exec -it -n cortexflow $proxy_pod_name -- nslookup proxy-service.cortexflow.svc.cluster.local


sleep 1.5

echo
echo "🔨 Testing curl command"
response=$(kubectl exec -it -n cortexflow $proxy_pod_name -- curl -s -o /dev/null -w "%{http_code}" http://localhost:9090/)
if [ "$response" -eq 200 ]; then
  echo "✅ Server is working"
  echo " Checking / endpoint"
  kubectl exec -it -n cortexflow $proxy_pod_name -- curl -v http://localhost:9090/

else
  echo "❌ Error in http response ERROR: $response"
  echo "❌ Service does not exists or is not exposed"
fi

echo
sleep 1.5
echo "🔨 Testing /health endpoint"
response=$(kubectl exec -it -n cortexflow $proxy_pod_name -- curl -s -o /dev/null -w "%{http_code}" http://localhost:9090/health)
if [ "$response" -eq 200 ]; then
  echo "✅ Server is working"
  echo " Checking /health endpoint"
  kubectl exec -it -n cortexflow $proxy_pod_name -- curl -o -v http://localhost:9090/health

else
  echo "❌ Error in http response ERROR: $response"
  echo "❌ Service does not exists or is not exposed"
fi

echo
sleep 1.5
echo "🔨 Testing /metrics endpoint"
response=$(kubectl exec -it -n cortexflow $proxy_pod_name -- curl -s -o /dev/null -w "%{http_code}" http://localhost:9090/metrics)
if [ "$response" -eq 200 ]; then
  echo "✅ Server is working"
  echo " Checking /metrics endpoint"
  kubectl exec -it -n cortexflow $proxy_pod_name -- curl -o -v http://localhost:9090/metrics
else
  echo "❌ Error in http response ERROR: $response"
  echo "❌ Service does not exists or is not exposed"
fi

echo
sleep 1.5
echo "🔨 Testing /status endpoint"
response=$(kubectl exec -it -n cortexflow $proxy_pod_name -- curl -s -o /dev/null -w "%{http_code}" http://localhost:9090/status)
if [ "$response" -eq 200 ]; then
  echo "✅ Server is working"
  echo " Checking /status endpoint"
  kubectl exec -it -n cortexflow proxy_pod_name -- curl -o -v http://localhost:9090/status

else
  echo "❌ Error in http response ERROR: $response"
  echo "❌ Service does not exists or is not exposed"
fi
