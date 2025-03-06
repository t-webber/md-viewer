use actix_web::{HttpRequest, HttpResponse, web};

use crate::{
    AppData,
    drive::{FileType, get_file_metadata, insure_root_contains_file, load_files},
    state::ok_or_internal,
    token, unwrap_return_internal,
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
async fn display_file(data: AppData, req: HttpRequest, path: web::Path<(String,)>) -> HttpResponse {
    ok_or_internal(get_file_metadata(token!(data, req), &path.into_inner().0).await)
}

#[actix_web::get("/app-folder")]
async fn app_folder(req: HttpRequest, data: AppData) -> HttpResponse {
    HttpResponse::Ok().body(format!(
        "The id of the app folder is:\n{}",
        unwrap_return_internal!(data.as_drive().app_folder_id(token!(data, req)).await)
    ))
}

#[actix_web::get("/root/all")]
async fn ls_root(req: HttpRequest, data: AppData) -> HttpResponse {
    ok_or_internal(
        load_files(&[("q", "'root' in parents")], token!(data, req))
            .await
            .and_then(|files| {
                serde_json::to_string_pretty(&files)
                    .map_err(|err| format!("Failed to serialise:\n{err}"))
            }),
    )
}

#[actix_web::get("/root/type/{file_type}")]
async fn ls_root_type(req: HttpRequest, data: AppData, path: web::Path<(String,)>) -> HttpResponse {
    ok_or_internal(
        load_files(&[("q", "'root' in parents")], token!(data, req))
            .await
            .and_then(|files| {
                let filetype = path.into_inner().0;
                FileType::from_str(&filetype).map_or_else(
                    || Err(format!("Invalid file type {filetype}.")),
                    |parsed_filetype| {
                        serde_json::to_string_pretty(&files.filter_with_type(&parsed_filetype))
                            .map_err(|err| format!("Failed to serialise:\n{err}"))
                    },
                )
            }),
    )
}

#[actix_web::get("/root/make_hello")]
async fn make_hello(req: HttpRequest, data: AppData) -> HttpResponse {
    match insure_root_contains_file(token!(data, req), "hello", &FileType::Document).await {
        Err(err) => HttpResponse::InternalServerError().body(err),
        Ok(has) => HttpResponse::Ok().body(format!(
            "A document named hello at the root now exists!\nData:\n{has:?}"
        )),
    }
}

pub fn drive_config(cfg: &mut web::ServiceConfig) {
    cfg //
        .service(display_folder)
        .service(display_file)
        .service(app_folder)
        .service(ls_root)
        .service(ls_root_type)
        .service(make_hello);
}
