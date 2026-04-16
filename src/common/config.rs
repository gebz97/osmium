use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub thread_count: usize,
    pub bind_addr: String,
}

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self{
            thread_count: 4,
            bind_addr: String::from("0.0.0.0:8080"),
        }
    }
}