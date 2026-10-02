use super::discovery::Discovery;
use reqwest::{Client, Method};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::time::Duration;

pub(crate) struct Transport {
    pub discovery: Discovery,
    client: Client,
}
impl Transport {
    pub fn new(discovery: Discovery) -> Result<Self, String> {
        let client = Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(1))
            .timeout(Duration::from_secs(4))
            .build()
            .map_err(|_| "无法建立后台客户端")?;
        Ok(Self { discovery, client })
    }
    pub async fn request<T: DeserializeOwned>(
        &self,
        method: Method,
        suffix: &str,
        body: Option<Value>,
    ) -> Result<T, String> {
        let bytes = self.bytes(method, suffix, body).await?;
        serde_json::from_slice(&bytes).map_err(|_| "后台响应格式无效".into())
    }
    pub async fn bytes(
        &self,
        method: Method,
        suffix: &str,
        body: Option<Value>,
    ) -> Result<Vec<u8>, String> {
        let control = method != Method::GET;
        let mut request = self
            .client
            .request(method, format!("{}{suffix}", self.discovery.url))
            .bearer_auth(if control {
                &self.discovery.control_token
            } else {
                &self.discovery.read_token
            });
        if let Some(body) = body {
            let data = serde_json::to_vec(&body).map_err(|_| "后台请求编码失败")?;
            if data.len() > 8192 {
                return Err("后台请求超过 8 KiB".into());
            }
            request = request
                .header("content-type", "application/json")
                .body(data);
        }
        let mut response = request
            .send()
            .await
            .map_err(|_| "后台连接未响应；已发送操作须核对原回执")?;
        let status = response.status();
        let mut bytes = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| "后台响应不完整，请核对原回执")?
        {
            if bytes.len().saturating_add(chunk.len()) > 8 * 1024 * 1024 {
                return Err("后台响应超过容量限制".into());
            }
            bytes.extend_from_slice(&chunk);
        }
        if !status.is_success() {
            return Err(format!(
                "后台请求未成功（{}），请核对连接与原回执",
                status.as_u16()
            ));
        }
        Ok(bytes)
    }
}
