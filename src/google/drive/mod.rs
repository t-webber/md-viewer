mod action;
mod interface;
pub mod manager;

use actix_web::{HttpRequest, HttpResponse, web};
use interface::folder_contents;

use crate::state::{AppData, ok_or_internal};
use crate::{token, unwrap_return_internal};

#[actix_web::get("/ls")]
async fn ls(req: HttpRequest, data: AppData) -> HttpResponse {
    let token = token!(data, req);
    ok_or_internal(
        folder_contents(
            token,
            &unwrap_return_internal!(data.as_drive().app_folder_id(token).await),
        )
        .await
        .map(|drivelist| serde_json::to_string_pretty(&drivelist).unwrap()),
    )
}

pub fn drive_config(cfg: &mut web::ServiceConfig) {
    cfg //
        .service(ls)
        .service(web::scope("/action").configure(action::config));
}
