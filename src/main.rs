use std::{env::set_var, sync::Mutex};

use actix_web::{
    App, HttpResponse, HttpServer, Responder,
    web::{self, Data},
};
use counter::counter_config;
use credentials::{GoogleAuthCredentials, get_credentials};

mod counter;
mod credentials;

#[actix_web::get("/")]
async fn hello(data: Data<AppState>) -> impl Responder {
    HttpResponse::Ok().body(format!("Hello world in {}!", data.app_name))
}

#[actix_web::get("/credentials")]
async fn display_credentials(data: Data<AppState>) -> impl Responder {
    HttpResponse::Ok().body(if let Some(credentials) = &data.credentials {
        format!("{credentials}")
    } else {
        "No credentials found".to_owned()
    })
}

const APP_NAME: &str = "mdViewer";

struct AppState {
    app_name: &'static str,
    counter: Mutex<i32>,
    credentials: Option<GoogleAuthCredentials>,
}

impl AppState {
    fn new() -> Self {
        Self {
            app_name: APP_NAME,
            counter: Mutex::new(0),
            credentials: get_credentials().map(Some).unwrap_or_else(|err| {
                eprintln!("{err}");
                None
            }),
        }
    }
}

fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(hello)
        .service(display_credentials)
        .service(web::scope("/counter").configure(counter_config));
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
