use super::*;
use serde_json::json;
fn session() -> Session {
    let mut root: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    root["entryPoints"] = json!([]);
    let mut s = Session::default();
    s.replace(
        Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap(),
        None,
    );
    s
}
fn request(s: &Session, epoch: u32) -> crate::preview::Request {
    let view = s.document.as_ref().unwrap().view();
    serde_json::from_value(json!({"kind":"beginEffectDraft","generation":s.generation,"epoch":epoch,"sceneId":view.scenes[0].id,"illuminate":true,"effect":{
      "id":"29999999-0000-4000-8000-000000000001","name":"临时效果","enabled":true,"fixtureIds":[view.fixtures[0].id],"periodMs":1000,"spreadDegrees":0,"phaseDegrees":0,"reverse":false,"waveform":"triangle","dutyPercent":50,"channels":[{"attribute":"dimmer","low":0,"high":65535}]}})).unwrap()
}
fn snapshot(s: &mut Session) -> serde_json::Value {
    serde_json::to_value(s.preview(crate::preview::Request::Snapshot).unwrap()).unwrap()
}
#[test]
fn audition_never_changes_document_history_generation_or_recovery() {
    let mut s = session();
    let before = s.document.clone();
    let generation = s.generation;
    let version = s.content_version;
    let old = u32::try_from(snapshot(&mut s)["epoch"].as_u64().unwrap()).unwrap();
    s.preview(request(&s, old)).unwrap();
    let next = snapshot(&mut s);
    let epoch = u32::try_from(next["epoch"].as_u64().unwrap()).unwrap();
    assert!(next["loaded"]["draftEffectId"].is_string());
    assert_eq!(s.document, before);
    assert_eq!(s.generation, generation);
    assert_eq!(s.content_version, version);
    assert!(s.undo.is_empty());
    assert!(s.redo.is_empty());
    assert_eq!(s.checkpoint().document, s.document);
    s.preview(crate::preview::Request::EndEffectDraft { epoch })
        .unwrap();
    assert!(snapshot(&mut s)["loaded"].get("draftEffectId").is_none());
    assert_eq!(s.document, before);
    assert!(s.undo.is_empty());
}
#[test]
fn committed_edit_and_history_invalidate_draft_ownership_without_changing_history_semantics() {
    let mut s = session();
    let before = s.document.clone();
    let epoch = u32::try_from(snapshot(&mut s)["epoch"].as_u64().unwrap()).unwrap();
    s.preview(request(&s, epoch)).unwrap();
    let draft_epoch = u32::try_from(snapshot(&mut s)["epoch"].as_u64().unwrap()).unwrap();
    s.edit(
        s.generation,
        EditCommand::SetInfo {
            name: "已应用工程".into(),
            description: String::new(),
        },
    )
    .unwrap();
    assert_eq!(s.undo.len(), 1);
    assert!(snapshot(&mut s)["loaded"].is_null());
    s.preview(crate::preview::Request::EndEffectDraft { epoch: draft_epoch })
        .unwrap();
    s.history(s.generation, false).unwrap();
    assert_eq!(s.document, before);
    let epoch = u32::try_from(snapshot(&mut s)["epoch"].as_u64().unwrap()).unwrap();
    s.preview(request(&s, epoch)).unwrap();
    s.history(s.generation, true).unwrap();
    assert!(snapshot(&mut s)["loaded"].is_null());
    assert_eq!(s.document.as_ref().unwrap().view().name, "已应用工程");
}
#[test]
fn strict_draft_protocol_rejects_extra_fields_and_generic_edit_payloads() {
    for input in [
        json!({"kind":"endEffectDraft","epoch":0,"generation":1}),
        json!({"kind":"beginEffectDraft","generation":0,"epoch":0,"commands":[]}),
    ] {
        assert!(serde_json::from_value::<crate::preview::Request>(input).is_err());
    }
}

#[test]
fn prepared_draft_cannot_commit_after_project_edit_or_new_playback() {
    let mut s = session();
    let epoch = u32::try_from(snapshot(&mut s)["epoch"].as_u64().unwrap()).unwrap();
    let prepared = s
        .prepare_effect_draft(request(&s, epoch))
        .unwrap()
        .compile()
        .unwrap();
    s.edit(
        s.generation,
        EditCommand::SetInfo {
            name: "新内容".into(),
            description: String::new(),
        },
    )
    .unwrap();
    assert!(s.finish_effect_draft(prepared).is_err());
    assert!(snapshot(&mut s)["loaded"].is_null());
    let epoch = u32::try_from(snapshot(&mut s)["epoch"].as_u64().unwrap()).unwrap();
    let prepared = s
        .prepare_effect_draft(request(&s, epoch))
        .unwrap()
        .compile()
        .unwrap();
    let scene = s.document.as_ref().unwrap().view().scenes[0].id.clone();
    s.preview(crate::preview::Request::LoadScene {
        generation: s.generation,
        scene_id: scene,
    })
    .unwrap();
    assert!(s.finish_effect_draft(prepared).is_err());
    assert_eq!(snapshot(&mut s)["loaded"]["status"], "idle");
}
