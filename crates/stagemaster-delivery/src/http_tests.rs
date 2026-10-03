use super::*;
#[path = "../tests/support/fixture.rs"]
mod fixture;
#[path = "../tests/support/https.rs"]
mod https;
use std::{future::pending, sync::atomic::AtomicBool};

fn http() -> Http {
    http_timeout(Duration::from_secs(30))
}
fn http_timeout(timeout: Duration) -> Http {
    let client = client()
        .no_proxy()
        .add_root_certificate(reqwest::Certificate::from_der(&https::credentials().ca).unwrap())
        .timeout(timeout)
        .build()
        .unwrap();
    Http { client }
}
fn expected(bytes: &[u8]) -> Identity {
    Package::from_bytes(bytes.into(), None).unwrap().identity()
}
#[tokio::test]
async fn actual_https_and_file_produce_the_same_content_then_survive_source_loss() {
    let bytes = fixture::bytes(12000);
    let identity = expected(&bytes);
    let server = https::Server::start(https::response(
        "200 OK",
        &format!("Content-Length: {}\r\n", bytes.len()),
        &bytes,
    ))
    .await;
    let remote = http()
        .download(&server.url, identity, pending())
        .await
        .unwrap();
    drop(server);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("from-usb.smpkg");
    std::fs::write(&path, &bytes).unwrap();
    let local = crate::from_file(&path, Some(identity), &AtomicBool::new(false)).unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(remote.identity(), local.identity());
    assert_eq!(remote.archive().source(), local.archive().source());
    assert_eq!(
        remote
            .archive()
            .load(&remote, 0)
            .unwrap()
            .plan
            .steps()
            .len(),
        local.archive().load(&local, 0).unwrap().plan.steps().len()
    );
}
#[tokio::test]
async fn redirects_errors_wrong_length_truncation_encoding_and_oversize_are_rejected() {
    let bytes = fixture::bytes(12000);
    let id = expected(&bytes);
    let huge = format!("Content-Length: {}\r\n", usize::MAX);
    let cases = [
        https::response("302 Found", "Location: https://localhost/other\r\n", &[]),
        https::response("404 Not Found", "", &[]),
        https::response("200 OK", &huge, &[]),
        https::response(
            "200 OK",
            &format!("Content-Length: {}\r\n", bytes.len()),
            &bytes[..100],
        ),
        https::response("200 OK", "Content-Encoding: gzip\r\n", &bytes),
        https::response("200 OK", "", &[bytes.clone(), vec![0]].concat()),
    ];
    for response in cases {
        let server = https::Server::start(response).await;
        assert!(http().download(&server.url, id, pending()).await.is_err());
    }
    let server = https::Server::start(https::response("200 OK", "", &bytes)).await;
    let mut wrong = id;
    wrong.digest[0] ^= 1;
    assert!(matches!(
        http().download(&server.url, wrong, pending()).await,
        Err(Error::Identity)
    ));
}
#[tokio::test]
async fn cancellation_timeout_and_tls_checks_do_not_expose_signed_addresses() {
    let id = expected(&fixture::bytes(12000));
    let server = https::Server::hanging().await;
    assert!(matches!(
        http().download(&server.url, id, async {}).await,
        Err(Error::Cancelled)
    ));
    assert!(matches!(
        http_timeout(Duration::from_millis(400))
            .download(&server.url, id, pending())
            .await,
        Err(Error::Timeout)
    ));
    let server = https::Server::start(https::response("200 OK", "", &[])).await;
    let address = format!("{}?secret=test-token", server.url);
    let Err(error) = Http::new().unwrap().download(&address, id, pending()).await else {
        panic!("untrusted CA accepted");
    };
    assert!(matches!(error, Error::Network));
    assert!(!error.to_string().contains("test-token"));
    for url in [
        "http://127.0.0.1/data",
        "https://user:pass@localhost/data",
        "https://localhost/data#part",
    ] {
        assert!(matches!(
            http().download(url, id, pending()).await,
            Err(Error::Source)
        ));
    }
}

#[path = "http_tests/lifecycle.rs"]
mod lifecycle;
#[path = "http_tests/runtime.rs"]
mod runtime;

#[tokio::test]
async fn chunked_delivery_and_cancellation_after_partial_body_use_the_same_ingress() {
    let bytes = fixture::bytes(12000);
    let identity = expected(&bytes);
    let mut body = Vec::new();
    for chunk in bytes.chunks(37) {
        body.extend_from_slice(format!("{:x}\r\n", chunk.len()).as_bytes());
        body.extend_from_slice(chunk);
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(b"0\r\n\r\n");
    let server = https::Server::start(https::response(
        "200 OK",
        "Transfer-Encoding: chunked\r\n",
        &body,
    ))
    .await;
    assert_eq!(
        http()
            .download(&server.url, identity, pending())
            .await
            .unwrap()
            .identity(),
        identity
    );
    let mut server = https::Server::partial(https::response(
        "200 OK",
        &format!("Content-Length: {}\r\n", bytes.len()),
        &bytes[..100],
    ))
    .await;
    let sent = server.sent.take().unwrap();
    let cancelled = async move {
        sent.await.unwrap();
    };
    assert!(matches!(
        http().download(&server.url, identity, cancelled).await,
        Err(Error::Cancelled)
    ));
}
