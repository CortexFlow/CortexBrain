use lazy_static::lazy_static;
use serde_json::Value;

//container patch
lazy_static! {
    pub static ref PATCH: Value = {
        let json_str = r#"[
            {
                "op": "add",
                "path": "/spec/containers/-",
                "value": {
                    "name": "proxy-sidecar",
                    "image": "lorenzotettamanti/cortexflow-proxy:latest"
                }
            }
        ]"#;
        serde_json::from_str(json_str).unwrap()
    };
}
