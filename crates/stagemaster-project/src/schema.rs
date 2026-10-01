//! Shared embedded schema resolution for documents and portable recipe fragments.
use serde_json::{Value, json};
use std::sync::OnceLock;

pub(super) fn project() -> &'static jsonschema::Validator {
    static SCHEMA: OnceLock<jsonschema::Validator> = OnceLock::new();
    SCHEMA.get_or_init(|| jsonschema::validator_for(&source()).expect("project schema"))
}
pub(super) fn effect_template() -> &'static jsonschema::Validator {
    static SCHEMA: OnceLock<jsonschema::Validator> = OnceLock::new();
    SCHEMA.get_or_init(|| {
        let root = json!({"$defs":source()["$defs"],"$ref":"#/$defs/EffectTemplate"});
        jsonschema::validator_for(&root).expect("effect template schema")
    })
}
fn source() -> Value {
    let rewrite = |s: &str| {
        s.replace(
            "urn:stagemaster:schema:common:0.1.0-draft.1#/$defs/",
            "#/$defs/Common/$defs/",
        )
    };
    let mut root: Value = serde_json::from_str(&rewrite(include_str!(
        "../../../docs/project-format/schemas/project.schema.json"
    )))
    .expect("project schema");
    let mut common: Value = serde_json::from_str(&rewrite(include_str!(
        "../../../docs/project-format/schemas/common.schema.json"
    )))
    .expect("common schema");
    common.as_object_mut().expect("schema object").remove("$id");
    root["$defs"]["Common"] = common;
    root
}
