#[allow(dead_code)]
#[path = "support/fixture_function.rs"]
mod support;
use stagemaster_project::{FunctionSelection, ProfileDefault};

#[test]
fn prepared_manual_translation_keeps_function_precision_types_and_release_distinct() {
    for fine in [false, true] {
        let (doc, fixture, scenes) = support::setup(fine);
        let compiled = doc.compile_scene(&scenes[0]).unwrap();
        let output = &compiled.output;
        let red = ProfileDefault::Function(FunctionSelection {
            function_key: "red".into(),
            position: 0,
        });
        let (index, value) = output
            .manual_value(&fixture, "color-wheel", Some(&red))
            .unwrap();
        assert_eq!(value, Some(if fine { 20 } else { 20 * 257 }));
        assert_eq!(
            output.manual_value(&fixture, "color-wheel", None).unwrap(),
            (index, None)
        );
        assert!(
            output
                .manual_value(
                    &fixture,
                    "color-wheel",
                    Some(&ProfileDefault::Normalized(20))
                )
                .is_err()
        );
        assert!(output.manual_value(&fixture, "dimmer", Some(&red)).is_err());
        assert!(output.manual_value("missing", "dimmer", None).is_err());
        assert!(output.manual_value(&fixture, "missing", None).is_err());
        let invalid = ProfileDefault::Function(FunctionSelection {
            function_key: "red".into(),
            position: 1,
        });
        assert!(
            output
                .manual_value(&fixture, "color-wheel", Some(&invalid))
                .is_err()
        );
        let (_, zero) = output
            .manual_value(&fixture, "dimmer", Some(&ProfileDefault::Normalized(0)))
            .unwrap();
        assert_eq!(zero, Some(0));
    }
}
