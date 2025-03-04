use reqwest::RequestBuilder;

pub async fn send_and_text(req: RequestBuilder) -> Result<String, String> {
    match req.send().await {
        Ok(value) => match value.text().await {
            Ok(text) => Ok(text),
            Err(err) => Err(format!("Text error:\n{err}")),
        },
        Err(err) => Err(format!("Request error:\n{err}")),
    }
}
