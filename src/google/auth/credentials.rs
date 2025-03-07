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

    pub const fn new(id: String, redirect_uri: String, secret: String) -> Self {
        Self { id, redirect_uri, secret }
    }
}
