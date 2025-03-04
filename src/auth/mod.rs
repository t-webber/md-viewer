pub mod credentials;
pub mod drive;
pub mod login;

use actix_web::web;
use drive::ls_drive;
use login::{callback_error, google_callback, google_login, profile_info};
use reqwest::RequestBuilder;

pub fn auth_config(cfg: &mut web::ServiceConfig) {
    cfg.service(google_callback)
        .service(callback_error)
        .service(google_login)
        .service(ls_drive)
        .service(profile_info);
}

async fn send_and_text(req: RequestBuilder) -> Result<String, String> {
    match req.send().await {
        Ok(value) => match value.text().await {
            Ok(text) => Ok(text),
            Err(err) => Err(format!("Text error:\n{err}")),
        },
        Err(err) => Err(format!("Request error:\n{err}")),
    }
}
