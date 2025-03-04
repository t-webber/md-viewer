use actix_web::{HttpResponse, web};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{AppState, api::send_and_text, ok_or_internal, token};

const APP_FOLDER: &str = "___@@@md-viewer@@@___";

#[derive(Deserialize, Serialize, Debug)]
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

async fn create_folder(token: &str, folder_name: &str) -> Result<DriveFile, String> {
    eprintln!("Folder {folder_name} not found. Creating...");
    let url = "https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart";

    let metadata = json!({
        "name": folder_name,
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
        .bearer_auth(token)
        .header("Content-Type", content_type)
        .body(multipart)
        .send()
        .await
    {
        Ok(res) => match res.text().await {
            Ok(text) => serde_json::from_str(&text)
                .map_err(|err| format!("Failed to serialise response: {err}")),
            Err(err) => Err(format!("Failed to get text: {err}")),
        },
        Err(err) => Err(format!("Failed to post: {err}")),
    }
}

pub fn drive_config(cfg: &mut web::ServiceConfig) {
    cfg.service(ls_drive)
        .service(ls_type)
        .service(ls_folder)
        .service(route_has_blob)
        .service(route_has_blob);
}

async fn has_folder(token: &str, name: &str) -> Result<Option<DriveFile>, String> {
    load_files(&[("q", "'root' in parents")], token)
        .await
        .map(|files| files.files.into_iter().find(|file| file.name == name))
}

async fn insure_folder_exists(token: &str, folder_name: &str) -> Result<DriveFile, String> {
    match has_folder(token, folder_name).await {
        Err(err) => Err(err),
        Ok(Some(created_folder)) => Ok(created_folder),
        Ok(None) => create_folder(token, folder_name).await,
    }
}

#[actix_web::get("/root")]
async fn ls_drive(data: web::Data<AppState>) -> HttpResponse {
    ok_or_internal!(
        load_files(&[("q", "'root' in parents")], token!(data))
            .await
            .and_then(|files| {
                serde_json::to_string_pretty(&files)
                    .map_err(|err| format!("Failed to serialise:\n{err}"))
            })
    )
}

#[actix_web::get("/type/{file_type}")]
async fn ls_type(data: web::Data<AppState>, path: web::Path<(String,)>) -> HttpResponse {
    ok_or_internal!(
        load_files(&[("q", "'root' in parents")], token!(data))
            .await
            .and_then(|files| {
                serde_json::to_string_pretty(&files.filter_with_type(&path.into_inner().0))
                    .map_err(|err| format!("Failed to serialise:\n{err}"))
            })
    )
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
async fn ls_folder(data: web::Data<AppState>, path: web::Path<(String,)>) -> HttpResponse {
    ok_or_internal!(
        load_files(
            &[("q", &format!("'{}' in parents", path.into_inner().0))],
            token!(data),
        )
        .await
        .and_then(|files| {
            serde_json::to_string_pretty(&files)
                .map_err(|err| format!("Failed to serialise:\n{err}"))
        })
    )
}

#[actix_web::get("/has_blob")]
async fn route_has_blob(data: web::Data<AppState>) -> HttpResponse {
    match insure_folder_exists(token!(data), APP_FOLDER).await {
        Err(err) => HttpResponse::InternalServerError().body(err),
        Ok(has) => HttpResponse::Ok().body(format!("Your drive has {APP_FOLDER}\nData:\n{has:?}")),
    }
}
