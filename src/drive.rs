use actix_web::web;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{AppState, api::send_and_text};

#[derive(Deserialize, Serialize)]
#[expect(non_snake_case, reason = "needed by serde")]
struct DriveFile {
    id: String,
    kind: String,
    mimeType: String,
    name: String,
}

#[derive(Deserialize, Serialize)]
#[expect(non_snake_case, reason = "needed by serde")]
struct DriveFileList {
    files: Box<[DriveFile]>,
    incompleteSearch: bool,
    kind: String,
}

impl DriveFileList {
    fn filter_with_type(self, file_type: &str) -> Box<[DriveFile]> {
        self.files
            .into_iter()
            .filter(|x| x.mimeType == format!("application/vnd.google-apps.{file_type}"))
            .collect()
    }
}

pub fn drive_config(cfg: &mut web::ServiceConfig) {
    cfg.service(ls_drive).service(ls_type);
}

#[actix_web::get("")]
async fn ls_drive(data: web::Data<AppState>) -> String {
    let token = match data.to_token() {
        Ok(token) => token,
        Err(err) => return err,
    };
    load_files("'root' in parents", token)
        .await
        .and_then(|files| {
            serde_json::to_string_pretty(&files)
                .map_err(|err| format!("Failed to serialise:\n{err}"))
        })
        .unwrap_or_else(|err| err)
}

#[actix_web::get("/{file_type}")]
async fn ls_type(data: web::Data<AppState>, path: web::Path<(String,)>) -> String {
    let token = match data.to_token() {
        Ok(token) => token,
        Err(err) => return err,
    };
    load_files("'root' in parents", token)
        .await
        .and_then(|files| {
            serde_json::to_string_pretty(&files.filter_with_type(&path.into_inner().0))
                .map_err(|err| format!("Failed to serialise:\n{err}"))
        })
        .unwrap_or_else(|err| err)
}

async fn load_files(query: &str, token: String) -> Result<DriveFileList, String> {
    send_and_text(
        Client::new()
            .get("https://www.googleapis.com/drive/v3/files")
            .bearer_auth(token)
            .query(&[("q", query)]),
    )
    .await
    .and_then(|stringified| {
        serde_json::from_str(&stringified).map_err(|err| format!("Failed to deserialise:\n{err}"))
    })
}
