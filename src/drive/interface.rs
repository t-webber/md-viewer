use actix_web::{HttpRequest, HttpResponse, web};

pub fn interface_config(cfg: &mut web::ServiceConfig) {
    cfg //
        .service(create_name)
        .service(get_content)
        .service(get_doc_len)
        .service(set_content);
}

use reqwest::Client;
use serde_json::json;

use crate::{
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
    let url = "https://www.googleapis.com/drive/v3/files";

    let metadata = json!({
        "name": name,
        "parents": [folder_id],
        "mimeType": "application/vnd.google-apps.document"
    });

    let client = Client::new();
    let response = client
        .post(url)
        .bearer_auth(token)
        .header("Content-Type", "application/json")
        .json(&metadata)
        .send()
        .await
        .map_err(|err| err.to_string())?;

    let text = response.text().await.map_err(|err| err.to_string())?;
    let json: serde_json::Value = serde_json::from_str(&text).map_err(|err| err.to_string())?;

    json.get("id")
        .and_then(|id| id.as_str().map(String::from))
        .ok_or_else(|| "Failed to get file ID".to_owned())
}

async fn get_file_content(id: &str, token: &str) -> Result<String, String> {
    let url = format!("https://www.googleapis.com/drive/v3/files/{id}/export?mimeType=text/plain");

    let client = Client::new();
    let response = client
        .get(url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|err| err.to_string())?;

    response.text().await.map_err(|err| err.to_string())
}

async fn get_document_length(id: &str, token: &str) -> Result<i32, String> {
    let url = format!("https://docs.googleapis.com/v1/documents/{id}");

    let client = Client::new();
    let response = client
        .get(&url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|err| err.to_string())?;

    if response.status().is_success() {
        let json_response: serde_json::Value = response
            .json()
            .await
            .map_err(|err| format!("Invalid response:\n{err}"))?;
        if let Some(end_index) = json_response
            .get("body")
            .and_then(|value| value.get("content"))
            .and_then(|value| value.as_array())
            .and_then(|c| c.last())
            .and_then(|last_element| last_element["endIndex"].as_i64())
        {
            return Ok(end_index as i32);
        }
        Err("Failed to find document end index.".to_owned())
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
    let url = format!("https://docs.googleapis.com/v1/documents/{id}:batchUpdate");

    println!("Updating file {id} with {content}.");

    let end = dbg!(get_document_length(id, token).await?) - 1;

    let request_body = if end <= 1i32 {
        println!(">>>>>>>>>> Old empty");
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
        println!(">>>>>>>>>> New empty");
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
        println!(">>>>>>>>>> Nothing empty");
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

    let client = Client::new();
    let response = client
        .post(url)
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
