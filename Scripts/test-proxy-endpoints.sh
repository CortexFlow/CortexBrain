echo "🔨 Testing curl command"
response=$(kubectl exec -n cortexflow $1 -- curl -s -o /dev/null -w "%{http_code}" http://localhost:9090/)
if [ "$response" -eq 200 ]; then
  echo "✅ Server is working"
  echo " Checking / endpoint"
  kubectl exec -n cortexflow $1 -- curl -v http://localhost:9090/
else
  echo "❌ Error in http response ERROR: $response. Service does not exists or is not exposed"
fi

echo
sleep 1.5
echo "🔨 Testing /health endpoint"
response=$(kubectl exec -n cortexflow $1 -- curl -s -o /dev/null -w "%{http_code}" http://localhost:9090/health)
if [ "$response" -eq 200 ]; then
  echo "✅ Server is working"
  echo " Checking /health endpoint"
  kubectl exec -n cortexflow $1 -- curl -v http://localhost:9090/health
else
  echo "❌ Error in http response ERROR: $response. Service does not exists or is not exposed"
fi

echo
sleep 1.5
echo "🔨 Testing /metrics endpoint"
response=$(kubectl exec -n cortexflow $1 -- curl -s -o /dev/null -w "%{http_code}" http://localhost:9090/metrics)
if [ "$response" -eq 200 ]; then
  echo "✅ Server is working"
  echo " Checking /metrics endpoint"
  kubectl exec -n cortexflow $1 -- curl -v http://localhost:9090/metrics
else
  echo "❌ Error in http response ERROR: $response. Service does not exists or is not exposed"
fi

echo
sleep 1.5
echo "🔨 Testing /status endpoint"
response=$(kubectl exec -n cortexflow $1 -- curl -s -o /dev/null -w "%{http_code}" http://localhost:9090/status)
if [ "$response" -eq 200 ]; then
  echo "✅ Server is working"
  echo " Checking /status endpoint"
  kubectl exec -n cortexflow $1 -- curl -v http://localhost:9090/status
else
  echo "❌ Error in http response ERROR: $response. Service does not exists or is not exposed"
fi
