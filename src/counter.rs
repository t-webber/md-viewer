use core::sync::atomic;

use actix_web::web::{self, Data};

use crate::AppState;

#[actix_web::get("/get")]
async fn get_counter(data: Data<AppState>) -> String {
    data.as_counter()
        .load(atomic::Ordering::Relaxed)
        .to_string()
}

#[actix_web::get("/incr")]
async fn incr_counter(data: Data<AppState>) -> String {
    data.as_counter().store(
        data.as_counter()
            .load(atomic::Ordering::Relaxed)
            .saturating_add(1),
        atomic::Ordering::Release,
    );
    data.as_counter()
        .load(atomic::Ordering::Acquire)
        .to_string()
}

#[actix_web::post("/set")]
async fn set_counter(data: Data<AppState>, req_body: String) -> String {
    req_body.parse::<i32>().map_or_else(
        |_| "Invalid counter".to_owned(),
        |value| {
            data.as_counter().store(value, atomic::Ordering::Relaxed);
            format!("Counter set to {value}")
        },
    )
}

pub fn counter_config(cfg: &mut web::ServiceConfig) {
    cfg.service(set_counter)
        .service(get_counter)
        .service(incr_counter);
}
