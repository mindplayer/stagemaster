use super::*;
use stagemaster_project::EffectTemplateFile;
#[test]
fn template_apply_is_one_history_entry_and_does_not_start_or_replace_playback() {
    let mut doc = Document::new("灯效模板验收").unwrap();
    let v = doc.view();
    doc.edit(EditCommand::AddFixture {
        name: "帕灯".into(),
        profile_id: v.profiles[0].id.clone(),
        domain_id: v.domains[0].id.clone(),
        universe: 1,
        address: 31,
    })
    .unwrap();
    doc.edit(EditCommand::AddScene {
        name: "编排".into(),
    })
    .unwrap();
    let v = doc.view();
    let scene = v.scenes[0].id.clone();
    let ids = vec![v.fixtures[0].id.clone()];
    let template = EffectTemplateFile::create(
        serde_json::from_value(serde_json::json!({
            "name": "亮度呼吸",
            "recipe": {
                "kind": "intensity-wave", "waveform": "smooth",
                "low": 0, "high": 65535, "dutyPercent": 50
            },
            "timing": {
                "periodMs": 2000, "phaseDegrees": 0,
                "spreadDegrees": 0, "reverseOrder": false
            }
        }))
        .unwrap(),
    )
    .unwrap();
    let mut session = Session::default();
    session.replace(doc.clone(), None);
    session
        .preview(crate::preview::Request::LoadScene {
            generation: session.generation,
            scene_id: scene.clone(),
        })
        .unwrap();
    let playing =
        serde_json::to_value(session.preview(crate::preview::Request::Snapshot).unwrap()).unwrap();
    let review = doc.review_effect_template(&template, &scene, &ids).unwrap();
    assert!(
        session
            .apply_effect_template(session.generation - 1, review)
            .is_err()
    );
    assert!(session.undo.is_empty());
    let review = doc.review_effect_template(&template, &scene, &ids).unwrap();
    session
        .apply_effect_template(session.generation, review)
        .unwrap();
    assert_eq!(session.undo.len(), 1);
    let after = session.document.clone().unwrap();
    let preview =
        serde_json::to_value(session.preview(crate::preview::Request::Snapshot).unwrap()).unwrap();
    assert_eq!(preview["epoch"], playing["epoch"]);
    assert_eq!(preview["loaded"]["status"], playing["loaded"]["status"]);
    assert_eq!(preview["loaded"]["stale"], true);
    session.history(session.generation, false).unwrap();
    assert_eq!(session.document.as_ref(), Some(&doc));
    session.history(session.generation, true).unwrap();
    assert_eq!(session.document.as_ref(), Some(&after));
}
