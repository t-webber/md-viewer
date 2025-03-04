#![warn(
    // missing_docs,
    warnings,
    deprecated_safe,
    future_incompatible,
    keyword_idents,
    let_underscore,
    nonstandard_style,
    refining_impl_trait,
    rust_2018_compatibility,
    rust_2018_idioms,
    rust_2021_compatibility,
    rust_2024_compatibility,
    unused,
    clippy::all,
    clippy::pedantic,
    clippy::style,
    clippy::perf,
    clippy::complexity,
    clippy::correctness,
    clippy::restriction,
    clippy::nursery,
    // clippy::cargo
)]
#![expect(
    clippy::single_call_fn,
    clippy::implicit_return,
    // clippy::missing_trait_methods,
    // clippy::else_if_without_else,
    // clippy::pattern_type_mismatch,
    reason = "bad lint"
)]
#![expect(clippy::blanket_clippy_restriction_lints, reason = "Enable all lints")]
#![expect(
    clippy::question_mark_used,
    clippy::mod_module_files,
    clippy::module_name_repetitions,
    // clippy::pub_with_shorthand,
    // clippy::unseparated_literal_suffix,
    reason = "style"
)]
#![expect(clippy::missing_docs_in_private_items, reason = "lazy")]
#![expect(clippy::exhaustive_structs, reason = "needed by actix")]

mod api;
mod auth;
mod counter;
mod drive;
mod url;

use actix_web::{App, HttpResponse, HttpServer, web};
use auth::{
    credentials::{GoogleAuthCredentials, get_credentials},
    login::ClientOAuthData,
};
use core::sync::atomic::AtomicI32;
use std::env::set_var;
use std::io;
use std::sync::Mutex;

const APP_NAME: &str = "mdViewer";
const LOCAL_ENV_PATH: &str = ".env";

#[derive(Debug)]
struct AppState {
    app_name: &'static str,
    cache: Mutex<Option<String>>,
    client_oauth_data: Mutex<Option<ClientOAuthData>>,
    counter: AtomicI32,
    credentials: GoogleAuthCredentials,
}

impl AppState {
    fn new() -> Result<web::Data<Self>, String> {
        Ok(web::Data::new(Self {
            app_name: APP_NAME,
            credentials: get_credentials()?,
            cache: Mutex::default(),
            counter: AtomicI32::default(),
            client_oauth_data: Mutex::default(),
        }))
    }

    fn to_token(&self) -> Result<String, String> {
        match self.client_oauth_data.lock().as_ref() {
            Err(err) => Err(format!("Failed to get global state:\n{err}")),
            Ok(data_guard) => data_guard.as_ref().map_or_else(
                || Err("User not logged in. Please go to /auth/login".to_owned()),
                |data| Ok(data.to_token()),
            ),
        }
    }
}

#[actix_web::get("/")]
async fn hello(data: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Ok().body(format!("Hello world in {}!", data.app_name))
}

#[actix_web::get("/debug")]
async fn debug(data: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Ok().body(format!("{data:?}"))
}

fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(hello)
        .service(debug)
        .service(web::scope("/auth").configure(auth::auth_config))
        .service(web::scope("/counter").configure(counter::counter_config))
        .service(web::scope("/drive").configure(drive::drive_config))
        .default_service(web::to(not_found));
}

async fn not_found() -> HttpResponse {
    HttpResponse::NotFound().body("Oops! Page not found.")
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    // SAFETY:
    // May fail, but will not cause problems (only useful for logging bugs)
    unsafe {
        set_var("RUST_LOG", "debug");
    };
    env_logger::init();

    let data = AppState::new().map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;

    HttpServer::new(move || App::new().configure(config).app_data(data.clone()))
        .bind(url::get_url())?
        .run()
        .await
}
