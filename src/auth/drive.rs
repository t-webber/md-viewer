use actix_web::web;
use reqwest::Client;

use crate::{AppState, auth::send_and_text};

#[actix_web::get("/ls")]
pub async fn ls_drive(data: web::Data<AppState>) -> String {
    let token = match data.to_token() {
        Ok(token) => token,
        Err(err) => return err,
    };
    send_and_text(
        Client::new()
            .get("https://www.googleapis.com/drive/v3/files")
            .bearer_auth(token)
            .query(&[("q", "'root' in parents")]),
    )
    .await
    .unwrap_or_else(|err| err)
}
