use core::sync::atomic;

use actix_web::web::{self, Data};

use crate::AppState;

#[actix_web::get("/get")]
async fn get_counter(data: Data<AppState>) -> String {
    data.counter.load(atomic::Ordering::Relaxed).to_string()
}

#[actix_web::get("/incr")]
async fn incr_counter(data: Data<AppState>) -> String {
    let old = data.counter.load(atomic::Ordering::Relaxed);
    data.counter
        .store(old.saturating_add(1), atomic::Ordering::Release);
    data.counter.load(atomic::Ordering::Acquire).to_string()
}

#[actix_web::post("/set")]
async fn set_counter(data: Data<AppState>, req_body: String) -> String {
    match req_body.parse::<i32>() {
        Ok(value) => {
            data.counter.store(value, atomic::Ordering::Relaxed);
            format!("Counter set to {value}")
        }
        Err(_) => "Invalid counter".to_owned(),
    }
}

pub fn counter_config(cfg: &mut web::ServiceConfig) {
    cfg.service(set_counter)
        .service(get_counter)
        .service(incr_counter);
}
