mod routes;

use actix_web::web;
use reqwest::Client;
use routes::{display_folder, ls_drive, ls_type, make_hello, route_has_blob, see_file};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::api::send_and_text;

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
    fn filter_with_type(self, filetype: &str) -> Box<[DriveFile]> {
        self.files
            .into_iter()
            .filter(|file| file.mimeType == format!("application/vnd.google-apps.{filetype}"))
            .collect()
    }

    fn find_with_name(self, filename: &str) -> Option<DriveFile> {
        self.files.into_iter().find(|file| file.name == filename)
    }
}

async fn app_folder_id(token: &str) -> Result<String, String> {
    insure_root_contains_file(token, APP_FOLDER, "folder")
        .await
        .map(|folder| folder.id)
}

async fn create_file(token: &str, filename: &str, filetype: &str) -> Result<DriveFile, String> {
    if filetype == "folder" {
        return create_folder(token, filename).await;
    }
    eprintln!("File {filename} not found. Creating...");

    let url = "https://www.googleapis.com/drive/v3/files";

    let metadata = json!({
        "name": filename,
        "mimeType": format!("application/vnd.google-apps.{filetype}")
    });

    match Client::new()
        .post(url)
        .bearer_auth(token)
        .header("Content-Type", "application/json")
        .json(&metadata)
        .send()
        .await
    {
        Ok(res) => match res.text().await {
            Ok(text) => serde_json::from_str(&text)
                .map_err(|err| format!("Failed to serialize response: {err}")),
            Err(err) => Err(format!("Failed to get text: {err}")),
        },
        Err(err) => Err(format!("Failed to post: {err}")),
    }
}

async fn create_folder(token: &str, filename: &str) -> Result<DriveFile, String> {
    eprintln!("File {filename} not found. Creating...");
    let url = "https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart";

    let metadata = json!({
        "name": filename,
        "mimeType": format!("application/vnd.google-apps.folder")
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
        .service(make_hello)
        .service(display_folder)
        .service(see_file)
        .service(route_has_blob);
}

async fn folder_contains_file(
    token: &str,
    filename: &str,
    folder_id: &str,
) -> Result<Option<DriveFile>, String> {
    load_files(&[("q", &format!("'{folder_id}' in parents"))], token)
        .await
        .map(|files| files.find_with_name(filename))
}

async fn insure_folder_contains_file(
    token: &str,
    filename: &str,
    filetype: &str,
    folder_path: &str,
    folder_id: &str,
) -> Result<DriveFile, String> {
    match folder_contains_file(token, filename, folder_id).await {
        Err(err) => Err(err),
        Ok(Some(created_folder)) => {
            eprintln!(">>>>>> exists !");
            Ok(created_folder)
        }
        Ok(None) => {
            eprintln!(">>>>>> created !");
            create_file(token, &format!("{folder_path}/{filename}"), filetype).await
        }
    }
}

async fn insure_root_contains_file(
    token: &str,
    filename: &str,
    filetype: &str,
) -> Result<DriveFile, String> {
    match root_contains_file(token, filename).await {
        Err(err) => Err(err),
        Ok(Some(created_folder)) => Ok(created_folder),
        Ok(None) => create_file(token, filename, filetype).await,
    }
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
        serde_json::from_str(&stringified)
            .map_err(|err| format!("Failed to deserialise on query {query:?}:\n{err}\n\nData (conversion objective was DriveFileList):\n{stringified}"))
    })
}

async fn root_contains_file(token: &str, filename: &str) -> Result<Option<DriveFile>, String> {
    load_files(&[("q", "'root' in parents")], token)
        .await
        .map(|files| files.files.into_iter().find(|file| file.name == filename))
}

async fn get_file_metadata(token: &str, file_id: &str) -> Result<String, String> {
    let url = format!("https://www.googleapis.com/drive/v3/files/{file_id}");

    match Client::new().get(&url).bearer_auth(token).send().await {
        Ok(res) => match res.text().await {
            Ok(text) => Ok(text), // Contains file name and MIME type
            Err(err) => Err(format!("Failed to get text: {err}")),
        },
        Err(err) => Err(format!("Failed to fetch metadata: {err}")),
    }
}
