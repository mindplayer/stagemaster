use super::*;
use stagemaster_project::{Document, EditCommand};
use std::fmt::Write;
fn setup() -> (Document, crate::execution::Collected) {
    let mut document = Document::new("手动录入").unwrap();
    let view = document.view();
    document
        .edit(EditCommand::AddFixture {
            name: "灯".into(),
            profile_id: view.profiles[0].id.clone(),
            domain_id: view.domains[0].id.clone(),
            universe: 1,
            address: 1,
        })
        .unwrap();
    document
        .edit(EditCommand::AddScene {
            name: "原场景".into(),
        })
        .unwrap();
    let view = document.view();
    let layout = document
        .compile_live_scene(&view.scenes[0].id, 0)
        .unwrap()
        .layout()
        .id()
        .iter()
        .fold(String::new(), |mut text, b| {
            write!(text, "{b:02x}").unwrap();
            text
        });
    let capture = document
        .capture_manual_scene(
            &layout,
            vec![ManualSceneReading {
                fixture_id: view.fixtures[0].id.clone(),
                attribute: "dimmer".into(),
                value: 0,
            }],
        )
        .unwrap();
    (
        document,
        crate::execution::Collected {
            capture,
            fixtures: vec![],
            source_name: "手动层".into(),
            revision: "4".into(),
        },
    )
}
#[test]
fn tickets_are_exact_expiring_retryable_and_consumed_only_on_success() {
    let service = Service::default();
    let (mut document, collected) = setup();
    let capture = collected.capture.clone();
    let original = document.clone();
    let first = service.prepare(1, collected).unwrap();
    assert!(
        service
            .apply(2, &first.token, |_| panic!("wrong generation"))
            .is_err()
    );
    assert!(service.apply(1, "bad", |_| panic!("wrong ticket")).is_err());
    assert!(
        service
            .apply(1, &first.token, |c| document
                .record_manual_scene(c, "原场景")
                .map(|_| ()))
            .is_err()
    );
    assert_eq!(document, original);
    service
        .apply(1, &first.token, |c| {
            document.record_manual_scene(c, "录入场景").map(|_| ())
        })
        .unwrap();
    assert_eq!(document.view().scenes.last().unwrap().values.len(), 1);
    assert!(
        service
            .apply(1, &first.token, |_| panic!("replay"))
            .is_err()
    );
    let second = service
        .prepare(
            2,
            crate::execution::Collected {
                capture: capture.clone(),
                fixtures: vec![],
                source_name: "手动".into(),
                revision: "5".into(),
            },
        )
        .unwrap();
    service.cancel(&first.token).unwrap();
    assert!(service.0.lock().unwrap().is_some());
    service.0.lock().unwrap().as_mut().unwrap().created = Instant::now().checked_sub(TTL).unwrap();
    assert!(
        service
            .apply(2, &second.token, |_| panic!("expired"))
            .err()
            .unwrap()
            .contains("过期")
    );
    assert!(service.0.lock().unwrap().is_none());
    let third = service
        .prepare(
            2,
            crate::execution::Collected {
                capture,
                fixtures: vec![],
                source_name: "手动".into(),
                revision: "5".into(),
            },
        )
        .unwrap();
    service.cancel(&third.token).unwrap();
    assert!(
        service
            .apply(2, &third.token, |_| panic!("cancelled"))
            .is_err()
    );
}
