use actix_web::{HttpRequest, web};

use crate::AppState;

pub mod credentials;

#[actix_web::get("/login")]
async fn login(data: web::Data<AppState>) -> String {
    format!(
        "https://accounts.google.com/o/oauth2/auth?client_id={client_id}&redirect_uri={redirect_uri}&response_type=code&scope=email%20profile&access_type=offline",
        client_id = data.credentials.as_id(),
        redirect_uri = data.credentials.as_redirect_uri()
    )
}

#[actix_web::get("/callback/{auth_code}")]
async fn callback(req: HttpRequest) -> String {
    req.match_info().get("auth_code").map_or_else(
        || String::from("Invalid Auth Code."),
        |code| code.to_owned(),
    )
}

pub fn auth_config(cfg: &mut web::ServiceConfig) {
    cfg.service(login).service(callback);
}
