if ! kubectl exec -n cortexflow $1 -c $2 -- which netstat >/dev/null 2>&1; then
    echo "🔨 installing netstat"
    kubectl exec -n cortexflow $1 -c $2 -- apt update
    kubectl exec -n cortexflow $1 -c $2 -- apt install -y net-tools
else
    echo "✅ Netstat is installed."
fi

sleep 1.5

if ! kubectl exec -n cortexflow $1 -c $2 -- which nc >/dev/null 2>&1; then
    echo "🔨 installing netcat"
    kubectl exec -n cortexflow $1 -c $2 -- apt install -y netcat-traditional
else
    echo "✅ Netcat is installed."
fi

sleep 1.5

if ! kubectl exec -n cortexflow $1 -c $2 -- which curl >/dev/null 2>&1; then
    echo "🔨 installing curl"
    kubectl exec -n cortexflow $1 -c $2 -- apt install -y curl
else
    echo "✅ Curl is installed."
fi

sleep 1.5

if ! kubectl exec -n cortexflow $1 -c $2 -- which nslookup >/dev/null 2>&1; then
    echo "🔨 installing dnsutils"
    kubectl exec -n cortexflow $1 -c $2 -- apt install -y dnsutils
else
    echo "✅ Nslookup is installed."
fi

sleep 1.5

if ! kubectl exec -n cortexflow $1 -c $2 -- which tcpdump >/dev/null 2>&1; then
    echo "🔨 installing tcpdump" 
    kubectl exec -n cortexflow $1 -c $2 -- apt install -y tcpdump
else
    echo "✅ tcpdump is installed."
fi

sleep 1.5
