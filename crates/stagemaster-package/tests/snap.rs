#[path = "support/raw_archive.rs"]
mod raw_archive;
use raw_archive::{raw_archive, reseal};
use stagemaster_package::{
    Archive, Builder, Error, Kind, Mapping, Output, Program, Source, StepLabel, decode_program,
    encode_program,
};
use stagemaster_playback::{Curve, EffectChannel, Plan, Player, Step};

fn program(snap: Vec<u16>) -> Program {
    Program {
        output: Output {
            universe: 1,
            mappings: vec![
                Mapping {
                    coarse: 512,
                    fine: Some(1),
                },
                Mapping {
                    coarse: 2,
                    fine: None,
                },
            ],
        },
        labels: (1..=2)
            .map(|n| StepLabel {
                id: [n; 16],
                name: format!("第{n}步"),
                number: n.to_string(),
            })
            .collect(),
        plan: Plan::with_snap_attributes(
            vec![257, 1000],
            vec![
                Step {
                    target: vec![60000, 5000],
                    delay_ms: 50,
                    fade_ms: 100,
                    wait_ms: Some(50),
                },
                Step {
                    target: vec![0, 1000],
                    delay_ms: 0,
                    fade_ms: 200,
                    wait_ms: Some(100),
                },
            ],
            true,
            vec![vec![], vec![]],
            snap,
        )
        .unwrap(),
    }
}
fn source() -> Source {
    Source {
        project_id: [1; 16],
        revision_id: [2; 16],
        snapshot_digest: [3; 32],
        project_name: "直接切换".into(),
    }
}
#[test]
fn roundtrip_preserves_discrete_and_continuous_channels_in_the_same_output() {
    let input = program(vec![0]);
    let bytes = encode_program(&input).unwrap();
    assert_eq!(bytes[0], 0x86);
    let (output, usage) = decode_program(&bytes, 0).unwrap();
    assert_eq!(output, input);
    assert_eq!(usage.snap_attributes, 1);
    let (_, old_usage) = decode_program(&encode_program(&program(vec![])).unwrap(), 0).unwrap();
    assert_eq!(usage.resident_bytes, old_usage.resident_bytes + 2 + 32);
    let mut a = Player::new(input.plan, 0);
    let mut b = Player::new(output.plan, 0);
    a.execute(0, 0).unwrap();
    b.execute(0, 0).unwrap();
    for t in 0..2001 {
        a.advance(t).unwrap();
        b.advance(t).unwrap();
        assert_eq!(a.values(), b.values());
        assert!([257, 60000, 0].contains(&a.values()[0]));
        let mut x = [0; 512];
        let mut y = [0; 512];
        input.output.render(a.values(), &mut x).unwrap();
        output.output.render(b.values(), &mut y).unwrap();
        assert_eq!(x, y);
    }
    assert!(decode_program(&bytes, 65536 - usage.loader_peak_bytes + 1).is_err());
}
#[test]
fn actual_legacy_file_rebuilds_byte_for_byte() {
    let bytes = include_bytes!("fixtures/legacy-v1.smpkg");
    let archive = Archive::open(bytes.as_slice()).unwrap();
    assert_eq!(archive.semantics(), 1);
    let mut b = Builder::new(archive.source().clone());
    for (i, e) in archive.entries().iter().enumerate() {
        let p = archive.load(bytes.as_slice(), i).unwrap();
        assert!(p.plan.snap_attributes().is_empty());
        assert_eq!(e.usage.snap_attributes, 0);
        b.add(e.kind, e.id, &e.name, &p).unwrap();
    }
    assert_eq!(b.finish().unwrap().0, bytes);
}
#[test]
fn mixed_archive_declares_new_semantics_and_refuses_header_downgrades() {
    let mut b = Builder::new(source());
    b.add(Kind::Sequence, [1; 16], "旧节目", &program(vec![]))
        .unwrap();
    b.add(Kind::Sequence, [2; 16], "离散节目", &program(vec![0]))
        .unwrap();
    let (bytes, a) = b.finish().unwrap();
    assert_eq!(a.semantics(), 2);
    assert_eq!(&bytes[12..14], &[2, 0]);
    assert_eq!(
        a.load(bytes.as_slice(), 0).unwrap().plan.snap_attributes(),
        []
    );
    assert_eq!(
        a.load(bytes.as_slice(), 1).unwrap().plan.snap_attributes(),
        [0]
    );
    let mut wrong = bytes.clone();
    wrong[12] = 1;
    reseal(&mut wrong);
    assert_eq!(Archive::open(wrong.as_slice()).unwrap_err(), Error::Version);
    let start = wrong
        .windows(stagemaster_package::SNAP_COMPILER.len())
        .position(|s| s == stagemaster_package::SNAP_COMPILER.as_bytes())
        .unwrap();
    wrong[start + stagemaster_package::SNAP_COMPILER.len() - 1] = b'1';
    reseal(&mut wrong);
    assert_eq!(Archive::open(wrong.as_slice()).unwrap_err(), Error::Version);
    for version in [0, 3, 255] {
        let mut wrong = bytes.clone();
        wrong[12] = version;
        reseal(&mut wrong);
        assert_eq!(Archive::open(wrong.as_slice()).unwrap_err(), Error::Version);
    }
}
// Attach a independently authored index array to a valid legacy block. Hashes are recomputed,
// so failures must come from semantic validation in the scan path, not integrity checks.
fn with_indices(mut block: Vec<u8>, tail: &[u8]) -> Vec<u8> {
    assert_eq!(block[0], 0x85);
    block[0] = 0x86;
    block.extend_from_slice(tail);
    block
}
#[test]
fn untrusted_indices_and_effect_conflicts_fail_during_archive_scan() {
    let legacy = encode_program(&program(vec![])).unwrap();
    for tail in [
        vec![0x80],
        vec![0x81, 2],
        vec![0x82, 0, 0],
        vec![0x82, 1, 0],
        vec![0x81, 0x19, 0xff, 0xff],
        vec![0x9f, 0, 0xff],
        vec![0x81, 0xf5],
        vec![0x9b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
    ] {
        let block = with_indices(legacy.clone(), &tail);
        assert!(decode_program(&block, 0).is_err(), "{tail:?}");
        assert!(
            Archive::open(raw_archive(&block, 2).as_slice()).is_err(),
            "{tail:?}"
        );
    }
    let mut p = program(vec![]);
    let effect = EffectChannel {
        index: 0,
        low: 0,
        high: 65535,
        period_ms: 1000,
        phase: 0,
        duty_percent: 50,
        curve: Curve::Smooth,
    };
    p.plan = Plan::with_effects(
        p.plan.defaults().to_vec(),
        p.plan.steps().to_vec(),
        true,
        vec![vec![], vec![effect]],
    )
    .unwrap();
    let block = with_indices(encode_program(&p).unwrap(), &[0x81, 0]);
    assert!(decode_program(&block, 0).is_err());
    assert!(Archive::open(raw_archive(&block, 2).as_slice()).is_err());
    let valid = with_indices(legacy, &[0x82, 0, 1]);
    assert!(Archive::open(raw_archive(&valid, 2).as_slice()).is_ok());
    assert_eq!(
        Archive::open(raw_archive(&valid, 1).as_slice()).unwrap_err(),
        Error::Version
    );
}
#[test]
fn every_new_block_truncation_and_unknown_shape_is_rejected() {
    let block = encode_program(&program(vec![0, 1])).unwrap();
    for i in 0..block.len() {
        assert!(decode_program(&block[..i], 0).is_err(), "{i}");
    }
    for head in [0x84, 0x87, 0x9f] {
        let mut invalid = block.clone();
        invalid[0] = head;
        assert!(decode_program(&invalid, 0).is_err());
    }
    let mut trailing = block;
    trailing.push(0);
    assert!(decode_program(&trailing, 0).is_err());
}

#[test]
fn highest_discrete_index_and_highest_effect_conflict_are_checked() {
    let mut p = program(vec![]);
    p.output.mappings = (1..=512)
        .map(|coarse| Mapping { coarse, fine: None })
        .collect();
    let steps = vec![Step {
        target: vec![0; 512],
        delay_ms: 0,
        fade_ms: 10,
        wait_ms: None,
    }];
    p.labels.truncate(1);
    p.plan =
        Plan::with_snap_attributes(vec![0; 512], steps.clone(), false, vec![vec![]], vec![511])
            .unwrap();
    let bytes = encode_program(&p).unwrap();
    assert_eq!(
        decode_program(&bytes, 0).unwrap().0.plan.snap_attributes(),
        [511]
    );
    let effect = EffectChannel {
        index: 511,
        low: 0,
        high: 65535,
        period_ms: 1000,
        phase: 0,
        curve: Curve::Smooth,
        duty_percent: 50,
    };
    p.plan = Plan::with_effects(vec![0; 512], steps, false, vec![vec![effect]]).unwrap();
    let invalid = with_indices(encode_program(&p).unwrap(), &[0x81, 0x19, 0x01, 0xff]);
    assert!(Archive::open(raw_archive(&invalid, 2).as_slice()).is_err());
}
