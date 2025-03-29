#!/bin/sh
./install-debugging-tools.sh test-proxy proxy-sidecar
./install-debugging-tools.sh test-proxy2 proxy-sidecar
./install-debugging-tools.sh test-proxy3 proxy-sidecar
./install-debugging-tools.sh test-proxy4 proxy-sidecar

# start the udp listener
kubectl exec test-proxy -c proxy-sidecar -n cortexflow -- sh -c '
    echo "Starting UDP listener on port 5053..."
    nohup nc -lu 5053 >/dev/null 2>&1 &
'

kubectl exec test-proxy2 -c proxy-sidecar -n cortexflow -- sh -c '
    echo "Starting UDP listener on port 5053..."
    nohup nc -lu 5053 >/dev/null 2>&1 &
'


test_proxy_to_proxy2() {
    for i in $(seq 1 300); do
        sleep $((RANDOM % 5 + 1))  
        echo "Sending UDP packet from test-proxy to test-proxy2..."
        kubectl exec test-proxy -c proxy-sidecar -n cortexflow -- sh -c '
            printf "{\"service\":\"test-proxy2.cortexflow\",\"direction\":\"Incoming\",\"payload\":\"eyJwYXlsb2FkIjogIkhlbGxvIGZyb20gcHJveHktc2lkZWNhciJ9\"}\n" | nc -u -w1 test-proxy2 5053
        '
    done
}

test_proxy2_to_proxy() {
    for i in $(seq 1 300); do
        sleep $((RANDOM % 5 + 1))
        echo "Sending UDP packet from test-proxy2 to test-proxy..."
        kubectl exec test-proxy2 -c proxy-sidecar -n cortexflow -- sh -c '
            printf "{\"service\":\"test-proxy.cortexflow\",\"direction\":\"Incoming\",\"payload\":\"eyJwYXlsb2FkIjogIkhlbGxvIGZyb20gcHJveHktc2lkZWNhciJ9\"}\n" | nc -u -w1 test-proxy 5053
        '
    done
}

test_proxy3_to_proxy2() {
    for i in $(seq 1 300); do
        sleep $((RANDOM % 5 + 1))
        echo "Sending UDP packet from test-proxy3 to test-proxy2..."
        kubectl exec test-proxy3 -c proxy-sidecar -n cortexflow -- sh -c '
            printf "{\"service\":\"test-proxy2.cortexflow\",\"direction\":\"Incoming\",\"payload\":\"eyJwYXlsb2FkIjogIkhlbGxvIGZyb20gcHJveHktc2lkZWNhciJ9\"}\n" | nc -u -w1 test-proxy2 5053
        '
    done
}

test_proxy4_to_proxy2() {
    for i in $(seq 1 300); do
        sleep $((RANDOM % 5 + 1))
        echo "Sending UDP packet from test-proxy4 to test-proxy2..."
        kubectl exec test-proxy4 -c proxy-sidecar -n cortexflow -- sh -c '
            printf "{\"service\":\"test-proxy2.cortexflow\",\"direction\":\"Incoming\",\"payload\":\"eyJwYXlsb2FkIjogIkhlbGxvIGZyb20gcHJveHktc2lkZWNhciJ9\"}\n" | nc -u -w1 test-proxy2 5053
        '
    done
}

# execute the functions in background
(test_proxy_to_proxy2 &) &
(test_proxy2_to_proxy &) &
(test_proxy3_to_proxy2 &) &
(test_proxy4_to_proxy2 &) &


sleep 300

# stop the listeners
kubectl exec test-proxy -c proxy-sidecar -n cortexflow -- sh -c 'pkill nc || kill $(pgrep nc)'
kubectl exec test-proxy2 -c proxy-sidecar -n cortexflow -- sh -c 'pkill nc || kill $(pgrep nc)'
