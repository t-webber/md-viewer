use actix_web::{
    HttpResponse, http,
    web::{self, Redirect},
};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{AppState, api::send_and_text};

const SCOPE: &str = "email%20profile%20https://www.googleapis.com/auth/drive%20openid";

#[actix_web::get("/login")]
pub async fn google_login(data: web::Data<AppState>) -> Redirect {
    web::Redirect::to(format!(
        "https://accounts.google.com/o/oauth2/auth?client_id={client_id}&redirect_uri={redirect_uri}&response_type=code&scope={scope}&access_type=offline&prompt=consent",
        client_id = data.credentials.as_id(),
        redirect_uri = data.credentials.as_redirect_uri(),
        scope = SCOPE
    ))
    .permanent()
    .using_status_code(http::StatusCode::FOUND)
}

#[derive(Deserialize)]
struct CallBack {
    code: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientOAuthData {
    access_token: String,
    expires_in: u32,
    id_token: String,
    scope: String,
    token_type: String,
}

impl ClientOAuthData {
    pub fn as_token(&self) -> &str {
        &self.access_token
    }
}

#[actix_web::get("/callback/google")]
pub async fn google_callback(
    query: web::Query<CallBack>,
    data: web::Data<AppState>,
) -> HttpResponse {
    let content = match send_and_text(
        Client::new()
            .post("https://oauth2.googleapis.com/token")
            .form(&data.credentials.make_params(&query.code)),
    )
    .await
    {
        Ok(text) => match data.client_oauth_data.lock() {
            Ok(mut app_client_data) => match serde_json::from_str(&text) {
                Ok(new_client_data) => {
                    *app_client_data = Some(new_client_data);
                    return HttpResponse::Found()
                        .append_header(("Location", "/auth/info"))
                        .finish();
                }
                Err(err) => format!("Failed to parse response:\n{err}\nResponse:\n{text}"),
            },
            Err(err) => format!("Lock error:\n{err}"),
        },
        Err(err) => err,
    };
    if let Ok(mut cache) = data.cache.lock() {
        *cache = Some(content);
    }
    HttpResponse::Found()
        .append_header(("Location", "/auth/callback/error"))
        .finish()
}

#[actix_web::get("/callback/error")]
pub async fn callback_error(data: web::Data<AppState>) -> String {
    data.cache.lock().map_or_else(
        |_| String::from("Failed to access error"),
        |mut cache| {
            let res = cache
                .to_owned()
                .unwrap_or_else(|| String::from("Unknown error"));
            *cache = None;
            res
        },
    )
}

#[actix_web::get("/info")]
pub async fn profile_info(data: web::Data<AppState>) -> String {
    let token = match data.to_token() {
        Ok(token) => token,
        Err(err) => return err,
    };
    send_and_text(
        Client::new()
            .get("https://www.googleapis.com/oauth2/v2/userinfo")
            .bearer_auth(token),
    )
    .await
    .unwrap_or_else(|err| err)
}
