echo "🔨 Testing network connections"
kubectl exec -n cortexflow $1 -- netstat -tulnp | grep $2

sleep 1.5

echo
echo "🔨 testing if the process is in execution"
kubectl exec -n cortexflow $1 -- ps aux | grep cortexflow-proxy

sleep 1.5
echo
echo "🔨 testing using netcat"
kubectl exec -n cortexflow $1 -- nc -zv proxy-service.cortexflow.svc.cluster.local $2

sleep 1.5
echo
echo "🔨 Checking if the proxy is listening in the 5053 port"
kubectl exec -n cortexflow $1 -- netstat -ulnp
