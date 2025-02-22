echo "deleting components"
kubectl delete deployment cortexflow-dns
kubectl delete deployment cortexflow-proxy
echo "deleting associated services"
kubectl delete svc cortexflow-dns-service
kubectl delete svc proxy-service