//! Isolated TLS server using short-lived test credentials; never changes system trust.
mod certificate;
pub use certificate::credentials;
use std::{sync::Arc, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    task::JoinHandle,
};
use tokio_rustls::{
    TlsAcceptor,
    rustls::{
        ServerConfig,
        pki_types::{CertificateDer, PrivatePkcs8KeyDer},
    },
};
pub struct Server {
    pub url: String,
    task: JoinHandle<()>,
    pub sent: Option<tokio::sync::oneshot::Receiver<()>>,
}
impl Server {
    pub async fn start(bytes: Vec<u8>) -> Self {
        Self::spawn(Some(bytes), false).await
    }
    pub async fn hanging() -> Self {
        Self::spawn(None, true).await
    }
    pub async fn partial(bytes: Vec<u8>) -> Self {
        Self::spawn(Some(bytes), true).await
    }
    async fn spawn(response: Option<Vec<u8>>, stall: bool) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let certificate = CertificateDer::from(credentials().certificate.clone());
        let key = PrivatePkcs8KeyDer::from(credentials().key.clone());
        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![certificate], key.into())
            .unwrap();
        let (sent, received) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let Ok(mut stream) = TlsAcceptor::from(Arc::new(config)).accept(stream).await else {
                return;
            };
            let mut request = Vec::new();
            while !request.ends_with(b"\r\n\r\n") && request.len() < 8192 {
                match stream.read_u8().await {
                    Ok(byte) => request.push(byte),
                    Err(_) => return,
                }
            }
            if let Some(bytes) = response {
                let _ = stream.write_all(&bytes).await;
            }
            let _ = sent.send(());
            if stall {
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
            let _ = stream.shutdown().await;
        });
        Self {
            url: format!("https://{address}/show.smpkg"),
            task,
            sent: Some(received),
        }
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.task.abort();
    }
}
pub fn response(status: &str, headers: &str, body: &[u8]) -> Vec<u8> {
    [
        format!("HTTP/1.1 {status}\r\nConnection: close\r\n{headers}\r\n").into_bytes(),
        body.to_vec(),
    ]
    .concat()
}
