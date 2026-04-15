use axum::{Router, routing::get};

use crate::common::config::AppConfig;

pub mod common;
pub mod hostmgr;
pub mod web;

fn main() -> Result<(), String> {
    let conf = AppConfig::default();
    let thr = std::thread::available_parallelism().map_err(|e| {
        eprintln!("{}", e);
        String::from("Unable to get thread count")
    })?;

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(thr.get().saturating_sub(2))
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;

    let router = Router::new().route("/status", get(web::router::app_status));
    let listener = tokio::net::TcpListener::bind(&conf.bind_addr);
    rt.block_on(async {
        // your async entrypoint here
        let listener = listener.await.unwrap();
        axum::serve(listener, router).await.unwrap();
    });

    Ok(())
}
