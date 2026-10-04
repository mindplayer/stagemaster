use super::*;
use crate::previs::protocol::Source;
use serde_json::{Value, json};
#[path = "../../../../crates/stagemaster-project/tests/stage_mixed_support/mod.rs"]
mod support;

fn targets(values: &[Value]) -> Vec<ViewportTarget> {
    values
        .iter()
        .map(|v| serde_json::from_value(json!({"kind":v["kind"],"id":v["targetId"]})).unwrap())
        .collect()
}
fn delta() -> SpatialVector3 {
    SpatialVector3 {
        x: "1.25".into(),
        y: "0".into(),
        z: "-0.25".into(),
    }
}
#[test]
fn viewport_mixed_edit_matches_core_once_and_round_trips_through_history() {
    let (doc, selected, ids) = support::setup();
    let mut expected = doc.clone();
    support::translate(&mut expected, &selected, ["1.25", "0", "-0.25"]).unwrap();
    let mut session = Session::from_previs_test_document(doc.clone());
    let r = session.previs_revision();
    session
        .translate_objects_from_viewport(
            r.generation,
            &r.content.to_string(),
            targets(&selected),
            delta(),
        )
        .unwrap();
    let actual = session.previs_document().unwrap();
    assert_eq!(support::root(&actual), support::root(&expected));
    let placements = actual.view().stage.placements;
    assert_eq!(
        placements
            .iter()
            .find(|p| p.fixture_id == ids[0])
            .unwrap()
            .position_meters
            .x,
        "1.25"
    );
    assert_eq!(session.undo.len(), 1);
    assert!(!session.previs_frame().unwrap().can_edit);
    assert!(
        session
            .translate_objects_from_viewport(
                r.generation,
                &r.content.to_string(),
                targets(&selected),
                delta()
            )
            .is_err()
    );
    assert_eq!(session.undo.len(), 1);
    session.history(session.generation, false).unwrap();
    assert_eq!(session.previs_document().unwrap(), doc);
    session.history(session.generation, true).unwrap();
    assert_eq!(session.previs_document().unwrap(), actual);
    assert_eq!(
        stagemaster_project::Document::decode(&actual.encode().unwrap()).unwrap(),
        actual
    );
}
#[test]
fn stale_context_invalid_members_and_indirect_locks_cannot_partially_move_a_group() {
    let (doc, selected, ids) = support::setup();
    let mut session = Session::from_previs_test_document(doc.clone());
    let r = session.previs_revision();
    for (generation, version) in [(r.generation + 1, r.content), (r.generation, r.content + 1)] {
        assert!(
            session
                .translate_objects_from_viewport(
                    generation,
                    &version.to_string(),
                    targets(&selected),
                    delta()
                )
                .is_err()
        );
    }
    for source in [
        Source::Background {
            host_id: "elsewhere".into(),
        },
        Source::Scene {
            scene_id: "deleted".into(),
        },
        Source::Playback,
    ] {
        session.previs_source = source;
        assert!(
            session
                .translate_objects_from_viewport(
                    r.generation,
                    &r.content.to_string(),
                    targets(&selected),
                    delta()
                )
                .is_err()
        );
    }
    session.previs_source = Source::Defaults;
    let mut missing = targets(&selected);
    missing.push(ViewportTarget::Construction {
        id: "missing".into(),
    });
    for selection in [vec![], missing, vec![targets(&selected)[0].clone(); 2]] {
        assert!(
            session
                .translate_objects_from_viewport(
                    r.generation,
                    &r.content.to_string(),
                    selection,
                    delta()
                )
                .is_err()
        );
    }
    assert_eq!(session.previs_document().unwrap(), doc);
    assert!(session.undo.is_empty());
    let mut locked = doc;
    support::edit(&mut locked, json!({"op":"setEditLocks","targets":[{"kind":"placement","targetId":ids[1]}],"locked":true})).unwrap();
    let mut session = Session::from_previs_test_document(locked.clone());
    let r = session.previs_revision();
    assert!(
        session
            .translate_objects_from_viewport(
                r.generation,
                &r.content.to_string(),
                targets(&selected),
                delta()
            )
            .unwrap_err()
            .contains("锁定")
    );
    assert_eq!(session.previs_document().unwrap(), locked);
    assert!(session.undo.is_empty());
}
#[test]
fn viewport_targets_do_not_accept_spaces_or_unrecognized_fields() {
    for value in [
        json!({"kind":"space","id":"room"}),
        json!({"kind":"placement","targetId":"lamp"}),
        json!({"kind":"construction","id":"rig","execute":true}),
    ] {
        assert!(serde_json::from_value::<ViewportTarget>(value).is_err());
    }
}
