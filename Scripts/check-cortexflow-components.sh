echo "Welcome to CortexFlow tools"
echo "Checking CortexFlow components"

echo "Checking if CortexFlow namespace exists..."
if kubectl get namespace cortexflow >/dev/null 2>&1; then
    echo "✅ Namespace 'cortexflow' exists."
    
    sleep 1
    echo "Checking pods..."
    kubectl get pods -n cortexflow
    
    echo

    sleep 1
    echo "Checking services..."
    kubectl get svc -n cortexflow
    echo 
else
    echo "❌ Namespace 'cortexflow' does not exist."
    exit 1
fi
