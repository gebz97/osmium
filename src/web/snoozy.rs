use std::thread;

use axum::response::IntoResponse;

pub async fn snooze() -> impl IntoResponse {
    let result = thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_secs(5));
        "Done Snooooooooozing!".to_string()
    })
    .join()
    .unwrap();
    result
}