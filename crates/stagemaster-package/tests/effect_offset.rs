use stagemaster_package::{Mapping, Output, Program, StepLabel, encode_program};
use stagemaster_playback::{Plan, Player, Step};
fn program() -> Program {
    Program {
        output: Output {
            universe: 1,
            mappings: vec![Mapping {
                coarse: 1,
                fine: None,
            }],
        },
        labels: vec![StepLabel {
            id: [1; 16],
            name: "场景".into(),
            number: "1".into(),
        }],
        plan: Plan::new(
            vec![0],
            vec![Step {
                target: vec![100],
                delay_ms: 0,
                fade_ms: 0,
                wait_ms: None,
            }],
            false,
        )
        .unwrap(),
    }
}
#[test]
fn unsupported_offsets_cannot_be_silently_lost_in_existing_device_packages() {
    let mut p = program();
    let old = encode_program(&p).unwrap();
    p.plan = p.plan.with_entry_fade_offset(0).unwrap();
    assert_eq!(encode_program(&p).unwrap(), old);
    p.plan = p.plan.with_entry_fade_offset(237).unwrap();
    assert!(
        encode_program(&p)
            .unwrap_err()
            .to_string()
            .contains("渐变时间偏移")
    );
    p.plan = p.plan.with_entry_fade_offset(0).unwrap();
    p.plan = p.plan.with_effect_time_offset(0).unwrap();
    assert_eq!(encode_program(&p).unwrap(), old);
    p.plan = p.plan.with_effect_time_offset(333).unwrap();
    assert!(
        encode_program(&p)
            .unwrap_err()
            .to_string()
            .contains("效果时间偏移")
    );
    // Existing 512-byte metadata allowance includes both transient Program and live Player.
    assert!(std::mem::size_of::<Program>() + std::mem::size_of::<Player>() <= 512);
}
