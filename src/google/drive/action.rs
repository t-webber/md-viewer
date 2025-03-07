use actix_web::{HttpRequest, HttpResponse, web};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg //
        .service(create_name)
        .service(get_content)
        .service(get_doc_len)
        .service(set_content);
}

use reqwest::Client;
use serde_json::{Value, json};

use crate::{
    log,
    state::{AppData, ok_or_internal},
    token, unwrap_return_internal,
};

#[actix_web::get("/create/{name}")]
async fn create_name(data: AppData, req: HttpRequest, path: web::Path<(String,)>) -> HttpResponse {
    let token = token!(data, req);
    ok_or_internal(
        create_file_with_name(
            &path.into_inner().0,
            &unwrap_return_internal!(data.as_drive().app_folder_id(token).await),
            token,
        )
        .await,
    )
}

#[actix_web::get("/get-doc-len/{id}")]
async fn get_doc_len(data: AppData, req: HttpRequest, path: web::Path<(String,)>) -> HttpResponse {
    ok_or_internal(
        get_document_length(&path.into_inner().0, token!(data, req))
            .await
            .map(|len| len.to_string()),
    )
}

#[actix_web::get("/get-content/{id}")]
async fn get_content(data: AppData, req: HttpRequest, path: web::Path<(String,)>) -> HttpResponse {
    ok_or_internal(get_file_content(&path.into_inner().0, token!(data, req)).await)
}

#[actix_web::post("/set-content/{id}")]
async fn set_content(
    data: AppData,
    req: HttpRequest,
    content: String,
    path: web::Path<(String,)>,
) -> HttpResponse {
    ok_or_internal(set_file_content(&path.into_inner().0, &content, token!(data, req)).await)
}

async fn create_file_with_name(name: &str, folder_id: &str, token: &str) -> Result<String, String> {
    serde_json::from_str::<Value>(
        &Client::new()
            .post("https://www.googleapis.com/drive/v3/files")
            .bearer_auth(token)
            .header("Content-Type", "application/json")
            .json(&json!({
                "name": name,
                "parents": [folder_id],
                "mimeType": "application/vnd.google-apps.document"
            }))
            .send()
            .await
            .map_err(|err| err.to_string())?
            .text()
            .await
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?
    .get("id")
    .and_then(|id| id.as_str())
    .map(str::to_owned)
    .ok_or_else(|| "Failed to get file ID".to_owned())
}

async fn get_file_content(id: &str, token: &str) -> Result<String, String> {
    Client::new()
        .get(format!(
            "https://www.googleapis.com/drive/v3/files/{id}/export?mimeType=text/plain"
        ))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|err| err.to_string())?
        .text()
        .await
        .map_err(|err| err.to_string())
}

async fn get_document_length(id: &str, token: &str) -> Result<i32, String> {
    let response = Client::new()
        .get(format!("https://docs.googleapis.com/v1/documents/{id}"))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|err| err.to_string())?;

    if response.status().is_success() {
        response
            .json::<Value>()
            .await
            .map_err(|err| format!("Invalid response:\n{err}"))?
            .get("body")
            .and_then(|value| value.get("content"))
            .and_then(|value| value.as_array())
            .and_then(|value| value.last())
            .and_then(|value| value.get("endIndex"))
            .and_then(Value::as_i64)
            .and_then(|value| i32::try_from(value).ok())
            .map_or_else(|| Err("Failed to find document end index.".to_owned()), Ok)
    } else {
        Err(format!(
            "Failed to retrieve document info: {}",
            response
                .text()
                .await
                .map_err(|err| format!("Response has invalid text:\n{err}"))?
        ))
    }
}

async fn set_file_content(id: &str, content: &str, token: &str) -> Result<String, String> {
    let end = get_document_length(id, token).await?.saturating_sub(1);
    log!(
        "Updating file {id} (current len = {}) with {content}.",
        end.saturating_sub(1)
    );

    let request_body = if end <= 1i32 {
        json!({
            "requests": [
                {
                    "insertText": {
                        "text": content,
                        "location":  {
                            // "segmentId" empty for body
                            // "tabId" empty for "singular tab"?
                            "index": 1i32,

                        }
                    }
                }
            ]
        })
    } else if content.is_empty() {
        json!({
            "requests": [
                {
                    "deleteContentRange": {
                        "range": {
                            // "segmentId"
                            // "tabId"
                            "startIndex": 1i32,
                            "endIndex": end,
                        }
                    },
                }
            ]
        })
    } else {
        json!({
            "requests": [
                {
                    "deleteContentRange": {
                        "range": {
                            // "segmentId"
                            // "tabId"
                            "startIndex": 1i32,
                            "endIndex": end,
                        }
                    },
                },
                {
                    "insertText": {
                        "text": content,
                        "location":  {
                            // "segmentId"
                            // "tabId"
                            "index": 1i32,

                        }
                    }
                }
            ]
        })
    };

    let response = Client::new()
        .post(format!(
            "https://docs.googleapis.com/v1/documents/{id}:batchUpdate"
        ))
        .bearer_auth(token)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|err| err.to_string())?;

    if response.status().is_success() {
        Ok(format!(
            "File updated with content {content}\nResponse:\n{}",
            response
                .text()
                .await
                .map_err(|err| format!("Failed to get text from response:\n{err}"))?
        ))
    } else {
        Err(format!(
            "Failed to update content:\n{}",
            response
                .text()
                .await
                .map_err(|err| format!("Failed to get text from response:\n{err}"))?
        ))
    }
}
