use actix_web::web::{self, Redirect};
use actix_web::{HttpRequest, HttpResponse, http};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::api::send_and_text;
use crate::state::ok_or_internal;
use crate::{AppData, token, unwrap_return, unwrap_return_internal};

const SCOPE: &str = "email%20profile%20https://www.googleapis.com/auth/drive%20openid";

#[actix_web::get("/login")]
async fn google_login(data: AppData) -> Redirect {
    web::Redirect::to(format!(
        "https://accounts.google.com/o/oauth2/auth?client_id={client_id}&redirect_uri={redirect_uri}&response_type=code&scope={scope}&access_type=offline",
        client_id = data.as_credentials().as_id(),
        redirect_uri = data.as_credentials().as_redirect_uri(),
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
async fn google_callback(query: web::Query<CallBack>, data: AppData) -> HttpResponse {
    HttpResponse::InternalServerError().body(
        match send_and_text(
            Client::new()
                .post("https://oauth2.googleapis.com/token")
                .form(&data.as_credentials().as_params(&query.code)),
        )
        .await
        {
            Ok(text) => match serde_json::from_str(&text) {
                Ok(new_client_data) => {
                    unwrap_return!(data.set_client_data(new_client_data));
                    return HttpResponse::Found()
                        .append_header(("Location", unwrap_return_internal!(data.take_callback())))
                        .finish();
                }
                Err(err) => format!("Failed to parse response:\n{err}\nResponse:\n{text}"),
            },
            Err(err) => err,
        },
    )
}

#[actix_web::get("/info")]
async fn profile_info(data: AppData, req: HttpRequest) -> HttpResponse {
    ok_or_internal(
        send_and_text(
            Client::new()
                .get("https://www.googleapis.com/oauth2/v2/userinfo")
                .bearer_auth(token!(data, req)),
        )
        .await,
    )
}
