mod auth;
mod drive;

pub use auth::credentials::GoogleAuthCredentials;
pub use auth::login::ClientOAuthData;
pub use drive::manager::DriveManager;

use actix_web::web;

pub fn google_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/auth").configure(auth::auth_config))
        .service(web::scope("/drive").configure(drive::routes::drive_config));
}
