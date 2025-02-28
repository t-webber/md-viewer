use actix_web::web::{self, Data};

use crate::AppState;

#[actix_web::get("/get")]
async fn get_counter(data: Data<AppState>) -> String {
    match data.get_ref().counter.lock() {
        Ok(counter) => counter.to_string(),
        Err(_) => "Failed to get counter".to_owned(),
    }
}

#[actix_web::get("/incr")]
async fn incr_counter(data: Data<AppState>) -> String {
    match data.counter.lock() {
        Ok(mut counter) => {
            *counter += 1;
            "Counter incremented".to_owned()
        }
        Err(_) => "Failed to increment counter".to_owned(),
    }
}

#[actix_web::post("/set")]
async fn set_counter(data: Data<AppState>, req_body: String) -> String {
    match req_body.parse::<i32>() {
        Ok(value) => match data.counter.lock() {
            Ok(mut counter) => {
                *counter = value;
                format!("Counter set to {value}")
            }
            Err(_) => "Failed to set counter".to_owned(),
        },
        Err(_) => "Invalid counter".to_owned(),
    }
}

pub fn counter_config(cfg: &mut web::ServiceConfig) {
    cfg.service(set_counter)
        .service(get_counter)
        .service(incr_counter);
}
