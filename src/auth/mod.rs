pub mod credentials;
pub mod login;

use actix_web::web;
use login::{callback_error, google_callback, google_login, profile_info};

pub fn auth_config(cfg: &mut web::ServiceConfig) {
    cfg.service(google_callback)
        .service(callback_error)
        .service(google_login)
        .service(profile_info);
}
