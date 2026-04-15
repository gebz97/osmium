use axum::{Router, routing::{get, post}};

use crate::common::config::{AppConfig, AppState};

// pub async fn build_router(conf: &AppConfig) -> Result<Router, String> {
//     let state = AppState {
//         config: std::sync::Arc::new(conf.clone()),
//     };

//     let router = Router::new()
//         .route("/status", get(app_status))
//         // .route("/", post())
//         .with_state(state);

//     Ok(router)
// }

pub async fn app_status() -> String {
    return String::from("100% fam..")
}