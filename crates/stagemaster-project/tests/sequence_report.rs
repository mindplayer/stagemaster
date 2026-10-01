use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use stagemaster_project::{Document, PatchReport, SequenceReport};

fn fixture() -> Document {
    let mut root: Value = serde_json::from_slice(include_bytes!(
        "../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    root["entryPoints"] = json!([]);
    root["requires"]
        .as_array_mut()
        .unwrap()
        .push(json!({"key":"lighting.sequence-script","version":1}));
    root["project"]["name"] = json!("=工程,\"甲\"\n第二行");
    let seq = &mut root["lighting"]["sequences"][0];
    seq["name"] = json!("@列表");
    seq["steps"][0]["number"] = json!("10");
    seq["steps"][0]["advance"] = json!({"kind":"manual"});
    seq["steps"][0]["delay"]["ticks"] = json!("1");
    seq["steps"][0]["script"] =
        json!({"section":"第一幕","trigger":"=开场,\"开始\"\n举手","notes":"等待掌声\n再推进"});
    seq["steps"][1]["number"] = json!("2.5");
    Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap()
}
fn rows(report: &SequenceReport) -> Vec<csv::StringRecord> {
    csv::Reader::from_reader(report.bytes())
        .records()
        .collect::<Result<_, _>>()
        .unwrap()
}
#[test]
fn authored_order_script_and_manual_wait_survive_exact_snapshot_export() {
    let doc = fixture();
    let before = doc.encode().unwrap();
    let view = doc.view();
    let sequence = &view.sequences[0];
    let report = doc.sequence_report(&sequence.id).unwrap();
    assert_eq!(report.sequence_id(), sequence.id);
    assert_eq!(report.sequence_name(), "@列表");
    assert_eq!(report.step_count(), 2);
    let rows = rows(&report);
    assert_eq!(
        (&rows[0][9], &rows[0][10], &rows[1][9], &rows[1][10]),
        ("1", "10", "2", "2.5")
    );
    assert_eq!(
        (&rows[0][13], &rows[0][14], &rows[0][15]),
        ("第一幕", "'=开场,\"开始\"\n举手", "等待掌声\n再推进")
    );
    assert_eq!(
        (&rows[0][18], &rows[0][19], &rows[0][20], &rows[0][21]),
        ("0.001", "1.500", "手动推进", "")
    );
    assert_eq!(
        (&rows[1][20], &rows[1][21], &rows[1][13]),
        ("自动推进", "0.000", "")
    );
    assert_eq!((&rows[0][7], &rows[0][8]), ("继承前序", "执行一遍"));
    assert_eq!(&rows[0][1], "'=工程,\"甲\"\n第二行");
    assert_eq!(&rows[0][5], "'@列表");
    assert_eq!(&rows[0][4], format!("{:x}", Sha256::digest(&before)));
    assert_eq!(&rows[0][12], sequence.steps[0].id);
    assert_eq!(&rows[0][17], sequence.steps[0].scene_id);
    assert_eq!(&rows[0][16], "蓝色入场");
    assert!(report.bytes().starts_with(b"\xef\xbb\xbf"));
    assert!(report.bytes().ends_with(b"\r\n"));
    assert!(SequenceReport::recognizes(report.bytes()));
    assert_eq!(
        report.bytes(),
        doc.sequence_report(&sequence.id).unwrap().bytes()
    );
    assert_eq!(before, doc.encode().unwrap());
    assert!(doc.sequence_report("不存在").is_err());
}
#[test]
fn repeat_tracking_and_long_times_are_explicit_and_types_cannot_be_confused() {
    let mut root: Value = serde_json::from_slice(&fixture().encode().unwrap()).unwrap();
    let seq = &mut root["lighting"]["sequences"][0];
    seq["repeat"] = json!("loop");
    seq["tracking"] = json!("isolated");
    seq["steps"][1]["advance"]["wait"]["ticks"] = json!("86400000");
    let doc = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
    let report = doc.sequence_report(&doc.view().sequences[0].id).unwrap();
    let rows = rows(&report);
    assert_eq!(
        (&rows[1][7], &rows[1][8], &rows[1][21]),
        ("独立场景", "循环执行", "86400.000")
    );
    assert!(!PatchReport::recognizes(report.bytes()));
    assert!(!SequenceReport::recognizes(
        doc.patch_report().unwrap().bytes()
    ));
    assert!(!SequenceReport::recognizes(&report.bytes()[..100]));
    let bad = String::from_utf8(report.bytes().to_vec())
        .unwrap()
        .replace("StageMaster 节目单/1", "StageMaster 节目单/2");
    assert!(!SequenceReport::recognizes(bad.as_bytes()));
}
