pub mod credentials;
pub mod login;

use actix_web::web;
use login::{google_callback, google_login, profile_info};

pub fn auth_config(cfg: &mut web::ServiceConfig) {
    cfg.service(google_callback)
        .service(google_login)
        .service(profile_info);
}
