use std::env;

use crate::LOCAL_ENV_PATH;

#[derive(Debug)]
pub struct GoogleAuthCredentials {
    id: String,
    secret: String,
    redirect_uri: String,
}

impl GoogleAuthCredentials {
    pub fn as_id(&self) -> &String {
        &self.id
    }
    pub fn as_redirect_uri(&self) -> &String {
        &self.redirect_uri
    }
}

const ID: &str = "ID";
const SECRET: &str = "SECRET";
const REDIRECT_URI: &str = "REDIRECT_URI";

const ERR_PREFIX: &str = "Failed to fetch Google OAuth2 credentials: ";

fn get_var(env_var: &str) -> Result<String, String> {
    env::var(env_var)
        .map_err(|_| format!("{ERR_PREFIX}Missing variable `{env_var}` in `{LOCAL_ENV_PATH}`"))
}

pub fn get_credentials() -> Result<GoogleAuthCredentials, String> {
    dotenv::from_filename(LOCAL_ENV_PATH)
        .map_err(|_| format!("{ERR_PREFIX}Missing `{LOCAL_ENV_PATH}` file."))?;

    Ok(GoogleAuthCredentials {
        id: get_var(ID)?,
        secret: get_var(SECRET)?,
        redirect_uri: get_var(REDIRECT_URI)?,
    })
}
