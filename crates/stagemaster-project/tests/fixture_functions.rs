use stagemaster_project::{
    FunctionDefinition as F, FunctionMode as M, FunctionSelection as S, FunctionTable,
    ProfileDefault,
};

fn functions() -> Vec<F> {
    vec![
        F {
            key: "closed".into(),
            name: "关闭".into(),
            mode: M::Slot,
            dmx_from: 0,
            dmx_to: 7,
            dmx_default: 0,
            appearance: None,
        },
        F {
            key: "open".into(),
            name: "常开".into(),
            mode: M::Slot,
            dmx_from: 8,
            dmx_to: 15,
            dmx_default: 10,
            appearance: None,
        },
        F {
            key: "strobe".into(),
            name: "频闪".into(),
            mode: M::Range,
            dmx_from: 32,
            dmx_to: 200,
            dmx_default: 50,
            appearance: None,
        },
    ]
}
#[test]
fn decoding_reconstructs_every_valid_native_value_exactly() {
    for fine in [false, true] {
        let mut fs = functions();
        if fine {
            fs[2].dmx_to = u16::MAX;
        }
        let table = FunctionTable::new(&fs, fine).unwrap();
        for native in fs[2].dmx_from..=fs[2].dmx_to {
            let raw = if fine { native } else { native * 257 };
            assert_eq!(table.encode(&table.decode(raw).unwrap()).unwrap(), raw);
        }
        let scale = if fine { 1 } else { 257 };
        assert_eq!(table.decode(10 * scale).unwrap().function_key, "open");
        assert!(table.decode(9 * scale).is_err());
        assert!(table.decode(25 * scale).is_err());
        if !fine {
            assert!(table.decode(33).is_err());
        }
    }
}
#[test]
fn slots_are_exact_and_ranges_never_escape_to_adjacent_or_reserved_functions() {
    for fine in [false, true] {
        let fs = functions();
        let table = FunctionTable::new(&fs, fine).unwrap();
        let scale = if fine { 1 } else { 257 };
        assert_eq!(
            table
                .encode(&S {
                    function_key: "open".into(),
                    position: 0
                })
                .unwrap(),
            10 * scale
        );
        assert!(
            table
                .encode(&S {
                    function_key: "open".into(),
                    position: 1
                })
                .is_err()
        );
        assert!(table.initial_selection("missing").is_err());
        assert!(
            table
                .encode(&S {
                    function_key: "missing".into(),
                    position: 0
                })
                .is_err()
        );
        let mut last = 0;
        for position in 0..=u16::MAX {
            let value = table
                .encode(&S {
                    function_key: "strobe".into(),
                    position,
                })
                .unwrap();
            assert!((32 * scale..=200 * scale).contains(&value));
            assert!(value >= last);
            last = value;
            if !fine {
                assert_eq!(value / 257, u16::from(value.to_be_bytes()[0]));
            }
        }
        assert_eq!(last, 200 * scale);
        assert_eq!(
            table
                .encode(&table.initial_selection("strobe").unwrap())
                .unwrap(),
            50 * scale
        );
    }
}
#[test]
fn representatives_roundtrip_for_every_native_byte_and_full_word() {
    for (fine, to) in [(false, 255), (true, 65535)] {
        for value in 0..=to {
            let fs = [F {
                key: "speed".into(),
                name: "速度".into(),
                mode: M::Range,
                dmx_from: 0,
                dmx_to: to,
                dmx_default: value,
                appearance: None,
            }];
            let table = FunctionTable::new(&fs, fine).unwrap();
            assert_eq!(
                table
                    .encode(&table.initial_selection("speed").unwrap())
                    .unwrap(),
                if fine { value } else { value * 257 }
            );
        }
    }
}
#[test]
fn ranges_keys_defaults_and_precision_are_validated_without_reordering() {
    let fs = functions();
    let mut reversed = fs.clone();
    reversed.reverse();
    assert!(FunctionTable::new(&reversed, false).is_ok());
    assert_eq!(reversed[0], fs[2]);
    assert!(FunctionTable::new(&[], false).is_err());
    assert!(FunctionTable::new(&vec![fs[0].clone(); 65], false).is_err());
    for invalid in ["", "Open", "red blue", "a--b", "a.", "_red", "中", "a/1"] {
        let mut changed = fs.clone();
        changed[0].key = invalid.into();
        assert!(FunctionTable::new(&changed, false).is_err(), "{invalid}");
    }
    for (from, to, default) in [
        (15, 31, 20),
        (80, 90, 91),
        (210, 205, 206),
        (201, 201, 201),
        (201, 300, 250),
    ] {
        let mut changed = fs.clone();
        changed[2].dmx_from = from;
        changed[2].dmx_to = to;
        changed[2].dmx_default = default;
        assert!(FunctionTable::new(&changed, false).is_err());
    }
    let mut changed = fs.clone();
    changed[2].key = changed[0].key.clone();
    assert!(FunctionTable::new(&changed, false).is_err());
    let mut changed = fs.clone();
    changed[0].name = " \n ".into();
    assert!(FunctionTable::new(&changed, false).is_err());
    let mut changed = fs;
    changed[2].dmx_to = 300;
    assert!(FunctionTable::new(&changed, true).is_ok());
}
#[test]
fn function_and_numeric_default_dtos_are_distinct_and_strict() {
    assert_eq!(
        serde_json::from_str::<ProfileDefault>("123").unwrap(),
        ProfileDefault::Normalized(123)
    );
    let value = ProfileDefault::Function(S {
        function_key: "open".into(),
        position: 0,
    });
    let encoded = serde_json::to_value(&value).unwrap();
    assert_eq!(
        encoded,
        serde_json::json!({"functionKey":"open","position":0})
    );
    assert_eq!(
        serde_json::from_value::<ProfileDefault>(encoded).unwrap(),
        value
    );
    for invalid in [
        serde_json::json!(-1),
        serde_json::json!(65536),
        serde_json::json!({"functionKey":"open"}),
        serde_json::json!({"functionKey":"open","position":0,"dmx":255}),
    ] {
        assert!(serde_json::from_value::<ProfileDefault>(invalid).is_err());
    }
}
