use stagemaster_project::{Document, PackageSelection};
pub fn package() -> Vec<u8> {
    let mut json: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    json["entryPoints"] = serde_json::json!([]);
    Document::decode(&serde_json::to_vec(&json).unwrap())
        .unwrap()
        .build_package(&[PackageSelection::Sequence {
            id: "00000000-0000-4000-8000-000000000040".into(),
        }])
        .unwrap()
        .bytes
}
