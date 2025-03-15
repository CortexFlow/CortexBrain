/* contains the injection function */
/*
    Wiki:
    the proxy injector is an admission controller. An admission controller is a piece of code that intercepts requests to the kubernetes API,
    prior to persistence of the resource,but after the request is authenticated and authorized.

    Admission Request
    Admission Response
    Admission Review
    AdmissionReviewResponse

*/
//TODO: add injection to stateful sets, deployments and daemonset
//TODO: better pod checks
//TODO: better TLS certificates handling
//TODO: json dynamic generation

use crate::validation::check_and_validate_pod;
use axum::{Json, Router, extract::State, routing::post};
use axum_server::tls_rustls::RustlsConfig;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tracing::{error, info, instrument, warn};

// import the patch
use crate::vars::PATCH;

#[derive(Debug, Serialize, Deserialize)]
pub struct AdmissionRequest {
    uid: String,
    object: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AdmissionReview {
    #[serde(rename = "apiVersion", default = "default_api_version")]
    pub api_version: String,
    #[serde(default = "default_kind")]
    pub kind: String,
    pub request: AdmissionRequest,
    #[serde(skip_deserializing)]
    pub response: Option<AdmissionResponse>,
}

fn default_api_version() -> String {
    "admission.k8s.io/v1".to_string()
}

fn default_kind() -> String {
    "AdmissionReview".to_string()
}

#[derive(Debug, Serialize)]
pub struct AdmissionResponse {
    uid: String,
    allowed: bool,
    patch: Option<String>,
    #[serde(rename = "patchType")]
    patch_type: Option<String>,
}

#[instrument]
pub async fn inject(
    State(_state): State<Arc<()>>,
    Json(mut admission_review): Json<AdmissionReview>,
) -> Json<AdmissionReview> {
    let pod = &admission_review.request.object;

    // Log the received pod metadata for debugging
    info!("Pod metadata: {:?}", pod["metadata"]);
    info!("Pod annotations: {:?}", pod["metadata"]["annotations"]);

    // Injection logic
    let response = if check_and_validate_pod(pod).unwrap_or(false) {
        info!("Starting the proxy injector");

        let patch_string = serde_json::to_string(&*PATCH).unwrap();
        let patch_encoded = STANDARD.encode(&patch_string);

        info!("Patch to be applied: {}", &patch_string);

        AdmissionResponse {
            uid: admission_review.request.uid.clone(),
            allowed: true,
            patch: Some(patch_encoded),
            patch_type: Some("JSONPatch".to_string()),
        }
    } else {
        AdmissionResponse {
            uid: admission_review.request.uid.clone(),
            allowed: false,
            patch: None,
            patch_type: None,
        }
    };

    admission_review.response = Some(response);
    admission_review.api_version = "admission.k8s.io/v1".to_string();
    admission_review.kind = "AdmissionReview".to_string();

    Json(admission_review)
}

pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/mutate", post(inject))
        .with_state(Arc::new(()));

    let addr = SocketAddr::from(([0, 0, 0, 0], 9443));
    let config = RustlsConfig::from_pem_file(
        Path::new("/etc/webhook/certs/..data/tls.crt"),
        Path::new("/etc/webhook/certs/..data/tls.key"),
    )
    .await
    .map_err(|e| {
        error!("Failed to load TLS config: {}", e);
        e
    })?;

    info!("HTTPS server listening on {}", addr);

    if let Err(e) = axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service())
        .await
    {
        error!("Server error: {}", e);
        return Err(Box::new(e));
    }

    Ok(())
}
