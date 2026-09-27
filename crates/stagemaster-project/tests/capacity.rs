#[path = "support/capacity.rs"]
mod support;
use serde_json::{Value, json};
use stagemaster_project::{Document, EditCommand, MAX_BYTES};

fn rename(description: &str) -> EditCommand {
    EditCommand::SetInfo {
        name: "容量验收".into(),
        description: description.into(),
    }
}

#[test]
fn small_documents_keep_readable_encoding_and_all_escaped_content() {
    let mut doc = Document::new("容量验收").unwrap();
    doc.edit(rename("中文 🦀\n\"引号\"\\路径\t\u{0001}"))
        .unwrap();
    let bytes = doc.encode().unwrap();
    let raw: Value = serde_json::from_slice(&bytes).unwrap();
    let mut pretty = serde_json::to_vec_pretty(&raw).unwrap();
    pretty.push(b'\n');
    assert_eq!(bytes, pretty);
    assert_eq!(Document::decode(&bytes).unwrap(), doc);
}

#[test]
fn large_compact_document_remains_editable_and_preserves_all_fields() {
    let mut root = support::root_with_scenes(6000);
    assert!(serde_json::to_vec_pretty(&root).unwrap().len() > MAX_BYTES);
    let input = serde_json::to_vec(&root).unwrap();
    let mut doc = Document::decode(&input).unwrap();
    doc.edit(rename("紧凑工程：中文、灯光与空间全部保留"))
        .unwrap();
    root["project"]["description"] = json!("紧凑工程：中文、灯光与空间全部保留");
    let output = doc.encode().unwrap();
    assert!(output.len() <= MAX_BYTES);
    assert_eq!(serde_json::from_slice::<Value>(&output).unwrap(), root);
    assert_eq!(Document::decode(&output).unwrap(), doc);
    assert!(doc.next_revision().encode().unwrap().len() <= MAX_BYTES);
}

#[test]
fn exact_capacity_reserves_parent_revision_and_survives_repeated_saves() {
    let root = support::root_of_size(MAX_BYTES - 38);
    let doc = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
    let mut next = doc.next_revision();
    for _ in 0..3 {
        let bytes = next.encode().unwrap();
        assert_eq!(bytes.len(), MAX_BYTES);
        assert_eq!(bytes.last(), Some(&b'}')); // no extra newline at the exact limit
        let reopened = Document::decode(&bytes).unwrap();
        assert_eq!(reopened, next);
        assert!(reopened.same_content(&doc));
        let raw: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(
            raw["project"]["parentRevisionIds"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        next = reopened.next_revision();
    }
    let mut over = root;
    over["project"]["description"] = json!("x");
    let input = serde_json::to_vec(&over).unwrap();
    assert!(input.len() < MAX_BYTES);
    assert!(Document::decode(&input).unwrap_err().contains("修订信息"));
}

#[test]
fn encoded_byte_limit_counts_utf8_and_escaping_and_rolls_back_whole_batch() {
    let root = support::root_of_size(MAX_BYTES - 38 - 6);
    let mut doc = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
    // Both values encode to exactly six bytes inside the existing quotes.
    for description in ["中文", "\u{0001}"] {
        doc.edit(rename(description)).unwrap();
        assert_eq!(doc.next_revision().encode().unwrap().len(), MAX_BYTES);
    }
    let before = doc.clone();
    let scene_id = root["lighting"]["scenes"][0]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let error = doc
        .edit(EditCommand::Batch {
            commands: vec![
                EditCommand::RenameScene {
                    id: scene_id,
                    name: "y".repeat(256),
                },
                rename("中文x"),
            ],
        })
        .unwrap_err();
    assert!(error.contains("8 MiB"));
    assert_eq!(doc, before);
    doc.edit(rename("")).unwrap();
    assert!(doc.encode().unwrap().len() < MAX_BYTES);
    let mut oversized = vec![b' '; MAX_BYTES + 1];
    oversized[0] = b'{';
    assert!(Document::decode(&oversized).unwrap_err().contains("8 MiB"));
}
