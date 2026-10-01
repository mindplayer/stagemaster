use super::*;
use stagemaster_project::{FixtureEdit, ProfileFile};

#[test]
fn inspected_mode_does_not_edit_until_acceptance_and_import_is_one_history_entry() {
    let mut d = Document::new("模式导入").unwrap();
    let p = d.view();
    d.edit(EditCommand::AddFixture {
        name: "原灯".into(),
        profile_id: p.profiles[0].id.clone(),
        domain_id: p.domains[0].id.clone(),
        universe: 1,
        address: 1,
    })
    .unwrap();
    d.edit(EditCommand::AddScene {
        name: "原场景".into(),
    })
    .unwrap();
    let original = d.view();
    let source_id = original.fixtures[0].profile_id.clone();
    let mut s = Session::default();
    s.replace(d.clone(), None);
    s.preview(crate::preview::Request::LoadScene {
        generation: s.generation,
        scene_id: original.scenes[0].id.clone(),
    })
    .unwrap();
    let before = serde_json::to_value(s.snapshot()).unwrap();
    let playback =
        serde_json::to_value(s.preview(crate::preview::Request::Snapshot).unwrap()).unwrap();
    let captured = s
        .check_snapshot(s.generation)
        .unwrap()
        .profile_file(&source_id)
        .unwrap();
    let inspected = ProfileFile::decode(&captured.encode().unwrap()).unwrap();
    assert_eq!(before, serde_json::to_value(s.snapshot()).unwrap());
    let generation = s.generation;
    let command = || EditCommand::Fixture {
        command: FixtureEdit::SaveProfile {
            id: None,
            definition: Box::new(inspected.definition().clone()),
        },
    };
    assert!(s.edit(generation - 1, command()).is_err());
    s.edit(generation, command()).unwrap();
    assert_eq!(s.undo.len(), 1);
    let after = s.document.clone().unwrap();
    assert_eq!(after.view().fixtures[0].profile_id, source_id);
    assert_ne!(after.view().profiles.last().unwrap().id, source_id);
    let playing =
        serde_json::to_value(s.preview(crate::preview::Request::Snapshot).unwrap()).unwrap();
    assert_eq!(playback["loaded"]["sceneId"], original.scenes[0].id);
    assert_eq!(playing["epoch"], playback["epoch"]);
    assert_eq!(playing["loaded"]["sceneId"], playback["loaded"]["sceneId"]);
    assert_eq!(playing["loaded"]["status"], playback["loaded"]["status"]);
    assert_eq!(playing["loaded"]["stale"], true);
    s.history(s.generation, false).unwrap();
    assert_eq!(s.document.as_ref(), Some(&d));
    assert!(s.undo.is_empty());
    s.history(s.generation, true).unwrap();
    assert_eq!(s.document.as_ref(), Some(&after));
}
