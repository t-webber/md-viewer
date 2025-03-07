pub mod auth;
pub mod drive;

use actix_web::web;

pub fn google_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/auth").configure(auth::auth_config))
        .service(web::scope("/drive").configure(drive::drive_config));
}
