use serde_json::{Value, json};
use stagemaster_project::Document;

#[test]
fn observed_slots_preserve_nonadjacent_fine_channels_and_actual_eight_bit_resolution() {
    let mut raw: Value = serde_json::from_slice(include_bytes!(
        "../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    raw["entryPoints"] = json!([]);
    let profile = &mut raw["lighting"]["profiles"][0];
    profile["footprint"] = 5.into();
    profile["channels"][0]["offsets"] = json!([0, 4]);
    profile["channels"][0]["encoding"] = "u16-be".into();
    let doc = Document::decode(&serde_json::to_vec(&raw).unwrap()).unwrap();
    let compiled = doc.compile_scene(&doc.view().scenes[0].id).unwrap();
    let observer = doc
        .compile_scene(&doc.view().scenes[0].id)
        .unwrap()
        .output
        .observer()
        .unwrap();
    for value in [0, 1, 255, 256, 257, 32767, 32768, 65534, 65535] {
        let values = vec![value; compiled.plan.defaults().len()];
        let encoded = compiled.output.render(&values).unwrap();
        let slots = encoded.slots.as_slice().try_into().unwrap();
        let decoded = observer.observe(encoded.universe, slots).unwrap();
        assert_eq!(decoded.slots, encoded.slots);
        assert_eq!(decoded.fixtures[0].attributes[0].value, value);
        for attribute in &decoded.fixtures[0].attributes[1..] {
            assert_eq!(attribute.value, (value >> 8) * 257);
        }
        assert!(observer.observe(encoded.universe + 1, slots).is_err());
    }
}
