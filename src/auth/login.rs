use actix_web::web;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{AppState, auth::send_and_text};

const SCOPE: &str = "email%20profile%20https://www.googleapis.com/auth/drive.readonly";

#[actix_web::get("/login")]
pub async fn google_login(data: web::Data<AppState>) -> String {
    format!(
        "https://accounts.google.com/o/oauth2/auth?client_id={client_id}&redirect_uri={redirect_uri}&response_type=code&scope={scope}&access_type=offline",
        client_id = data.credentials.as_id(),
        redirect_uri = data.credentials.as_redirect_uri(),
        scope = SCOPE
    )
}

#[derive(Deserialize)]
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

impl ClientOAuthData {
    pub fn to_token(&self) -> String {
        self.access_token.to_owned()
    }
}

#[actix_web::get("/callback/google")]
pub async fn google_callback(query: web::Query<CallBack>, data: web::Data<AppState>) -> String {
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
