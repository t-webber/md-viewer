use std::{env::set_var, sync::Mutex};

use actix_web::{
    App, HttpResponse, HttpServer, Responder,
    web::{self, Data},
};
use counter::counter_config;

mod counter;

#[actix_web::get("/")]
async fn hello(data: Data<AppState>) -> impl Responder {
    HttpResponse::Ok().body(format!("Hello world in {}!", data.app_name))
}

#[actix_web::post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

async fn index() -> impl Responder {
    "Hello world!"
}

const APP_NAME: &str = "mdViewer";

struct AppState {
    app_name: &'static str,
    counter: Mutex<i32>,
}

impl AppState {
    fn new() -> Self {
        Self {
            app_name: APP_NAME,
            counter: Mutex::new(0),
        }
    }
}

fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(hello)
        .service(echo)
        .service(web::scope("/counter").configure(counter_config))
        .route("/hey", web::get().to(manual_hello))
        .service(web::scope("/app").route("/index.html", web::get().to(index)));
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    unsafe {
        set_var("RUST_LOG", "debug");
    }
    env_logger::init();

    let data = Data::new(AppState::new());

    HttpServer::new(move || App::new().configure(config).app_data(data.clone()))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
