use actix_web::{HttpRequest, HttpResponse, web};

use crate::{
    AppData,
    drive::{
        APP_FOLDER, app_folder_id, get_file_metadata, insure_folder_contains_file,
        insure_root_contains_file, load_files,
    },
    state::ok_or_internal,
    token,
};

#[actix_web::get("/folder/id/{id}")]
async fn display_folder(
    req: HttpRequest,
    data: AppData,
    path: web::Path<(String,)>,
) -> HttpResponse {
    ok_or_internal(
        load_files(
            &[("q", &format!("'{}' in parents", path.into_inner().0))],
            token!(data, req),
        )
        .await
        .and_then(|files| {
            serde_json::to_string_pretty(&files)
                .map_err(|err| format!("Failed to serialise:\n{err}"))
        }),
    )
}

#[actix_web::get("/file/id/{id}")]
async fn see_file(data: AppData, req: HttpRequest, path: web::Path<(String,)>) -> HttpResponse {
    ok_or_internal(get_file_metadata(token!(data, req), &path.into_inner().0).await)
}

#[actix_web::get("/has_blob")]
async fn route_has_blob(req: HttpRequest, data: AppData) -> HttpResponse {
    match insure_root_contains_file(token!(data, req), APP_FOLDER, "folder").await {
        Err(err) => HttpResponse::InternalServerError().body(err),
        Ok(has) => HttpResponse::Ok().body(format!("Your drive has {APP_FOLDER}\nData:\n{has:?}")),
    }
}

#[actix_web::get("/root")]
async fn ls_drive(req: HttpRequest, data: AppData) -> HttpResponse {
    ok_or_internal(
        load_files(&[("q", "'root' in parents")], token!(data, req))
            .await
            .and_then(|files| {
                serde_json::to_string_pretty(&files)
                    .map_err(|err| format!("Failed to serialise:\n{err}"))
            }),
    )
}

#[actix_web::get("/type/{file_type}")]
async fn ls_type(req: HttpRequest, data: AppData, path: web::Path<(String,)>) -> HttpResponse {
    ok_or_internal(
        load_files(&[("q", "'root' in parents")], token!(data, req))
            .await
            .and_then(|files| {
                serde_json::to_string_pretty(&files.filter_with_type(&path.into_inner().0))
                    .map_err(|err| format!("Failed to serialise:\n{err}"))
            }),
    )
}

#[actix_web::get("/hello")]
async fn make_hello(req: HttpRequest, data: AppData) -> HttpResponse {
    match app_folder_id(token!(data, req)).await {
        Ok(folder_id) => {
            match insure_folder_contains_file(
                token!(data, req),
                "file_eg",
                "document",
                APP_FOLDER,
                &folder_id,
            )
            .await
            {
                Err(err) => HttpResponse::InternalServerError().body(err),
                Ok(has) => {
                    HttpResponse::Ok().body(format!("{APP_FOLDER}/file_eg exists!\nData:\n{has:?}"))
                }
            }
        }
        Err(err) => HttpResponse::InternalServerError().body(err),
    }
}
