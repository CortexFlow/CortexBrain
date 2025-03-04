echo "Checking cortexflow proxy inside the pod"

sleep 1.5
echo "🔨 checking env variables"
kubectl exec -it -n cortexflow cortexflow-proxy-84dfff5c49-zzpgq -- env

sleep 1.5

if ! which netstat >/dev/null 2>&1; then
    echo "🔨 installing netstat"
    kubectl exec -it -n cortexflow cortexflow-proxy-84dfff5c49-zzpgq -- apt update
    kubectl exec -it -n cortexflow cortexflow-proxy-84dfff5c49-zzpgq -- apt install net-tools
else
    echo "✅ Netstat is installed."
fi

sleep 1.5

echo
if ! which curl >/dev/null 2>&1; then
    echo "🔨 installing curl"   
    kubectl exec -it -n cortexflow cortexflow-proxy-84dfff5c49-zzpgq -- apt install curl
else
    echo "✅ Curl is installed."
fi

sleep 1.5

echo
echo "🔨 Testing netstat command"
kubectl exec -it -n cortexflow cortexflow-proxy-84dfff5c49-zzpgq -- netstat -tulnp | grep 9090

sleep 1.5

echo
echo "🔨 testing if the process is in execution"
kubectl exec -it -n cortexflow cortexflow-proxy-84dfff5c49-zzpgq -- ps aux | grep cortexflow-proxy

sleep 1.5

echo
echo "🔨 Testing curl command"
response=$(kubectl exec -it -n cortexflow cortexflow-proxy-84dfff5c49-zzpgq -- curl -s -o /dev/null -w "%{http_code}" http://localhost:9090/)
if [ "$response" -eq 200 ]; then
  echo "✅ Server is working"
  echo " Checking / endpoint"
  kubectl exec -it -n cortexflow cortexflow-proxy-84dfff5c49-zzpgq -- curl -o -v http://localhost:9090/

else
  echo "❌ Error in http response ERROR: $response"
  echo "❌ Service does not exists or is not exposed"
fi

echo
sleep 1.5
echo "🔨 Testing /health endpoint"
response=$(kubectl exec -it -n cortexflow cortexflow-proxy-84dfff5c49-zzpgq -- curl -s -o /dev/null -w "%{http_code}" http://localhost:9090/health)
if [ "$response" -eq 200 ]; then
  echo "✅ Server is working"
  echo " Checking /health endpoint"
  kubectl exec -it -n cortexflow cortexflow-proxy-84dfff5c49-zzpgq -- curl -o -v http://localhost:9090/health

else
  echo "❌ Error in http response ERROR: $response"
  echo "❌ Service does not exists or is not exposed"
fi

echo
sleep 1.5
echo "🔨 Testing /metrics endpoint"
response=$(kubectl exec -it -n cortexflow cortexflow-proxy-84dfff5c49-zzpgq -- curl -s -o /dev/null -w "%{http_code}" http://localhost:9090/metrics)
if [ "$response" -eq 200 ]; then
  echo "✅ Server is working"
  echo " Checking /metrics endpoint"
  kubectl exec -it -n cortexflow cortexflow-proxy-84dfff5c49-zzpgq -- curl -o -v http://localhost:9090/metrics
else
  echo "❌ Error in http response ERROR: $response"
  echo "❌ Service does not exists or is not exposed"
fi

echo
sleep 1.5
echo "🔨 Testing /status endpoint"
response=$(kubectl exec -it -n cortexflow cortexflow-proxy-84dfff5c49-zzpgq -- curl -s -o /dev/null -w "%{http_code}" http://localhost:9090/status)
if [ "$response" -eq 200 ]; then
  echo "✅ Server is working"
  echo " Checking /status endpoint"
  kubectl exec -it -n cortexflow cortexflow-proxy-84dfff5c49-zzpgq -- curl -o -v http://localhost:9090/status

else
  echo "❌ Error in http response ERROR: $response"
  echo "❌ Service does not exists or is not exposed"
fi
