echo "Copying kubernetes manifests as temporary files"
cp ../core/src/testing/configmap.yaml configmap.yaml
cp ../core/src/testing/configmap-role.yaml configmap-role.yaml
cp ../core/src/testing/rolebinding.yaml rolebinding.yaml 
cp ../core/src/testing/dns-deployment.yaml dns-deployment.yaml
cp ../core/src/testing/proxy.yaml proxy.yaml
cp ../core/src/testing/cortexflow-rolebinding.yaml cortexflow-rolebinding.yaml


echo "creating Cortexflow namespace"
kubectl create namespace cortexflow

echo "installing Cortexflow components"
kubectl apply -f configmap.yaml -n cortexflow
kubectl apply -f configmap-role.yaml -n default
kubectl apply -f rolebinding.yaml -n kube-system
kubectl apply -f cortexflow-rolebinding.yaml -n cortexflow
kubectl apply -f dns-deployment.yaml -n cortexflow
kubectl apply -f proxy.yaml -n cortexflow

echo "Removing temporary files"
rm -rf configmap.yaml
rm -rf configmap-role.yaml
rm -rf rolebinding.yaml
rm -rf cortexflow-rolebinding.yaml
rm -rf dns-deployment.yaml
rm -rf proxy.yaml

sleep 5

echo "installation completed"
kubectl get pods -n cortexflow
kubectl get svc -n cortexflow