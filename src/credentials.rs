use std::{fmt::Display, fs::read_to_string};

pub struct GoogleAuthCredentials {
    id: String,
    secret: String,
    redirect_uri: String,
}

impl Display for GoogleAuthCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            id,
            secret,
            redirect_uri,
        } = self;
        write!(
            f,
            "Id: {id}\nSecret: {secret}\nRedirect URI: {redirect_uri}"
        )
    }
}

const ENV_PATH: &str = ".env";

const ID: &str = "ID";
const SECRET: &str = "SECRET";
const REDIRECT_URI: &str = "REDIRECT_URI";

const ERR_PREFIX: &str = "Failed to fetch Google OAuth2 credentials: ";

fn err(msg: &str, var: &str) -> String {
    format!("{ERR_PREFIX}{msg} variable `{var}` in `{ENV_PATH}`.")
}

fn unwrap(value: Option<Option<&str>>, var: &str) -> Result<String, String> {
    Ok(value
        .ok_or_else(|| err("Missing", var))?
        .ok_or_else(|| err("Missing value for", var))?
        .to_owned())
}

pub fn get_credentials() -> Result<GoogleAuthCredentials, String> {
    match read_to_string(ENV_PATH) {
        Ok(env) => {
            let mut id = None;
            let mut secret = None;
            let mut redirect_uri = None;
            for line in env.lines() {
                if line.starts_with(ID) {
                    id = Some(line.get(ID.len() + 1..));
                } else if line.starts_with(SECRET) {
                    secret = Some(line.get(SECRET.len() + 1..));
                } else if line.starts_with(REDIRECT_URI) {
                    redirect_uri = Some(line.get(REDIRECT_URI.len() + 1..));
                }
            }
            Ok(GoogleAuthCredentials {
                id: unwrap(id, ID)?,
                secret: unwrap(secret, SECRET)?,
                redirect_uri: unwrap(redirect_uri, REDIRECT_URI)?,
            })
        }
        Err(_) => Err(format!("{ERR_PREFIX}Missing `{ENV_PATH}`.")),
    }
}
