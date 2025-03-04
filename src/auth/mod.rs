use actix_web::web;
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientOAuthData {
    access_token: String,
    expires_in: u32,
    scope: String,
    token_type: String,
    id_token: String,
}

#[actix_web::get("/callback/google")]
async fn google_callback(query: web::Query<CallBack>, data: web::Data<AppState>) -> String {
    match send_and_text(
        Client::new()
            .post("https://oauth2.googleapis.com/token")
            .form(&data.credentials.make_params(&query.code)),
    )
    .await
    {
        Ok(text) => match data.client_oauth_data.lock() {
            Ok(mut client_oauth_data) => match serde_json::from_str(&text) {
                Ok(data) => {
                    *client_oauth_data = Some(data);
                    format!("Good response: {client_oauth_data:?}")
                }
                Err(err) => format!("Failed to parse response:\n{err}\nResponse:\n{text}"),
            },
            Err(err) => format!("Lock error:\n{err}"),
        },
        Err(err) => err,
    }
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

#[actix_web::get("/info")]
async fn profile_info(data: web::Data<AppState>) -> String {
    let token = match data.client_oauth_data.lock().as_ref() {
        Err(err) => return format!("Failed to get global state:\n{err}"),
        Ok(data) => match data.as_ref() {
            Some(data) => data.access_token.clone(),
            None => return "User not logged in. Please go to /auth/login".to_owned(),
        },
    };
    send_and_text(
        Client::new()
            .get("https://www.googleapis.com/oauth2/v2/userinfo")
            .bearer_auth(token),
    )
    .await
    .unwrap_or_else(|err| err)
}

pub fn auth_config(cfg: &mut web::ServiceConfig) {
    cfg.service(login)
        .service(google_callback)
        .service(profile_info);
}
