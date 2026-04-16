use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use futures::future::join_all;
use serde::{Deserialize, Serialize};

use crate::hostmgr::scanner;

#[derive(Serialize, Deserialize, Debug)]
pub struct ScanRequest {
    hosts: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ScanResponse {
    results: Vec<HostScanResult>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HostScanResult {
    host: String,
    status: String,
    response_time: Option<u32>,
}

pub async fn scan_handler(Json(req): Json<ScanRequest>) -> impl IntoResponse {
    let futures = req.hosts.iter().map(|host| scanner::scan_single_host(host));

    let scan_results = join_all(futures).await;

    let results = req
        .hosts
        .into_iter()
        .zip(scan_results)
        .map(|(host, result)| match result {
            scanner::ScanResult::Success { response_time } => HostScanResult {
                host,
                status: "success".to_string(),
                response_time: Some(response_time),
            },
            scanner::ScanResult::Failure { message } => HostScanResult {
                host,
                status: format!("failure: {}", message),
                response_time: None,
            },
        })
        .collect();

    let response = ScanResponse { results };
    (StatusCode::OK, Json(response))
}
