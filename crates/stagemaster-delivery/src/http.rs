use crate::{Error, Incoming, Package};
use reqwest::{Client, StatusCode, Url, header, redirect::Policy};
use stagemaster_install::Identity;
use std::{future::Future, time::Duration};

/// HTTP is a replaceable acquisition adapter, never an authority or a content identity.
pub struct Http {
    client: Client,
}
impl Http {
    /// # Errors
    /// Report inability to initialize the verified-TLS client without exposing source addresses.
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            client: client().build().map_err(|_| Error::Network)?,
        })
    }
    /// Caller supplies an explicit HTTPS address and independently selected content identity.
    /// Dropping this future or resolving `cancel` abandons all incomplete content.
    /// # Errors
    /// Reject other schemes, credentials in URLs, fragments, redirects, compression, wrong
    /// lengths, failed TLS/HTTP, timeout, cancellation and mismatched or unsupported packages.
    pub async fn download(
        &self,
        address: &str,
        expected: Identity,
        cancel: impl Future<Output = ()>,
    ) -> Result<Package, Error> {
        let url = Url::parse(address).map_err(|_| Error::Source)?;
        if url.scheme() != "https"
            || !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
            || url.host_str().is_none()
        {
            return Err(Error::Source);
        }
        let incoming = Incoming::new(Some(expected))?;
        tokio::select! {
            biased;
            () = cancel => Err(Error::Cancelled),
            result = self.receive(url, expected, incoming) => result,
        }
    }
    async fn receive(
        &self,
        url: Url,
        expected: Identity,
        mut incoming: Incoming,
    ) -> Result<Package, Error> {
        let mut response = self
            .client
            .get(url)
            .header(header::ACCEPT_ENCODING, "identity")
            .send()
            .await
            .map_err(network_error)?;
        if response.status() != StatusCode::OK {
            return Err(Error::Source);
        }
        if let Some(encoding) = response.headers().get(header::CONTENT_ENCODING)
            && !encoding.as_bytes().eq_ignore_ascii_case(b"identity")
        {
            return Err(Error::Source);
        }
        if response
            .content_length()
            .is_some_and(|length| length != expected.bytes as u64)
        {
            return Err(Error::Identity);
        }
        while let Some(bytes) = response.chunk().await.map_err(network_error)? {
            incoming.push(&bytes)?;
        }
        tokio::task::spawn_blocking(move || incoming.finish())
            .await
            .map_err(|_| Error::Source)?
    }
}
fn client() -> reqwest::ClientBuilder {
    Client::builder()
        .https_only(true)
        .redirect(Policy::none())
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .no_gzip()
        .no_brotli()
        .no_deflate()
        .no_zstd()
}
fn network_error(error: reqwest::Error) -> Error {
    let error = error.without_url();
    if error.is_timeout() {
        Error::Timeout
    } else {
        Error::Network
    }
}

#[cfg(test)]
#[path = "http_tests.rs"]
mod tests;
