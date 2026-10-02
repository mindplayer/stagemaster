mod support;
use reqwest::StatusCode;
use serde_json::json;
use std::time::Duration;
use support::*;

#[test]
fn local_authorization_generation_and_body_limits_are_enforced() {
    runtime().block_on(local_authorization_generation_and_body_limits_are_enforced_async());
}
async fn local_authorization_generation_and_body_limits_are_enforced_async() {
    let mut h = Harness::start();
    assert_access_restrictions(&h).await;
    assert_request_limits(&h).await;
    let previous = h.discovery.clone();
    h.close().await;
    assert_new_generation(&previous).await;
}

async fn assert_access_restrictions(h: &Harness) {
    assert_eq!(
        h.http.get(h.url("/state")).send().await.unwrap().status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        h.get("/state")
            .header("Origin", "https://example.test")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        h.get("/state")
            .header("Authorization", "Bearer another")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        h.http
            .post(h.url("/sessions"))
            .bearer_auth(h.discovery["readToken"].as_str().unwrap())
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
}

async fn assert_request_limits(h: &Harness) {
    let session = h.session().await;
    let oversized = h
        .post(&format!("/sessions/{session}/commands"))
        .header("Content-Type", "application/json")
        .body(" ".repeat(8193))
        .send()
        .await
        .unwrap();
    assert_eq!(oversized.status(), StatusCode::PAYLOAD_TOO_LARGE);
    for path in ["/sessions", "/shutdown"] {
        assert_eq!(
            h.post(path)
                .body(" ".repeat(8193))
                .send()
                .await
                .unwrap()
                .status(),
            StatusCode::PAYLOAD_TOO_LARGE
        );
    }
    for body in [
        json!({"serial":"1","ttlMs":5001,"command":{"kind":"release"}}),
        json!({"serial":1,"ttlMs":50,"command":{"kind":"release"}}),
        json!({"serial":"1","ttlMs":100,"command":{"kind":"acquire","durationMs":1000,"takeover":false,"lease":"forged"}}),
    ] {
        assert_eq!(
            h.post(&format!("/sessions/{session}/commands"))
                .json(&body)
                .send()
                .await
                .unwrap()
                .status(),
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    let one = h.command(&session, 1, json!({"kind":"release"})).await;
    assert_eq!(one["code"], "noControl");
    let conflict = h
        .post(&format!("/sessions/{session}/commands"))
        .json(&json!({
            "serial": "1", "ttlMs": 5000,
            "command": {"kind": "acquire", "durationMs": 1000, "takeover": false}
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(conflict.status(), StatusCode::CONFLICT);
    for _ in 1..8 {
        h.session().await;
    }
    assert_eq!(
        h.post("/sessions").send().await.unwrap().status(),
        StatusCode::CONFLICT
    );
}

async fn assert_new_generation(previous: &serde_json::Value) {
    let mut fresh = Harness::start();
    assert_ne!(previous["hostId"], fresh.discovery["hostId"]);
    assert_ne!(previous["controlToken"], fresh.discovery["controlToken"]);
    assert_ne!(previous["readToken"], fresh.discovery["readToken"]);
    assert_eq!(
        fresh
            .http
            .get(fresh.url("/state"))
            .bearer_auth(previous["readToken"].as_str().unwrap())
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
    let old_path = fresh.url("/state").replace(
        fresh.discovery["hostId"].as_str().unwrap(),
        previous["hostId"].as_str().unwrap(),
    );
    let stale = fresh
        .http
        .get(old_path)
        .bearer_auth(fresh.discovery["readToken"].as_str().unwrap())
        .send()
        .await
        .unwrap();
    assert_eq!(stale.status(), StatusCode::CONFLICT);
    fresh.close().await;
}

#[test]
fn incomplete_connections_expire_without_stalling_the_runtime() {
    runtime().block_on(incomplete_connections_expire_without_stalling_the_runtime_async());
}
async fn incomplete_connections_expire_without_stalling_the_runtime_async() {
    use tokio::io::AsyncWriteExt;
    let mut h = Harness::start();
    let id = h.session().await;
    let acquired = h.acquire(&id, false).await;
    let started = h
        .start_program(&id, &acquired["state"]["revision"], 2)
        .await;
    let url = reqwest::Url::parse(h.discovery["url"].as_str().unwrap()).unwrap();
    let address = (url.host_str().unwrap(), url.port().unwrap());
    let mut sockets = Vec::new();
    for _ in 0..16 {
        let mut socket = tokio::net::TcpStream::connect(address).await.unwrap();
        socket
            .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n")
            .await
            .unwrap();
        sockets.push(socket);
    }
    tokio::time::sleep(Duration::from_millis(150)).await;
    let observed = h.state().await; // Queued until an incomplete connection expires.
    assert_eq!(observed["instance"], started["state"]["instance"]);
    assert_eq!(observed["status"], "Running");
    assert!(
        observed["observedMs"]
            .as_str()
            .unwrap()
            .parse::<u64>()
            .unwrap()
            >= 4000
    );
    assert!(
        observed["elapsedMs"]
            .as_str()
            .unwrap()
            .parse::<u64>()
            .unwrap()
            > 100
    );
    drop(sockets);
    h.close().await;
}
