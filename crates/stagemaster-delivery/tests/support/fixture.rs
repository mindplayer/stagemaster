use stagemaster_project::{Document, PackageSelection};
pub fn bytes(level: u16) -> Vec<u8> {
    let mut json: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    json["entryPoints"] = serde_json::json!([]);
    json["lighting"]["profiles"][0]["attributes"][0]["default"]["value"] = level.into();
    json["lighting"]["sequences"][0]["repeat"] = "loop".into();
    let document = Document::decode(&serde_json::to_vec(&json).unwrap()).unwrap();
    let selection = document
        .view()
        .sequences
        .into_iter()
        .map(|s| PackageSelection::Sequence { id: s.id })
        .collect::<Vec<_>>();
    document.build_package(&selection).unwrap().bytes
}
