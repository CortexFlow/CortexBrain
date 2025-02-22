echo "Copying kubernetes manifests as temporary files"
cp ../core/src/testing/configmap.yaml configmap.yaml
cp ../core/src/testing/configmap-role.yaml configmap-role.yaml
cp ../core/src/testing/dns-deployment.yaml dns-deployment.yaml
cp ../core/src/testing/proxy.yaml proxy.yaml

echo "installing Cortexflow components"
kubectl apply -f configmap.yaml
kubectl apply -f configmap-role.yaml
kubectl apply -f dns-deployment.yaml
kubectl apply -f proxy.yaml

echo "Removing temporary files"
rm -rf configmap.yaml
rm -rf configmap-role.yaml
rm -rf dns-deployment.yaml
rm -rf proxy.yaml