use actix_web::web;

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

#[derive(serde::Deserialize)]
struct CallBack {
    code: String,
}

#[actix_web::get("/callback/google")]
async fn google_callback(query: web::Query<CallBack>, data: web::Data<AppState>) -> String {
    match data.google_token.lock() {
        Ok(mut token) => {
            *token = Some(query.code.clone());
            format!("Token set to {}", query.code)
        }
        Err(_) => format!("Failed to set token to {}", query.code),
    }
}

pub fn auth_config(cfg: &mut web::ServiceConfig) {
    cfg.service(login).service(google_callback);
}
