#![cfg(feature = "audio")]
mod support;
use serde_json::{Value, json};
use std::time::{Duration, Instant};
use support::{
    Harness,
    audio::{applied, control},
    group::{id, until},
    loops, runtime,
};

fn command(state: &Value, operation: &Value) -> Value {
    json!({"kind":"submit","expectedRevision":state["revision"],"action":{
        "kind":"media","group":id(2),"generation":state["media"][0]["generation"],"action":operation
    }})
}
async fn operate(h: &Harness, session: &str, serial: u64, operation: Value) -> Value {
    let accepted = control(h, session, serial, &h.state().await, operation).await;
    assert_eq!(accepted["kind"], "accepted", "{accepted}");
    applied(h, &accepted).await
}
async fn probe(
    h: &Harness,
    session: &str,
    serial: u64,
    label: &str,
    command: Value,
) -> (u16, Value) {
    let before = h.state().await;
    let request = json!({"serial":serial.to_string(),"ttlMs":5000,"command":command});
    let response = h
        .post(&format!("/sessions/{session}/commands"))
        .json(&request)
        .send()
        .await
        .unwrap();
    let status = response.status().as_u16();
    let mut body = response.json::<Value>().await.unwrap();
    let deadline = Instant::now() + Duration::from_secs(7);
    while body["status"] == "pending" {
        assert!(Instant::now() < deadline, "same receipt never completed");
        body = h
            .http
            .get(h.url(&format!("/sessions/{session}/receipts/{serial}")))
            .bearer_auth(h.discovery["controlToken"].as_str().unwrap())
            .send()
            .await
            .unwrap()
            .json::<Value>()
            .await
            .unwrap();
        if body["status"] == "pending" {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
    let after = h.state().await;
    eprintln!(
        "AUDIO022 {}",
        json!({"case":label,"request":request,"http":status,"response":body,
            "before":{"audio":before["audio"],"media":before["media"]},
            "after":{"audio":after["audio"],"media":after["media"]},"controlPosts":1})
    );
    assert_eq!(
        after["audio"], before["audio"],
        "rejected target changed audio"
    );
    assert_eq!(
        after["media"], before["media"],
        "rejected target changed media"
    );
    (status, body)
}
fn classified_refusal((status, body): (u16, Value), code: &str, message: &str) {
    assert_eq!(status, 200);
    assert_eq!(body["status"], "complete");
    assert_eq!(body["outcome"]["kind"], "rejected");
    assert_eq!(body["outcome"]["code"], code);
    assert_eq!(body["outcome"]["message"], message);
    assert!(body["outcome"]["state"].is_null());
}
fn invalid_refusal(result: (u16, Value)) {
    classified_refusal(result, "invalid", "请求格式、参数或长度不正确");
}
fn loop_refusal(result: (u16, Value)) {
    classified_refusal(
        result,
        "loopTargetChanged",
        "循环播放目标已变化，请确认当前区段和遍次后重新操作",
    );
}

async fn stale_target_probes(h: &Harness, session: &str, current: &Value, original_target: &Value) {
    loop_refusal(
        probe(
            h,
            session,
            7,
            "stale-pass",
            command(current, original_target),
        )
        .await,
    );
    let mut wrong_instance = loops::exit(current, true);
    let instance = current["audio"]["instance"]
        .as_str()
        .unwrap()
        .parse::<u64>()
        .unwrap();
    wrong_instance["instance"] = (instance + 1).to_string().into();
    loop_refusal(
        probe(
            h,
            session,
            8,
            "wrong-instance",
            command(current, &wrong_instance),
        )
        .await,
    );
    let mut invalid_region = loops::exit(current, true);
    invalid_region["region"] = 128.into();
    invalid_refusal(
        probe(
            h,
            session,
            9,
            "invalid-region-range",
            command(current, &invalid_region),
        )
        .await,
    );
    let mut wrong_region = loops::exit(current, true);
    wrong_region["region"] = 1.into();
    loop_refusal(
        probe(
            h,
            session,
            10,
            "wrong-region",
            command(current, &wrong_region),
        )
        .await,
    );
    let mut wrong_generation = command(current, &loops::exit(current, true));
    wrong_generation["action"]["generation"] = "0".into();
    classified_refusal(
        probe(h, session, 11, "wrong-generation", wrong_generation).await,
        "mediaTargetChanged",
        "音乐运行目标已变化，请核对当前状态后重新操作",
    );
}
async fn invalid_and_finished_probes(h: &Harness, session: &str, current: &Value, paused: &Value) {
    let mut malformed = loops::exit(current, true);
    malformed["instance"] = "01".into();
    let (status, body) = probe(
        h,
        session,
        12,
        "malformed-decimal",
        command(current, &malformed),
    )
    .await;
    assert_eq!(status, 422);
    assert_eq!(
        body,
        json!({"code":"invalid","message":"请求格式、参数或长度不正确"})
    );
    // The malformed input did not consume network serial 12. This is a new explicit,
    // different target command, not a retransmission of any rejected exit intention.
    let accepted = operate(h, session, 12, loops::exit(current, true)).await;
    assert_eq!(accepted["audio"]["loopState"]["exitRequested"], true);
    let mut invalid_pass = loops::exit(&accepted, false);
    invalid_pass["pass"] = "0".into();
    invalid_refusal(
        probe(
            h,
            session,
            13,
            "invalid-zero-pass",
            command(&accepted, &invalid_pass),
        )
        .await,
    );
    let cancelled = operate(h, session, 14, loops::exit(&accepted, false)).await;
    assert_eq!(cancelled["audio"]["loopState"]["exitRequested"], false);
    assert_eq!(cancelled["audio"]["instance"], paused["audio"]["instance"]);
    assert_eq!(
        cancelled["audio"]["positionMs"],
        current["audio"]["positionMs"]
    );
    let mut zero_instance = loops::exit(&cancelled, true);
    zero_instance["instance"] = "0".into();
    invalid_refusal(
        probe(
            h,
            session,
            15,
            "invalid-zero-instance",
            command(&cancelled, &zero_instance),
        )
        .await,
    );
    let mut unknown_group = command(&cancelled, &loops::exit(&cancelled, true));
    unknown_group["action"]["group"] = id(99).into();
    invalid_refusal(probe(h, session, 16, "unknown-group", unknown_group).await);
    let mut invalid_with_stale_generation = command(&cancelled, &loops::exit(&cancelled, true));
    invalid_with_stale_generation["action"]["generation"] = "0".into();
    invalid_with_stale_generation["action"]["action"]["pass"] = "0".into();
    invalid_refusal(
        probe(
            h,
            session,
            17,
            "invalid-before-stale-generation",
            invalid_with_stale_generation,
        )
        .await,
    );
    let stopped = operate(h, session, 18, json!({"kind":"stop"})).await;
    assert!(stopped["audio"]["loopState"].is_null());
    loop_refusal(
        probe(
            h,
            session,
            19,
            "loop-no-longer-active",
            command(&stopped, &loops::exit(&cancelled, true)),
        )
        .await,
    );
}

#[test]
fn actual_short_loop_distinguishes_stale_targets_from_malformed_input_without_reposting() {
    runtime().block_on(async {
        let mut h = Harness::prepared(|p| Some(loops::write(p, &json!({"kind":"untilExit"}))));
        let session = h.session().await;
        h.acquire(&session, false).await;
        let paused = operate(
            &h,
            &session,
            2,
            json!({"kind":"seek","positionMs":2500,"playing":false}),
        )
        .await;
        let original_target = loops::exit(&paused, true);
        let applied_exit = operate(&h, &session, 3, original_target.clone()).await;
        assert_eq!(applied_exit["audio"]["loopState"]["exitRequested"], true);
        operate(&h, &session, 4, loops::exit(&paused, false)).await;
        operate(&h, &session, 5, json!({"kind":"play"})).await;
        until(&h, |s| {
            s["state"]["audio"]["loopState"]["pass"]
                .as_str()
                .is_some_and(|p| p.parse::<u64>().is_ok_and(|pass| pass > 1))
        })
        .await;
        let current = operate(&h, &session, 6, json!({"kind":"pause"})).await;
        assert_eq!(current["audio"]["instance"], paused["audio"]["instance"]);
        assert_ne!(
            current["audio"]["loopState"]["pass"],
            paused["audio"]["loopState"]["pass"]
        );
        assert_eq!(current["audio"]["status"], "paused");
        assert_eq!(current["audio"]["loopState"]["exitRequested"], false);
        stale_target_probes(&h, &session, &current, &original_target).await;
        invalid_and_finished_probes(&h, &session, &current, &paused).await;
        h.close().await;
    });
}
