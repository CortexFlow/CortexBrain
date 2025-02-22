echo "installing Cortexflow components"
kubectl apply -f configmap.yaml
kubectl apply -f configmap-role.yaml
kubectl apply -f dns-deployment.yaml
kubectl apply -f proxy.yaml