use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct HttpRequestChecker {
    pub url: String,
}