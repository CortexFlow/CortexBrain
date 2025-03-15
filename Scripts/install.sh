echo "🛸 Copying kubernetes manifests as temporary files"
cp ../core/src/testing/configmap.yaml configmap.yaml
cp ../core/src/testing/configmap-role.yaml configmap-role.yaml
cp ../core/src/testing/rolebinding.yaml rolebinding.yaml 
cp ../core/src/testing/dns-deployment.yaml dns-deployment.yaml
cp ../core/src/testing/proxy.yaml proxy.yaml
cp ../core/src/testing/cortexflow-rolebinding.yaml cortexflow-rolebinding.yaml
cp ../core/src/testing/coredns-rolebinding.yaml coredns-rolebinding.yaml
cp ../core/src/testing/coredns-clusterrole.yaml coredns-clusterrole.yaml
cp ../core/src/testing/coredns-config.yaml coredns-config.yaml
cp ../core/src/testing/certificate-manager.yaml certificate-manager.yaml

echo "🛸 creating Cortexflow namespace"
kubectl create namespace cortexflow
echo "🛸 installing cert-manager"
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/latest/download/cert-manager.yaml
echo
echo "🛸 wait until cert-manager installation is completed.It may takes a while"
sleep 100
echo
kubectl get pods -n cert-manager

sleep 10
echo
echo "🛸 installing Cortexflow components"
kubectl apply -f configmap.yaml -n cortexflow
kubectl apply -f configmap-role.yaml -n default
kubectl apply -f rolebinding.yaml -n kube-system
kubectl apply -f cortexflow-rolebinding.yaml -n cortexflow
kubectl apply -f coredns-clusterrole.yaml
kubectl apply -f coredns-rolebinding.yaml -n cortexflow
kubectl apply -f coredns-config.yaml -n kube-system
kubectl apply -f proxy.yaml -n cortexflow

echo
echo "🛸 creating the issuer"
kubectl apply -f certificate-manager.yaml
echo
echo "🛸 caBundle certificate"
kubectl get secret proxy-injector-tls -n cortexflow -o jsonpath='{.data.ca\.crt}'
echo "🛸 tls.key code"
kubectl get secret proxy-injector-tls -n cortexflow -o jsonpath='{.data.tls\.key}'
echo
echo "🛸 tls.cert code"
kubectl get secret proxy-injector-tls -n cortexflow -o jsonpath='{.data.tls\.crt}'
echo
echo
echo "🛸 Insert the caBundle, tls.key and tls.cert in the install-injector.sh script"
echo "🛸 Make sure to exacly copy the codes otherwise the proxy-injector will not work! "
echo
echo "🛸 Removing temporary files"
rm -rf configmap.yaml
rm -rf configmap-role.yaml
rm -rf rolebinding.yaml
rm -rf cortexflow-rolebinding.yaml
rm -rf dns-deployment.yaml
rm -rf proxy.yaml
rm -rf coredns-rolebinding.yaml
rm -rf coredns-clusterrole.yaml
rm -rf coredns-config.yaml
rm -rf certificate-manager.yaml

sleep 5

echo "🛸 installation completed"
kubectl get pods -n cortexflow
kubectl get svc -n cortexflow

echo "🛸 To install the proxy injector please run install-injector script"