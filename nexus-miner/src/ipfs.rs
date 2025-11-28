//! IPFS client for uploading/downloading molecular data

use anyhow::Result;

pub struct IpfsClient {
    gateway: String,
    client: reqwest::Client,
}

impl IpfsClient {
    pub fn new(gateway: &str) -> Result<Self> {
        Ok(Self {
            gateway: gateway.to_string(),
            client: reqwest::Client::new(),
        })
    }
    
    pub async fn upload(&self, content: &str) -> Result<String> {
        // TODO: Implement actual IPFS upload
        Ok("QmPlaceholder".to_string())
    }
    
    pub async fn download(&self, cid: &str) -> Result<String> {
        // TODO: Implement actual IPFS download
        let url = format!("{}/ipfs/{}", self.gateway, cid);
        let resp = self.client.get(&url).send().await?;
        let content = resp.text().await?;
        Ok(content)
    }
}
