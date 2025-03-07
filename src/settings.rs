use core::fmt::Display;
use std::env::var;

use crate::auth::credentials::GoogleAuthCredentials;

const ERR_PREFIX: &str = "Failed to fetch Google OAuth2 credentials: ";

const ENV_PATH: &str = ".env";

pub struct Env {
    pub credentials: GoogleAuthCredentials,
    pub addr: (String, u16),
    pub app_folder: String,
}

pub fn unwrap_or_default<T, E, D, I>(res: Result<T, E>, default: I, var: D) -> T
where
    D: Display,
    I: Into<T> + Display,
{
    res.unwrap_or_else(|_e| {eprintln!("\n{var} not specified. Falling back to default: {default}.\nTo customise this, please add this variable in your `.env` file.\n"); default.into()})
}

pub fn load_env() -> Result<Env, String> {
    dotenv::from_filename(ENV_PATH)
        .map_err(|_err| format!("{ERR_PREFIX}Missing `{ENV_PATH}` file."))?;

    Ok(Env {
        credentials: GoogleAuthCredentials::new(
            get_var("ID")?,
            get_var("REDIRECT_URI")?,
            get_var("SECRET")?,
        ),
        addr: (
            unwrap_or_default(get_var("HOST"), "127.0.0.1", "HOST"),
            unwrap_or_default(
                get_var("PORT").map(|port| {
                    port.parse::<u16>().unwrap_or_else(|_err| {
                        eprintln!("\n`PORT` is not a valid integer. Falling back to default.\n");
                        8080u16
                    })
                }),
                8080u16,
                "PORT",
            ),
        ),
        app_folder: unwrap_or_default(get_var("APP_FOLDER"), "_@!md-viewer!@_", "APP_FOLDER"),
    })
}

fn get_var(env_var: &str) -> Result<String, String> {
    var(env_var).map_err(|_err| format!("{ERR_PREFIX}Missing variable `{env_var}` in `{ENV_PATH}`"))
}
