use std::net::{IpAddr, Ipv4Addr};
use tokio::time::Duration;

pub enum ScanResult {
    Success { response_time: u32 },
    Failure { message: String },
}

pub async fn scan_single_host(host: &str) -> ScanResult {
    let ip_addr: IpAddr = match host.parse() {
        Ok(addr) => addr,
        Err(_) => {
            return ScanResult::Failure {
                message: "Invalid IP address".to_string(),
            };
        }
    };

    match tokio::task::spawn_blocking(move || match ping::new(ip_addr).send() {
        Ok(response) => match response.rtt.as_micros().try_into() {
            Ok(rtt) => ScanResult::Success { response_time: rtt },
            Err(_) => ScanResult::Failure {
                message: "Response time out of range".to_string(),
            },
        },
        Err(_) => ScanResult::Failure {
            message: "Ping failed".to_string(),
        },
    })
    .await
    {
        Ok(result) => result,
        Err(_) => ScanResult::Failure {
            message: "Spawn blocking task failed".to_string(),
        },
    }
}
