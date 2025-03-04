use actix_web::web;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{AppState, api::send_and_text, token};

const APP_FOLDER: &str = "___@@@md-viewer@@@___";

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
    cfg.service(ls_drive)
        .service(ls_type)
        .service(ls_folder)
        .service(make_blob)
        .service(route_has_blob);
}

async fn has_folder(token: &str, name: &str) -> Result<bool, String> {
    load_files(&[("q", "'root' in parents")], token)
        .await
        .map(|files| files.files.iter().any(|file| file.name == name))
}

#[actix_web::get("/root")]
async fn ls_drive(data: web::Data<AppState>) -> String {
    load_files(&[("q", "'root' in parents")], token!(data))
        .await
        .and_then(|files| {
            serde_json::to_string_pretty(&files)
                .map_err(|err| format!("Failed to serialise:\n{err}"))
        })
        .unwrap_or_else(|err| err)
}

#[actix_web::get("/type/{file_type}")]
async fn ls_type(data: web::Data<AppState>, path: web::Path<(String,)>) -> String {
    load_files(&[("q", "'root' in parents")], token!(data))
        .await
        .and_then(|files| {
            serde_json::to_string_pretty(&files.filter_with_type(&path.into_inner().0))
                .map_err(|err| format!("Failed to serialise:\n{err}"))
        })
        .unwrap_or_else(|err| err)
}

async fn load_files(query: &[(&str, &str)], token: &str) -> Result<DriveFileList, String> {
    send_and_text(
        Client::new()
            .get("https://www.googleapis.com/drive/v3/files")
            .bearer_auth(token)
            .query(query),
    )
    .await
    .and_then(|stringified| {
        serde_json::from_str(&stringified).map_err(|err| format!("Failed to deserialise:\n{err}"))
    })
}

#[actix_web::get("/folder/{id}")]
async fn ls_folder(data: web::Data<AppState>, path: web::Path<(String,)>) -> String {
    load_files(
        &[("q", &format!("'{}' in parents", path.into_inner().0))],
        token!(data),
    )
    .await
    .and_then(|files| {
        serde_json::to_string_pretty(&files).map_err(|err| format!("Failed to serialise:\n{err}"))
    })
    .unwrap_or_else(|err| err)
}

#[actix_web::get("/has_blob")]
async fn route_has_blob(data: web::Data<AppState>) -> String {
    has_folder(token!(data), APP_FOLDER).await.map_or_else(
        |err| err,
        |has| format!("Your drive has {APP_FOLDER}?: {has}"),
    )
}

#[actix_web::get("/make_blob")]
async fn make_blob(data: web::Data<AppState>) -> String {
    let url = "https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart";

    let metadata = json!({
        "name": APP_FOLDER,
        "mimeType": "application/vnd.google-apps.folder"
    })
    .to_string();

    let boundary = "boundary";
    let multipart = format!(
        "--{boundary}\r\n\
         Content-Type: application/json; charset=UTF-8\r\n\r\n\
         {metadata}\r\n\
         --{boundary}--\r\n",
    );

    let content_type = format!("multipart/related; boundary={boundary}");

    match Client::new()
        .post(url)
        .bearer_auth(token!(data))
        .header("Content-Type", content_type)
        .body(multipart)
        .send()
        .await
    {
        Ok(res) => match res.text().await {
            Ok(text) => text,
            Err(err) => {
                format!("Failed to get text: {err}")
            }
        },
        Err(err) => format!("Failed to post: {err}"),
    }
}
