use anyhow::Result;
use serde::Deserialize;

pub struct IpfsClient {
    gateway: String,
    http: reqwest::Client,
}

#[derive(Deserialize)]
struct AddResponse {
    #[serde(rename = "Hash")]
    hash: String,
}

impl IpfsClient {
    pub fn new(gateway: &str) -> Result<Self> {
        Ok(IpfsClient {
            gateway: gateway.to_string(),
            http: reqwest::Client::new(),
        })
    }

    pub async fn upload(&self, content: &str) -> Result<String> {
        let url = format!("{}/api/v0/add", self.gateway);
        let form = reqwest::multipart::Form::new()
            .text("file", content.to_string());

        let resp = self.http.post(&url)
            .multipart(form)
            .send().await?;

        let add_resp: AddResponse = resp.json().await?;
        Ok(add_resp.hash)
    }

    pub async fn download(&self, cid: &str) -> Result<String> {
        let url = format!("{}/api/v0/cat?arg={}", self.gateway, cid);
        let resp = self.http.post(&url).send().await?;
        Ok(resp.text().await?)
    }
}
