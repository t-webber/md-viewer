use std::env;

use crate::LOCAL_ENV_PATH;

const HOST: &str = "HOST";
const PORT: &str = "PORT";

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 8080;

pub fn get_url() -> (String, u16) {
    if dotenv::from_filename(LOCAL_ENV_PATH).is_err() {
        eprintln!(
            "Failed to fetch `{HOST}` and `{PORT}` from `{LOCAL_ENV_PATH}`. Falling back to default."
        )
    }

    (
        env::var(HOST).unwrap_or(DEFAULT_HOST.to_owned()),
        env::var(PORT)
            .map_err(|_| ())
            .and_then(|port| port.parse::<u16>().map_err(|_| ()))
            .unwrap_or_else(|_| {
                eprintln!("Found a `{PORT}` that is not a valid integer. Falling back to default.");
                DEFAULT_PORT
            }),
    )
}
