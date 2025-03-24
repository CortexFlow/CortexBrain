/*
Contains container patch with features:
-init container with iptables prerouting
    - redirect all the tcp traffic to the 5054 port
    - redirect all the udp traffic to the 5053 port
- proxy sidecar image injection

*/
use lazy_static::lazy_static;
use serde_json::Value;

// Container patch
lazy_static! {
    pub static ref PATCH: Value = {
        let json_str = r#"[
            {
                "op": "add",
                "path": "/spec/initContainers",
                "value": [
                    {
                        "name": "init-iptables",
                        "image": "ubuntu",
                        "securityContext": {
                            "capabilities": {
                                "add": ["NET_ADMIN"]
                            }
                        },
                        "command": [
                            "/bin/sh", "-c",
                            "apt-get update && apt-get install -y iptables && iptables -t nat -A PREROUTING -p tcp -j REDIRECT --to-port 5054 && iptables -t nat -A PREROUTING -p udp -j REDIRECT --to-port 5053"
                        ]
                    }
                ]
            },
            {
                "op": "add",
                "path": "/spec/containers/-",
                "value": {
                    "name": "proxy-sidecar",
                    "image": "lorenzotettamanti/cortexflow-proxy:latest",
                    "ports": [
                        {
                            "containerPort": 5054,
                            "protocol": "TCP"
                        },
                        {
                            "containerPort": 5053,
                            "protocol": "UDP"
                        }
                    ]
                }
            }
        ]"#;
        serde_json::from_str(json_str).unwrap()
    };
}
