use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::lookup_host;

pub struct Host {
    pub target: String,
    pub ipv4: Option<Ipv4Addr>,
    pub port: u16,
    pub creds: HostCredential,
}

pub enum HostParseError {
    InvalidTarget(String), // not an IP and resolution failed
    ResolutionFailed(std::io::Error),
    NoIpv4Found,
}

impl Host {
    pub async fn new_with_userpass(
        target: &str,
        port: Option<u16>,
        username: &str,
        password: &str,
    ) -> Result<Self, HostParseError> {
        let port = port.unwrap_or(22);

        // 1. Try direct IPv4 parse
        let ipv4 = if let Ok(ip) = target.parse::<Ipv4Addr>() {
            ip
        } else {
            // 2. Fallback to DNS resolution
            let addrs = lookup_host((target, port))
                .await
                .map_err(HostParseError::ResolutionFailed)?;

            addrs
                .filter_map(|addr| match addr {
                    SocketAddr::V4(v4) => Some(*v4.ip()),
                    _ => None,
                })
                .next()
                .ok_or(HostParseError::NoIpv4Found)?
        };

        Ok(Self {
            target: target.to_string(),
            ipv4: Some(ipv4),
            port,
            creds: HostCredential::UserPass {
                username: username.to_string(),
                password: password.to_string(),
            },
        })
    }
}

enum HostCredential {
    PrivateKey { key_path: String },
    UserPass { username: String, password: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum HostScanState {
    Pending,
    Running { started_at: i64 },

    Completed { finished_at: i64 },

    Failed(HostScanError),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum HostScanError {
    // Connectivity / reachability
    Unreachable,
    DnsResolutionFailed,
    ConnectionRefused,
    ConnectionTimedOut,

    // Protocol / transport
    SshError(String),
    HttpError(u16),

    // System / OS-level
    PermissionDenied,
    CommandFailed { cmd: String, code: Option<i32> },

    // Data / parsing
    InvalidResponse,
    ParseError(String),

    // Resource constraints
    Timeout,
    ResourceExhausted,

    // Internal / unexpected
    Internal(String),
}
