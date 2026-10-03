use super::*;
use crate::rigging_preview::{Request, project};
use serde_json::{Value, json};

fn setup() -> (Session, String, Vec<String>) {
    let mut document = Document::new("挂接草稿").unwrap();
    let v = document.view();
    for i in 0..3 {
        document
            .edit(EditCommand::AddFixture {
                name: format!("灯{i}"),
                profile_id: v.profiles[1].id.clone(),
                domain_id: v.domains[0].id.clone(),
                universe: 1,
                address: 1 + i * 4,
            })
            .unwrap();
    }
    document.edit(serde_json::from_value(json!({"op":"stage","command":{
        "op":"putConstruction","id":null,"name":"侧桁架","shape":{
            "kind":"rig","rigKind":"truss","spaceId":null,"positionMeters":{"x":"4","y":"3","z":"5"},
            "yawDegrees":"90","lengthMeters":"6","widthMeters":"0.3","heightMeters":"0.4"
        }
    }})).unwrap()).unwrap();
    let view = document.view();
    let mut session = Session::default();
    session.replace(document, None);
    session.saved = session.document.clone();
    (
        session,
        view.stage.constructions[0].id.clone(),
        view.fixtures.iter().map(|f| f.id.clone()).collect(),
    )
}
fn command(rig: &str, ids: &[String]) -> Value {
    json!({"op":"attachFixtures","constructionId":rig,"fixtureIds":ids,"layout":{
        "startMarginMeters":"1","endMarginMeters":"1","dropMeters":"0.2"
    }})
}
fn preview(s: &Session, command: &Value) -> Result<Value, String> {
    let request: Request =
        serde_json::from_value(json!({"generation":s.generation,"command":command})).unwrap();
    project(s.check_snapshot(s.generation)?, request).map(|p| serde_json::to_value(p).unwrap())
}
#[test]
fn rigging_projection_is_the_actual_rotated_edit_without_history_or_content_change() {
    let (mut s, rig, mut ids) = setup();
    ids.reverse();
    let before = serde_json::to_value(s.snapshot()).unwrap();
    let document = s.document.clone();
    let c = command(&rig, &ids);
    let result = preview(&s, &c).unwrap();
    assert_eq!(result["changed"], true);
    assert_eq!(serde_json::to_value(s.snapshot()).unwrap(), before);
    assert_eq!(s.document, document);
    let points = result["placements"].as_array().unwrap();
    assert_eq!(
        points[0]["positionMeters"],
        json!({"x":"4","y":"1","z":"4.6"})
    );
    assert_eq!(
        points[2]["positionMeters"],
        json!({"x":"4","y":"5","z":"4.6"})
    );
    s.edit(
        s.generation,
        serde_json::from_value(json!({"op":"stage","command":c})).unwrap(),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(s.document.as_ref().unwrap().view().stage.placements).unwrap(),
        result["placements"]
    );
    assert_eq!(s.undo.len(), 1);
    s.history(s.generation, false).unwrap();
    assert_eq!(s.document, document);
    assert!(s.check_snapshot(s.generation - 1).is_err());
}
#[test]
fn keep_position_noop_ignores_attachment_order_and_rejects_missing_placement() {
    let (mut s, rig, ids) = setup();
    let mut keep = command(&rig, &ids[..1]);
    keep["layout"] = Value::Null;
    assert!(preview(&s, &keep).unwrap_err().contains("已有灯位"));
    s.edit(
        s.generation,
        serde_json::from_value(json!({"op":"stage","command":command(&rig, &ids)})).unwrap(),
    )
    .unwrap();
    let result = preview(&s, &keep).unwrap();
    assert_eq!(result["changed"], false);
    let mut detach = command(&rig, &ids[..1]);
    detach["layout"] = Value::Null;
    detach["constructionId"] = Value::Null;
    assert_eq!(preview(&s, &detach).unwrap()["changed"], true);
}
#[test]
fn preview_rejects_locks_bad_margins_and_unknown_edits_without_mutation() {
    let (mut s, rig, ids) = setup();
    let mut invalid = command(&rig, &ids);
    invalid["layout"]["endMarginMeters"] = "6".into();
    assert!(preview(&s, &invalid).unwrap_err().contains("余量"));
    assert!(
        preview(
            &s,
            &json!({"op":"removeConstruction","id":rig,"detachFixtures":true})
        )
        .is_err()
    );
    s.edit(
        s.generation,
        serde_json::from_value(json!({"op":"stage","command":command(&rig,&ids)})).unwrap(),
    )
    .unwrap();
    s.edit(s.generation, serde_json::from_value(json!({"op":"stage","command":{"op":"setEditLocks","targets":[{"kind":"placement","targetId":ids[0]}],"locked":true}})).unwrap()).unwrap();
    let before = serde_json::to_value(s.snapshot()).unwrap();
    let mut invalid = command(&rig, &ids);
    invalid["layout"]["dropMeters"] = "1".into();
    assert!(preview(&s, &invalid).unwrap_err().contains("锁定"));
    assert_eq!(serde_json::to_value(s.snapshot()).unwrap(), before);
    assert!(
        serde_json::from_value::<Request>(
            json!({"generation":s.generation,"command":command(&rig,&ids),"extra":true})
        )
        .is_err()
    );
}

#[test]
fn rigging_projection_does_not_replace_or_stale_a_loaded_preview() {
    let (mut s, rig, ids) = setup();
    s.edit(
        s.generation,
        EditCommand::AddScene {
            name: "现场".into(),
        },
    )
    .unwrap();
    let scene = s.document.as_ref().unwrap().view().scenes[0].id.clone();
    s.preview(crate::preview::Request::LoadScene {
        generation: s.generation,
        scene_id: scene,
    })
    .unwrap();
    let before =
        serde_json::to_value(s.preview(crate::preview::Request::Snapshot).unwrap()).unwrap();
    let history = s.undo.len();
    preview(&s, &command(&rig, &ids)).unwrap();
    let after =
        serde_json::to_value(s.preview(crate::preview::Request::Snapshot).unwrap()).unwrap();
    assert_eq!(before, after);
    assert_eq!(s.undo.len(), history);
}
