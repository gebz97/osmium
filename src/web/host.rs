use crate::hostmgr::model::hosts::{ActiveModel, Entity, Model};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, DatabaseConnection, EntityTrait, IntoActiveModel, Set,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct HostResponse {
    pub id: i32,
    pub hostname: String,
    pub fqdn: Option<String>,
    pub physical: Option<bool>,
    pub status: Option<String>,
    pub os_name: Option<String>,
    pub os_version: Option<String>,
    pub os_major: Option<String>,
    pub kernel_version: Option<String>,
    pub ip_address: Option<String>,
    pub subnet: Option<String>,
    pub environment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateHostRequest {
    pub hostname: String,
    pub fqdn: Option<String>,
    pub physical: Option<bool>,
    pub status: Option<String>,
    pub os_name: Option<String>,
    pub os_version: Option<String>,
    pub os_major: Option<String>,
    pub kernel_version: Option<String>,
    pub ip_address: Option<String>,
    pub subnet: Option<String>,
    pub environment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateHostRequest {
    pub fqdn: Option<String>,
    pub physical: Option<bool>,
    pub status: Option<String>,
    pub os_name: Option<String>,
    pub os_version: Option<String>,
    pub os_major: Option<String>,
    pub kernel_version: Option<String>,
    pub ip_address: Option<String>,
    pub subnet: Option<String>,
    pub environment: Option<String>,
}

// Validation functions for PostgreSQL inet and cidr types
fn is_valid_inet(ip: &str) -> bool {
    // Basic validation for inet format (IP address, optionally with mask)
    ip.parse::<std::net::IpAddr>().is_ok()
        || ip
            .split('/')
            .next()
            .and_then(|s| s.parse::<std::net::IpAddr>().ok())
            .is_some()
}

fn is_valid_cidr(cidr: &str) -> bool {
    // Basic validation for CIDR format (IP/mask)
    if !cidr.contains('/') {
        return false;
    }
    let parts: Vec<&str> = cidr.split('/').collect();
    if parts.len() != 2 {
        return false;
    }

    // Check if IP part is valid
    if parts[0].parse::<std::net::IpAddr>().is_err() {
        return false;
    }

    // Check if mask part is a valid number (0-32 for IPv4, 0-128 for IPv6)
    if let Ok(mask) = parts[1].parse::<u8>() {
        if parts[0].parse::<std::net::Ipv4Addr>().is_ok() {
            return mask <= 32;
        } else if parts[0].parse::<std::net::Ipv6Addr>().is_ok() {
            return mask <= 128;
        }
    }

    false
}

// Helper function to convert Model to HostResponse
fn model_to_response(host: Model) -> HostResponse {
    HostResponse {
        id: host.id,
        hostname: host.hostname,
        fqdn: host.fqdn,
        physical: host.physical,
        status: host.status,
        os_name: host.os_name,
        os_version: host.os_version,
        os_major: host.os_major,
        kernel_version: host.kernel_version,
        ip_address: host.ip_address,
        subnet: host.subnet,
        environment: host.environment,
    }
}

/// GET /hosts - Retrieve all hosts
pub async fn get_hosts_handler(State(db): State<Arc<DatabaseConnection>>) -> impl IntoResponse {
    let hosts = match Entity::find().all(db.as_ref()).await {
        Err(e) => {
            eprintln!("Error fetching hosts: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Vec::<HostResponse>::new()),
            )
                .into_response();
        }
        Ok(res) => res,
    };

    if hosts.is_empty() {
        return (StatusCode::NO_CONTENT, Json(Vec::<HostResponse>::new())).into_response();
    }

    let responses: Vec<HostResponse> = hosts.into_iter().map(model_to_response).collect();
    (StatusCode::OK, Json(responses)).into_response()
}

/// GET /hosts/{id} - Retrieve a specific host by ID
pub async fn get_host_by_id_handler(
    State(db): State<Arc<DatabaseConnection>>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match Entity::find_by_id(id).one(db.as_ref()).await {
        Err(e) => {
            eprintln!("Error fetching host: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Internal server error"})),
            )
                .into_response()
        }
        Ok(Some(host)) => (StatusCode::OK, Json(model_to_response(host))).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Host not found"})),
        )
            .into_response(),
    }
}

/// POST /hosts - Create a new host
pub async fn create_host_handler(
    State(db): State<Arc<DatabaseConnection>>,
    Json(req): Json<CreateHostRequest>,
) -> impl IntoResponse {
    // Validate inet format if provided
    if let Some(ref ip) = req.ip_address {
        if !is_valid_inet(ip) {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": format!("Invalid IP address format: {}", ip)})),
            )
                .into_response();
        }
    }

    // Validate cidr format if provided
    if let Some(ref subnet) = req.subnet {
        if !is_valid_cidr(subnet) {
            return (
                StatusCode::BAD_REQUEST,
                Json(
                    serde_json::json!({"error": format!("Invalid CIDR subnet format: {}", subnet)}),
                ),
            )
                .into_response();
        }
    }

    let active_model = ActiveModel {
        hostname: Set(req.hostname),
        fqdn: Set(req.fqdn),
        physical: Set(req.physical),
        status: Set(req.status),
        os_name: Set(req.os_name),
        os_version: Set(req.os_version),
        os_major: Set(req.os_major),
        kernel_version: Set(req.kernel_version),
        ip_address: Set(req.ip_address),
        subnet: Set(req.subnet),
        environment: Set(req.environment),
        ..Default::default()
    };

    match active_model.insert(db.as_ref()).await {
        Err(e) => {
            eprintln!("Error creating host: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": format!("Failed to create host: {}", e)})),
            )
                .into_response()
        }
        Ok(host) => (StatusCode::CREATED, Json(model_to_response(host))).into_response(),
    }
}

/// PUT /hosts/{id} - Update an existing host
pub async fn update_host_handler(
    State(db): State<Arc<DatabaseConnection>>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateHostRequest>,
) -> impl IntoResponse {
    // Validate inet format if provided
    if let Some(ref ip) = req.ip_address {
        if !is_valid_inet(ip) {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": format!("Invalid IP address format: {}", ip)})),
            )
                .into_response();
        }
    }

    // Validate cidr format if provided
    if let Some(ref subnet) = req.subnet {
        if !is_valid_cidr(subnet) {
            return (
                StatusCode::BAD_REQUEST,
                Json(
                    serde_json::json!({"error": format!("Invalid CIDR subnet format: {}", subnet)}),
                ),
            )
                .into_response();
        }
    }

    // First, find the existing host
    let host = match Entity::find_by_id(id).one(db.as_ref()).await {
        Err(e) => {
            eprintln!("Error fetching host: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Internal server error"})),
            )
                .into_response();
        }
        Ok(Some(h)) => h,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Host not found"})),
            )
                .into_response();
        }
    };

    // Convert to ActiveModel and update fields
    let mut active_model = host.into_active_model();

    if let Some(fqdn) = req.fqdn {
        active_model.fqdn = Set(Some(fqdn));
    }
    if let Some(physical) = req.physical {
        active_model.physical = Set(Some(physical));
    }
    if let Some(status) = req.status {
        active_model.status = Set(Some(status));
    }
    if let Some(os_name) = req.os_name {
        active_model.os_name = Set(Some(os_name));
    }
    if let Some(os_version) = req.os_version {
        active_model.os_version = Set(Some(os_version));
    }
    if let Some(os_major) = req.os_major {
        active_model.os_major = Set(Some(os_major));
    }
    if let Some(kernel_version) = req.kernel_version {
        active_model.kernel_version = Set(Some(kernel_version));
    }
    if let Some(ip_address) = req.ip_address {
        active_model.ip_address = Set(Some(ip_address));
    }
    if let Some(subnet) = req.subnet {
        active_model.subnet = Set(Some(subnet));
    }
    if let Some(environment) = req.environment {
        active_model.environment = Set(Some(environment));
    }

    match active_model.update(db.as_ref()).await {
        Err(e) => {
            eprintln!("Error updating host: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": format!("Failed to update host: {}", e)})),
            )
                .into_response()
        }
        Ok(host) => (StatusCode::OK, Json(model_to_response(host))).into_response(),
    }
}

/// DELETE /hosts/{id} - Delete a host
pub async fn delete_host_handler(
    State(db): State<Arc<DatabaseConnection>>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    // First, check if the host exists
    match Entity::find_by_id(id).one(db.as_ref()).await {
        Err(e) => {
            eprintln!("Error fetching host: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Internal server error"})),
            )
                .into_response();
        }
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Host not found"})),
            )
                .into_response();
        }
        Ok(Some(_)) => {}
    }

    // Delete the host
    match Entity::delete_by_id(id).exec(db.as_ref()).await {
        Err(e) => {
            eprintln!("Error deleting host: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Failed to delete host: {}", e)})),
            )
                .into_response()
        }
        Ok(_) => (StatusCode::NO_CONTENT, "").into_response(),
    }
}
