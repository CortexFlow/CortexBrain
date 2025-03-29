#!/bin/sh

./install-debugging-tools.sh test-proxy proxy-sidecar
./install-debugging-tools.sh test-proxy2 proxy-sidecar
./install-debugging-tools.sh test-proxy3 proxy-sidecar
./install-debugging-tools.sh test-proxy4 proxy-sidecar

# start the tcp listener
kubectl exec test-proxy -c proxy-sidecar -n cortexflow -- sh -c '
    echo "Starting TCP listener on port 5054..."
    nohup sh -c "nc -l -p 5054" >/dev/null 2>&1 &
'

kubectl exec test-proxy2 -c proxy-sidecar -n cortexflow -- sh -c '
    echo "Starting TCP listener on port 5054..."
    nohup sh -c "nc -l -p 5054" >/dev/null 2>&1 &
'


test_proxy_to_proxy2() {
    for i in $(seq 1 300); do
        sleep $((RANDOM % 5 + 1)) 
        kubectl exec test-proxy -c proxy-sidecar -n cortexflow -- sh -c '
            printf "{\"service\":\"test-proxy2.cortexflow\",\"direction\":\"Incoming\",\"payload\":\"eyJwYXlsb2FkIjogIkhlbGxvIGZyb20gcHJveHktc2lkZWNhciJ9\"}\n" | nc -w1 test-proxy2 5054
        '
    done
}

test_proxy2_to_proxy() {
    for i in $(seq 1 300); do
        sleep $((RANDOM % 5 + 1))
        kubectl exec test-proxy2 -c proxy-sidecar -n cortexflow -- sh -c '
            printf "{\"service\":\"test-proxy.cortexflow\",\"direction\":\"Incoming\",\"payload\":\"eyJwYXlsb2FkIjogIkhlbGxvIGZyb20gcHJveHktc2lkZWNhciJ9\"}\n" | nc -w1 test-proxy 5054
        '
    done
}

test_proxy3_to_proxy2() {
    for i in $(seq 1 300); do
        sleep $((RANDOM % 5 + 1))
        kubectl exec test-proxy3 -c proxy-sidecar -n cortexflow -- sh -c '
            printf "{\"service\":\"test-proxy2.cortexflow\",\"direction\":\"Incoming\",\"payload\":\"eyJwYXlsb2FkIjogIkhlbGxvIGZyb20gcHJveHktc2lkZWNhciJ9\"}\n" | nc -w1 test-proxy2 5054
        '
    done
}

test_proxy4_to_proxy2() {
    for i in $(seq 1 300); do
        sleep $((RANDOM % 5 + 1))
        kubectl exec test-proxy4 -c proxy-sidecar -n cortexflow -- sh -c '
            printf "{\"service\":\"test-proxy2.cortexflow\",\"direction\":\"Incoming\",\"payload\":\"eyJwYXlsb2FkIjogIkhlbGxvIGZyb20gcHJveHktc2lkZWNhciJ9\"}\n" | nc -w1 test-proxy2 5054
        '
    done
}

# execute the functions in background
test_proxy_to_proxy2 &
test_proxy2_to_proxy &
test_proxy3_to_proxy2 &
test_proxy4_to_proxy2 &


sleep 300

# stop the listeners
kubectl exec test-proxy -c proxy-sidecar -n cortexflow -- sh -c 'pkill nc'
kubectl exec test-proxy2 -c proxy-sidecar -n cortexflow -- sh -c 'pkill nc'
