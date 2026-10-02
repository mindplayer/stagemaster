// Internal fault injection, deliberately unavailable to host callers.
use super::*;
use stagemaster_project::{Document, PackageSelection};

#[test]
fn exhausted_source_or_frame_serial_retracts_the_entire_observation() {
    let mut raw: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    raw["entryPoints"] = serde_json::json!([]);
    let doc = Document::decode(&serde_json::to_vec(&raw).unwrap()).unwrap();
    let spec = SourceSpec {
        id: [1; 16],
        priority: 0,
        playback: Some(PackageSelection::Scene {
            id: doc.view().scenes[0].id.clone(),
        }),
    };
    for source_fault in [false, true] {
        let mut s = Session::prepare(&doc, [1; 16], std::slice::from_ref(&spec), 0).unwrap();
        let key = s.key([1; 16]).unwrap();
        s.control(key, Command::Execute(0), 0).unwrap();
        assert!(s.frame().is_some());
        if source_fault {
            s.sources[0].serial = u64::MAX - 1;
        } else {
            s.sequence = u64::MAX;
        }
        assert!(s.tick(25).is_err());
        assert!(s.frame().is_none());
        assert!(s.values().is_none());
        assert!(s.winner(0).is_none());
        assert!(s.fault().is_some());
        assert!(s.control(key, Command::Stop, 26).is_err());
        assert!(s.frame().is_none());
    }
}
