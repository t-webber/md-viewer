use std::{env::set_var, sync::Mutex};

use actix_web::{
    App, HttpResponse, HttpServer, Responder, get, post,
    web::{self, Data},
};

#[get("/")]
async fn hello(data: Data<AppState>) -> impl Responder {
    HttpResponse::Ok().body(format!("Hello world in {}!", data.app_name))
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

async fn index() -> impl Responder {
    "Hello world!"
}

#[get("/counter/get")]
async fn get_counter(data: Data<AppState>) -> String {
    match data.get_ref().counter.lock() {
        Ok(counter) => counter.to_string(),
        Err(_) => "Failed to get counter".to_owned(),
    }
}

#[get("/counter/incr")]
async fn incr_counter(data: Data<AppState>) -> String {
    match data.counter.lock() {
        Ok(mut counter) => {
            *counter += 1;
            "Counter incremented".to_owned()
        }
        Err(_) => "Failed to increment counter".to_owned(),
    }
}

#[post("counter/set")]
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

const APP_NAME: &str = "mdViewer";

struct AppState {
    app_name: &'static str,
    counter: Mutex<i32>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            app_name: APP_NAME,
            counter: Mutex::new(0),
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    unsafe {
        set_var("RUST_LOG", "debug");
    }
    env_logger::init();

    let data = Data::new(AppState::default());

    HttpServer::new(move || {
        App::new()
            .service(hello)
            .service(echo)
            .service(set_counter)
            .service(get_counter)
            .service(incr_counter)
            .route("/hey", web::get().to(manual_hello))
            .service(web::scope("/app").route("/index.html", web::get().to(index)))
            .app_data(data.clone())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
