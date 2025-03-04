use std::{env::set_var, sync::Mutex};

use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use auth::{
    ClientOAuthData,
    credentials::{GoogleAuthCredentials, get_credentials},
};

mod auth;
mod counter;
mod url;

const LOCAL_ENV_PATH: &str = ".env";

#[actix_web::get("/")]
async fn hello(data: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().body(format!("Hello world in {}!", data.app_name))
}

#[actix_web::get("/debug")]
async fn debug(data: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().body(format!("{:?}", data))
}

const APP_NAME: &str = "mdViewer";

#[derive(Debug)]
struct AppState {
    app_name: &'static str,
    counter: Mutex<i32>,
    credentials: GoogleAuthCredentials,
    client_oauth_data: Mutex<Option<ClientOAuthData>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            app_name: APP_NAME,
            counter: Mutex::new(0),
            credentials: dbg!(get_credentials().unwrap()),
            client_oauth_data: Mutex::new(None),
        }
    }
}

fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(hello)
        .service(debug)
        .service(web::scope("/auth").configure(auth::auth_config))
        .service(web::scope("/counter").configure(counter::counter_config));
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    unsafe {
        set_var("RUST_LOG", "debug");
    }
    env_logger::init();

    let data = web::Data::new(AppState::new());

    HttpServer::new(move || App::new().configure(config).app_data(data.clone()))
        .bind(dbg!(url::get_url()))?
        .run()
        .await
}
