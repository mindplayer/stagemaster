use serde_json::{Value, json};

pub fn root_with_scenes(copies: usize) -> Value {
    let mut root: Value = serde_json::from_slice(include_bytes!(
        "../../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    root["entryPoints"] = json!([]);
    root["project"]["name"] = json!("容量验收");
    root["project"]["description"] = json!("");
    root["project"]["parentRevisionIds"] = json!([]);
    let scene = root["lighting"]["scenes"][0].clone();
    for index in 0..copies {
        let mut copy = scene.clone();
        copy["id"] = json!(format!("90000000-0000-4000-8000-{index:012x}"));
        root["lighting"]["scenes"]
            .as_array_mut()
            .unwrap()
            .push(copy);
    }
    root
}

pub fn root_of_size(bytes: usize) -> Value {
    let mut root = root_with_scenes(9300);
    for scene in root["lighting"]["scenes"].as_array_mut().unwrap() {
        scene["name"] = json!("X");
    }
    let mut remaining = bytes
        .checked_sub(serde_json::to_vec(&root).unwrap().len())
        .unwrap();
    for scene in root["lighting"]["scenes"].as_array_mut().unwrap() {
        let added = remaining.min(255);
        scene["name"] = json!(format!("X{}", "x".repeat(added)));
        remaining -= added;
        if remaining == 0 {
            break;
        }
    }
    assert_eq!(remaining, 0, "fixture must fit the schema's name limits");
    assert_eq!(serde_json::to_vec(&root).unwrap().len(), bytes);
    root
}
