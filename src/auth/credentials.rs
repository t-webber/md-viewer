use std::env;

use crate::LOCAL_ENV_PATH;

const ERR_PREFIX: &str = "Failed to fetch Google OAuth2 credentials: ";
const ID: &str = "ID";
const REDIRECT_URI: &str = "REDIRECT_URI";
const SECRET: &str = "SECRET";

#[derive(Debug)]
pub struct GoogleAuthCredentials {
    id: String,
    redirect_uri: String,
    secret: String,
}

impl GoogleAuthCredentials {
    pub const fn as_id(&self) -> &String {
        &self.id
    }
    pub const fn as_redirect_uri(&self) -> &String {
        &self.redirect_uri
    }
    pub fn as_params<'code, 'db: 'code>(
        &'db self,
        code: &'code str,
    ) -> [(&'code str, &'code str); 5] {
        [
            ("client_id", self.id.as_str()),
            ("client_secret", self.secret.as_str()),
            ("code", code),
            ("redirect_uri", self.redirect_uri.as_str()),
            ("grant_type", "authorization_code"),
        ]
    }
}

pub fn get_credentials() -> Result<GoogleAuthCredentials, String> {
    dotenv::from_filename(LOCAL_ENV_PATH)
        .map_err(|_err| format!("{ERR_PREFIX}Missing `{LOCAL_ENV_PATH}` file."))?;

    Ok(GoogleAuthCredentials {
        id: get_var(ID)?,
        secret: get_var(SECRET)?,
        redirect_uri: get_var(REDIRECT_URI)?,
    })
}

fn get_var(env_var: &str) -> Result<String, String> {
    env::var(env_var)
        .map_err(|_err| format!("{ERR_PREFIX}Missing variable `{env_var}` in `{LOCAL_ENV_PATH}`"))
}
