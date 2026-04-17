use std::sync::Arc;
use std::time::Duration;

use axum::routing::post;
use axum::{Router, routing::get};
use sea_orm::{ConnectOptions, Database};

use crate::common::config::AppConfig;
use crate::web::host::*;

pub mod common;
pub mod hostmgr;
pub mod web;

fn main() -> Result<(), String> {
    let conf = AppConfig::default();
    // let thr = std::thread::available_parallelism().map_err(|e| {
    //     eprintln!("{}", e);
    //     String::from("Unable to get thread count")
    // })?;

    let mut opt = ConnectOptions::new("postgresql://osmium:osmium@pg.gebz.local/osmium");
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(false)
        .sqlx_logging_level(log::LevelFilter::Info)
        .set_schema_search_path("hostmgr");

    let rt = tokio::runtime::Builder::new_multi_thread()
        // .worker_threads(thr.get().saturating_sub(2) &conf.thread_count)
        .worker_threads(conf.thread_count)
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;

    match rt.block_on(async {
        let db = match Database::connect(opt).await {
            Ok(db) => db,
            Err(e) => {
                eprintln!("{}", e);
                return Err("Unable to connect to database");
            }
        };
        let router = Router::new()
            .route("/status", get(web::router::app_status))
            // .route("/snooze", get(web::snoozy::snooze))
            // .route("/spoopy", get(web::router::spoopy_handler))
            .route("/scan", post(web::scan::scan_handler))
            .route("/hosts", get(web::host::get_hosts_handler).post(create_host_handler))
            .route(
                "/hosts/{id}",
                get(get_host_by_id_handler)
                    .put(update_host_handler)
                    .delete(delete_host_handler),
            )
            .with_state(Arc::new(db));

        let listener = tokio::net::TcpListener::bind(&conf.bind_addr);
        // your async entrypoint here
        let listener = listener.await.unwrap();
        // axum::serve(listener, router).await.unwrap();
        if let Err(e) = axum::serve(listener, router).await {
            eprint!("Error: {}", e);
        };
        // Keep runtime alive
        // std::future::pending::<()>().await;
        Ok(())
    }) {
        Ok(_) => return Ok(()),
        Err(e) => {
            eprintln!("{}", e);
            return Err(String::from("Runtime Error"));
        }
    };
}
