echo "🛸 Copying kubernetes manifests as temporary files"
cp ../core/src/testing/proxy-injector.yaml proxy-injector.yaml
cp ../core/src/testing/admission-webhook.yaml admission-webhook.yaml
echo
echo "🛸 Installing the proxy-injector"
kubectl apply -f proxy-injector.yaml
sleep 2
echo "🛸 Installing the admission-webhook"
kubectl apply -f admission-webhook.yaml

echo "🛸 Removing temporary files"
rm -rf proxy-injector.yaml
rm -rf admission-webhook.yaml

sleep 3
echo "🛸 Installation completed"