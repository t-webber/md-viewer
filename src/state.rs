use core::sync::atomic::AtomicI32;
use std::sync::{Mutex, MutexGuard};

use actix_web::{HttpRequest, HttpResponse, web};

use crate::auth::{
    credentials::{GoogleAuthCredentials, get_credentials},
    login::ClientOAuthData,
};

pub type AppData = web::Data<AppState>;

#[derive(Debug)]
pub struct AppState {
    app_name: &'static str,
    callback: Mutex<Option<String>>,
    client_oauth_data: Mutex<Option<ClientOAuthData>>,
    counter: AtomicI32,
    credentials: GoogleAuthCredentials,
    root_folder: &'static str,
    root_folder_id: Mutex<Option<String>>,
}

pub fn ok_or_internal(value: Result<String, String>) -> HttpResponse {
    match value {
        Ok(val) => HttpResponse::Ok().body(val),
        Err(err) => HttpResponse::InternalServerError().body(err),
    }
}

pub fn map_err_internal<T>(value: Result<T, String>) -> Result<T, HttpResponse> {
    value.map_err(|err| HttpResponse::InternalServerError().body(err))
}

#[macro_export]
macro_rules! token {
    ($data:ident, $req:ident) => {
        &$crate::unwrap_return!($data.to_token(&$req))
    };
}

#[macro_export]
macro_rules! unwrap_return {
    ($value:expr) => {
        match $value {
            Ok(val) => val,
            Err(err) => return err,
        }
    };
}

#[macro_export]
macro_rules! unwrap_return_internal {
    ($value:expr) => {
        match $value {
            Ok(val) => val,
            Err(err) => return actix_web::HttpResponse::InternalServerError().body(err),
        }
    };
}

impl AppState {
    pub fn new() -> Result<web::Data<Self>, String> {
        Ok(web::Data::new(Self {
            app_name: "mdViewer",
            credentials: get_credentials()?,
            counter: AtomicI32::default(),
            client_oauth_data: Mutex::default(),
            callback: Mutex::default(),
            root_folder: "___@@@md-viewer@@@___",
            root_folder_id: Mutex::default(),
        }))
    }

    pub fn to_token(&self, req: &HttpRequest) -> Result<String, HttpResponse> {
        map_err_internal(unlock(&self.client_oauth_data, "client data"))?
            .as_ref()
            .map_or_else(
                || {
                    map_err_internal(self.set_callback(req.path().to_owned()))?;
                    Err(HttpResponse::TemporaryRedirect()
                        .append_header(("Location", "/auth/login"))
                        .finish())
                },
                |client_data| Ok(client_data.as_token().to_owned()),
            )
    }

    pub const fn as_app_name(&self) -> &str {
        self.app_name
    }

    pub const fn as_counter(&self) -> &AtomicI32 {
        &self.counter
    }

    pub const fn as_credentials(&self) -> &GoogleAuthCredentials {
        &self.credentials
    }

    fn set_callback(&self, new_callback: String) -> Result<(), String> {
        unlock(&self.callback, "callback")
            .map(|mut old_callback| *old_callback = Some(new_callback))
    }

    pub fn take_callback(&self) -> Result<String, String> {
        unlock(&self.callback, "callback")
            .map(|mut callback| callback.take().unwrap_or_else(|| "/auth/info".to_owned()))
    }

    pub fn set_client_data(&self, new_client_data: ClientOAuthData) -> Result<(), HttpResponse> {
        unlock(&self.client_oauth_data, "callback")
            .map_err(|err| HttpResponse::Ok().body(err))
            .map(|mut old_client_data| *old_client_data = Some(new_client_data))
    }
}

fn lock_error_msg(data_type: &str, err: &impl ToString) -> String {
    format!(
        "Failed to obtain lock for {data_type}:\n{}",
        err.to_string()
    )
}

fn unlock<'data, T>(
    data: &'data Mutex<T>,
    data_type: &'static str,
) -> Result<MutexGuard<'data, T>, String> {
    data.lock().map_err(|err| lock_error_msg(data_type, &err))
}
