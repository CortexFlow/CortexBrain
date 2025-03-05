echo "deleting components"
kubectl delete deployment cortexflow-dns -n cortexflow
kubectl delete deployment cortexflow-proxy -n cortexflow
echo "deleting associated services"
kubectl delete svc cortexflow-dns-service -n cortexflow
kubectl delete svc proxy-service -n cortexflow

echo "deployment and services deleted"
kubectl get deployment -n cortexflow
kubectl get svc -n cortexflow

echo "deleting cortexflow namespace"
kubectl delete namespace cortexflow
