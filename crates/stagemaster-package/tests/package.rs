use sha2::{Digest, Sha256};
use stagemaster_package::{
    Archive, Builder, Error, Kind, Mapping, Output, Program, ReadAt, Source, StepLabel,
    decode_program, encode_program,
};
use stagemaster_playback::{Curve, EffectChannel, Keyframe, Plan, Player, Step, Transition};
use std::cell::RefCell;

fn program() -> Program {
    let channels = vec![EffectChannel {
        index: 0,
        low: 1,
        high: 65000,
        period_ms: 1600,
        phase: 731,
        curve: Curve::Keyframes(vec![
            Keyframe {
                phase: 0,
                value: 0,
                transition: Transition::Linear,
            },
            Keyframe {
                phase: 32000,
                value: 65535,
                transition: Transition::Smooth,
            },
            Keyframe {
                phase: 48000,
                value: 0,
                transition: Transition::Hold,
            },
        ]),
        duty_percent: 20,
    }];
    Program {
        output: Output {
            universe: 23,
            mappings: vec![
                Mapping {
                    coarse: 512,
                    fine: Some(1),
                },
                Mapping {
                    coarse: 22,
                    fine: None,
                },
            ],
        },
        labels: vec![
            StepLabel {
                id: [2; 16],
                name: "入场".into(),
                number: "1.5".into(),
            },
            StepLabel {
                id: [3; 16],
                name: "退出".into(),
                number: "2".into(),
            },
        ],
        plan: Plan::with_effects(
            vec![1000, 32767],
            vec![
                Step {
                    target: vec![60000, 50000],
                    delay_ms: 20,
                    fade_ms: 180,
                    wait_ms: Some(2000),
                },
                Step {
                    target: vec![1234, 65535],
                    delay_ms: 0,
                    fade_ms: 100,
                    wait_ms: None,
                },
            ],
            true,
            vec![channels, vec![]],
        )
        .unwrap(),
    }
}
fn source() -> Source {
    Source {
        project_id: [1; 16],
        revision_id: [2; 16],
        snapshot_digest: [3; 32],
        project_name: "演出".into(),
    }
}
fn package() -> Vec<u8> {
    let mut builder = Builder::new(source());
    builder
        .add(Kind::Sequence, [8; 16], "整场", &program())
        .unwrap();
    builder
        .add(Kind::Sequence, [7; 16], "开场", &program())
        .unwrap();
    builder.finish().unwrap().0
}
#[test]
fn round_trip_preserves_all_semantics_and_split_output() {
    let input = program();
    let bytes = encode_program(&input).unwrap();
    let (output, usage) = decode_program(&bytes, 1000).unwrap();
    assert_eq!(input, output);
    assert_eq!(usage.attributes, 2);
    assert_eq!(usage.keyframes, 3);
    assert_eq!(usage.value_bytes, 20);
    assert!(usage.loader_peak_bytes > usage.resident_bytes + 1000);
    let mut original = Player::new(input.plan, 0);
    let mut decoded = Player::new(output.plan, 0);
    original.execute(0, 0).unwrap();
    decoded.execute(0, 0).unwrap();
    for t in 0..4000 {
        original.advance(t).unwrap();
        decoded.advance(t).unwrap();
        assert_eq!(original.values(), decoded.values());
        let mut a = [0; 512];
        let mut b = [0; 512];
        input.output.render(original.values(), &mut a).unwrap();
        output.output.render(decoded.values(), &mut b).unwrap();
        assert_eq!(a, b);
    }
    let mut slots = [7; 512];
    assert!(input.output.render(&[], &mut slots).is_err());
    assert_eq!(slots, [7; 512]);
    input.output.render(&[0x1234, 0x56ff], &mut slots).unwrap();
    assert_eq!((slots[511], slots[0], slots[21]), (0x12, 0x34, 0x56));
}
#[test]
fn builder_is_deterministic_canonical_and_duplicate_safe() {
    let bytes = package();
    let mut b = Builder::new(source());
    b.add(Kind::Sequence, [7; 16], "开场", &program()).unwrap();
    assert!(b.add(Kind::Sequence, [7; 16], "重复", &program()).is_err());
    b.add(Kind::Sequence, [8; 16], "整场", &program()).unwrap();
    assert_eq!(bytes, b.finish().unwrap().0);
    let archive = Archive::open(bytes.as_slice()).unwrap();
    assert_eq!(archive.entries()[0].id, [7; 16]);
    assert_eq!(archive.source(), &source());
    assert_eq!(archive.universe(), 23);
    assert!(archive.load(bytes.as_slice(), 2).is_err());
    assert!(Builder::new(source()).finish().is_err());
}
#[test]
fn every_truncation_and_bit_corruption_is_rejected() {
    let bytes = package();
    for n in 0..bytes.len() {
        assert!(Archive::open(&bytes[..n]).is_err(), "truncated at {n}");
    }
    for n in 0..bytes.len() {
        let mut modified = bytes.clone();
        modified[n] ^= 1;
        assert!(
            Archive::open(modified.as_slice()).is_err(),
            "corrupted at {n}"
        );
    }
    let mut extra = bytes;
    extra.push(0);
    assert!(Archive::open(extra.as_slice()).is_err());
}
#[test]
fn single_load_reads_only_the_requested_block_and_rechecks_storage() {
    struct Reader {
        bytes: Vec<u8>,
        reads: RefCell<Vec<(usize, usize)>>,
    }
    impl ReadAt for Reader {
        fn len(&self) -> usize {
            self.bytes.len()
        }
        fn read_exact(&self, offset: usize, bytes: &mut [u8]) -> Result<(), Error> {
            self.reads.borrow_mut().push((offset, bytes.len()));
            self.bytes.as_slice().read_exact(offset, bytes)
        }
    }
    let mut reader = Reader {
        bytes: package(),
        reads: RefCell::new(vec![]),
    };
    let archive = Archive::open(&reader).unwrap();
    reader.reads.borrow_mut().clear();
    assert_eq!(archive.load(&reader, 1).unwrap(), program());
    let reads = reader.reads.borrow();
    assert_eq!(reads.len(), 1);
    let offset = reads[0].0;
    assert_eq!(reads[0].1, archive.entries()[1].usage.encoded_bytes);
    drop(reads);
    reader.bytes[offset] ^= 1;
    assert_eq!(archive.load(&reader, 1).unwrap_err(), Error::Integrity);
}
fn reseal(bytes: &mut [u8]) {
    let mut h = Sha256::new();
    h.update(&bytes[..32]);
    h.update(&bytes[64..]);
    bytes[32..64].copy_from_slice(&h.finalize());
}
#[test]
fn valid_hash_does_not_bypass_versions_reserved_fields_or_directory_checks() {
    for index in [8, 10, 12, 14, 24, 26] {
        let mut bytes = package();
        bytes[index] = 99;
        reseal(&mut bytes);
        assert!(Archive::open(bytes.as_slice()).is_err());
    }
    let mut bytes = package();
    // Replace definite catalogue array with an indefinite array, with a valid package digest.
    bytes[64] = 0x9f;
    reseal(&mut bytes);
    assert!(Archive::open(bytes.as_slice()).is_err());
    let mut bytes = package();
    let compiler = b"stagemaster-lighting-1";
    let start = bytes
        .windows(compiler.len())
        .position(|w| w == compiler)
        .unwrap();
    bytes[start + compiler.len() - 1] = b'2';
    reseal(&mut bytes);
    assert_eq!(Archive::open(bytes.as_slice()).unwrap_err(), Error::Version);
}
#[test]
fn blocks_reject_aliasing_unknown_fields_and_memory_excess_before_plan_load() {
    let input = program();
    let mut duplicate = input.clone();
    duplicate.output.mappings[1].coarse = 1;
    assert!(encode_program(&duplicate).is_err());
    let mut duplicate = input.clone();
    duplicate.labels[1].id = duplicate.labels[0].id;
    assert!(encode_program(&duplicate).is_err());
    let bytes = encode_program(&input).unwrap();
    for n in 0..bytes.len() {
        assert!(decode_program(&bytes[..n], 0).is_err());
    }
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(decode_program(&trailing, 0).is_err());
    let mut indefinite = bytes.clone();
    indefinite[0] = 0x9f;
    assert!(decode_program(&indefinite, 0).is_err());
    assert!(decode_program(&bytes, usize::MAX).is_err());
    assert!(decode_program(&bytes, 65536).is_err());
    let mut scene = Builder::new(source());
    scene.add(Kind::Scene, [8; 16], "非单场景", &input).unwrap();
    assert!(scene.finish().is_err());
}
#[test]
fn reference_metadata_sizes_do_not_underestimate_this_target() {
    use std::mem::size_of;
    assert!(size_of::<Step>() <= 56);
    assert!(size_of::<StepLabel>() <= 64);
    assert!(size_of::<EffectChannel>() <= 80);
    assert!(size_of::<Keyframe>() <= 8);
    assert!(size_of::<Mapping>() <= 8);
    assert!(size_of::<stagemaster_package::Entry>() <= 224);
}

// Build bytes independently of the product Builder, so valid hashes cannot hide parser errors.
fn raw_archive(block: &[u8]) -> Vec<u8> {
    let mut e = minicbor::Encoder::new(Vec::new());
    e.array(6)
        .unwrap()
        .str(stagemaster_package::COMPILER)
        .unwrap()
        .bytes(&[1; 16])
        .unwrap()
        .bytes(&[2; 16])
        .unwrap()
        .bytes(&[3; 32])
        .unwrap()
        .str("独立编码")
        .unwrap()
        .array(1)
        .unwrap()
        .array(6)
        .unwrap()
        .u8(1)
        .unwrap()
        .bytes(&[4; 16])
        .unwrap()
        .str("节目")
        .unwrap()
        .u32(0)
        .unwrap()
        .u32(u32::try_from(block.len()).unwrap())
        .unwrap()
        .bytes(&Sha256::digest(block))
        .unwrap();
    let catalog = e.into_writer();
    let mut bytes = vec![0; 64];
    bytes[..8].copy_from_slice(b"STMPLAY\0");
    bytes[8..16].copy_from_slice(&[1, 0, 64, 0, 1, 0, 1, 0]);
    bytes[16..20].copy_from_slice(&u32::try_from(catalog.len()).unwrap().to_le_bytes());
    let total = 64 + catalog.len() + block.len();
    bytes[20..24].copy_from_slice(&u32::try_from(total).unwrap().to_le_bytes());
    bytes[24] = 1;
    bytes.extend(catalog);
    bytes.extend_from_slice(block);
    reseal(&mut bytes);
    bytes
}
#[test]
fn semantic_validation_cannot_be_bypassed_by_rehashing_the_entire_file() {
    let block = encode_program(&program()).unwrap();
    let mut mutations = Vec::new();
    let mut d = minicbor::Decoder::new(&block);
    d.array().unwrap();
    mutations.push((d.position(), 0)); // invalid universe
    d.u16().unwrap();
    d.array().unwrap();
    d.array().unwrap();
    let coarse_position = d.position();
    // 512 uses a three-byte encoding. Replacing its last byte gives slot 513.
    mutations.push((coarse_position + 2, 1));
    d.u16().unwrap();
    d.u16().unwrap();
    d.array().unwrap();
    d.u16().unwrap();
    d.u16().unwrap();
    d.array().unwrap();
    d.u16().unwrap();
    d.u16().unwrap();
    mutations.push((d.position(), 0)); // repeat must be CBOR bool
    d.bool().unwrap();
    d.array().unwrap();
    d.array().unwrap();
    d.bytes().unwrap();
    d.str().unwrap();
    d.str().unwrap();
    d.array().unwrap();
    d.u16().unwrap();
    d.u16().unwrap();
    d.u64().unwrap();
    d.u64().unwrap();
    d.u64().unwrap();
    d.array().unwrap();
    d.array().unwrap();
    mutations.push((d.position(), 22)); // effect index out of bounds
    d.u16().unwrap();
    d.u16().unwrap();
    d.u16().unwrap();
    d.u32().unwrap();
    d.u16().unwrap();
    d.u8().unwrap();
    mutations.push((d.position(), 4)); // unknown curve
    d.u8().unwrap();
    d.array().unwrap();
    d.array().unwrap();
    d.u16().unwrap();
    d.u16().unwrap();
    mutations.push((d.position(), 7)); // unknown transition
    for (pos, value) in mutations {
        let mut invalid = block.clone();
        invalid[pos] = value;
        assert!(
            Archive::open(raw_archive(&invalid).as_slice()).is_err(),
            "accepted mutation at {pos}"
        );
    }
    assert_eq!(
        Archive::open(raw_archive(&block).as_slice())
            .unwrap()
            .entries()
            .len(),
        1
    );
    // A plausible but gigantic declared mapping count is rejected before any matching allocation.
    let huge = [
        0x85, 0x01, 0x9b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    ];
    assert!(Archive::open(raw_archive(&huge).as_slice()).is_err());
}
#[test]
fn bounded_random_input_never_panics_or_accepts_incomplete_structures() {
    let mut seed = 17_u64;
    for len in 0..2000 {
        let input: Vec<u8> = (0..len)
            .map(|_| {
                seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                seed.to_le_bytes()[3]
            })
            .collect();
        assert!(decode_program(&input, 0).is_err());
    }
}
#[test]
fn fully_valid_large_plans_are_rejected_by_measured_shape_budget() {
    let mut p = program();
    p.output.mappings = (1..=512)
        .map(|coarse| Mapping { coarse, fine: None })
        .collect();
    p.labels = (1..=32)
        .map(|i| StepLabel {
            id: [i; 16],
            name: "步骤".into(),
            number: i.to_string(),
        })
        .collect();
    p.plan = Plan::new(
        vec![0; 512],
        (0..32)
            .map(|_| Step {
                target: vec![0; 512],
                delay_ms: 0,
                fade_ms: 0,
                wait_ms: None,
            })
            .collect(),
        false,
    )
    .unwrap();
    // Mostly zero CBOR values fit the file budget, but input + plan + player cannot fit 64 KiB.
    assert!(matches!(encode_program(&p), Err(Error::Limit(_))));
}
