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
    clippy::pattern_type_mismatch,
    // clippy::missing_trait_methods,
    // clippy::else_if_without_else,
    reason = "bad lint"
)]
#![expect(clippy::blanket_clippy_restriction_lints, reason = "Enable all lints")]
#![expect(
    clippy::question_mark_used,
    clippy::mod_module_files,
    clippy::module_name_repetitions,
    clippy::arbitrary_source_item_ordering,
    clippy::unseparated_literal_suffix,
    // clippy::pub_with_shorthand,
    reason = "style"
)]
#![allow(clippy::missing_docs_in_private_items, reason = "lazy")]
#![expect(clippy::exhaustive_structs, reason = "needed by actix")]
#![expect(clippy::print_stderr, reason = "logging is good")]
#![allow(clippy::future_not_send, reason = "todo")]
#![allow(clippy::significant_drop_tightening, reason = "todo")]

mod api;
mod auth;
mod counter;
mod drive;
mod state;
mod url;

use actix_web::{App, HttpResponse, HttpServer, web};
use drive::routes::drive_config;
use state::{AppData, AppState};
use std::env::set_var;
use std::io;

const LOCAL_ENV_PATH: &str = ".env";

#[actix_web::get("/")]
async fn hello(data: AppData) -> HttpResponse {
    HttpResponse::Ok().body(format!("Hello world in {}!", data.as_app_name()))
}

#[actix_web::get("/debug")]
async fn debug(data: AppData) -> HttpResponse {
    HttpResponse::Ok().body(format!("{data:?}"))
}

fn config(cfg: &mut web::ServiceConfig) {
    cfg //
        .service(hello)
        .service(debug)
        .default_service(web::to(not_found))
        .service(web::scope("/auth").configure(auth::auth_config))
        .service(web::scope("/drive").configure(drive_config))
        .service(web::scope("/counter").configure(counter::counter_config));
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
