pub mod interface;
pub mod manager;
pub mod routes;

use core::result;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{api::send_and_text, log};

type Result<T, E = String> = result::Result<T, E>;

#[derive(Deserialize, Serialize, Debug)]
#[expect(non_snake_case, reason = "needed by serde")]
struct DriveFile {
    id: String,
    kind: String,
    mimeType: String,
    name: String,
}

macro_rules! make_file_type {
    ($($pascal:ident $str:expr,)*) => {
        enum FileType {
            $($pascal,)*
        }

        impl FileType {
            fn from_str(value: &str) -> Option<Self> {
                match value {
                    $($str => Some(Self::$pascal),)*
                    _ => None
                }
            }

            const fn as_str(&self) -> &str {
                match self {
                    $(Self::$pascal => $str,)*
                }
            }
        }
    };
}

impl FileType {
    fn as_mime_type(&self) -> String {
        format!("application/vnd.google-apps.{}", self.as_str())
    }
}

make_file_type!(
    Document "document",
    Spreadsheet "spreadsheet",
    Folder "folder",
);

#[derive(Deserialize, Serialize)]
#[expect(non_snake_case, reason = "needed by serde")]
struct DriveFileList {
    files: Box<[DriveFile]>,
    incompleteSearch: bool,
    kind: String,
}

impl DriveFileList {
    fn filter_with_type(self, filetype: &FileType) -> Box<[DriveFile]> {
        self.files
            .into_iter()
            .filter(|file| file.mimeType == filetype.as_mime_type())
            .collect()
    }

    fn find(self, filename: &str, filetype: &FileType) -> Option<DriveFile> {
        self.files
            .into_iter()
            .find(|file| file.name == filename && file.mimeType == filetype.as_mime_type())
    }
}

async fn create_file(token: &str, filename: &str, filetype: &FileType) -> Result<DriveFile> {
    log!("File {filename} not found. Creating...");

    match Client::new()
        .post("https://www.googleapis.com/drive/v3/files")
        .bearer_auth(token)
        .header("Content-Type", "application/json")
        .json(&json!({
            "name": filename,
            "mimeType": filetype.as_mime_type()
        }))
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

async fn create_folder(token: &str, filename: &str) -> Result<DriveFile> {
    log!("File {filename} not found. Creating...");

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
        .post("https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart")
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

async fn insure_root_contains_file(
    token: &str,
    filename: &str,
    filetype: &FileType,
) -> Result<DriveFile> {
    match root_contains_file(token, filename, filetype).await {
        Err(err) => Err(err),
        Ok(Some(created_folder)) => Ok(created_folder),
        Ok(None) => create_file(token, filename, filetype).await,
    }
}

async fn load_files(query: &[(&str, &str)], token: &str) -> Result<DriveFileList> {
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

async fn root_contains_file(
    token: &str,
    filename: &str,
    filetype: &FileType,
) -> Result<Option<DriveFile>> {
    load_files(&[("q", "'root' in parents")], token)
        .await
        .map(|files| files.find(filename, filetype))
}

async fn get_file_metadata(token: &str, file_id: &str) -> Result<String> {
    let url = format!("https://www.googleapis.com/drive/v3/files/{file_id}");

    match Client::new().get(&url).bearer_auth(token).send().await {
        Ok(res) => match res.text().await {
            Ok(text) => Ok(text), // Contains file name and MIME type
            Err(err) => Err(format!("Failed to get text: {err}")),
        },
        Err(err) => Err(format!("Failed to fetch metadata: {err}")),
    }
}
