use super::*;
use rodio::buffer::SamplesBuffer;

fn buffer() -> LoopBuffer {
    LoopBuffer::decode(
        SamplesBuffer::new(
            2.try_into().unwrap(),
            1000.try_into().unwrap(),
            (0..400_u16).map(f32::from).collect::<Vec<_>>(),
        ),
        LoopRange {
            start_ms: 500,
            end_ms: 700,
        },
    )
    .unwrap()
}
#[test]
fn repeated_stereo_frames_and_cursor_share_the_same_period() {
    let buffer = buffer();
    let source = buffer.source(550);
    let samples = source.take(4000).collect::<Vec<_>>();
    for (index, sample) in samples.iter().enumerate() {
        assert_eq!(
            sample.to_bits(),
            f32::from(u16::try_from((index + 100) % 400).unwrap()).to_bits()
        );
    }
    for (elapsed, expected) in [(0, 550), (149, 699), (150, 500), (2150, 500), (2175, 525)] {
        assert_eq!(
            buffer.position(550, Duration::from_millis(elapsed)),
            expected
        );
    }
    assert_eq!(Arc::strong_count(&buffer.samples), 1);
    assert_eq!(buffer.samples.len(), 400);
}
#[test]
fn frame_rounding_is_consistent_at_fractional_millisecond_rates() {
    let range = LoopRange {
        start_ms: 7,
        end_ms: 108,
    };
    let buffer = LoopBuffer::decode(
        SamplesBuffer::new(
            1.try_into().unwrap(),
            22050.try_into().unwrap(),
            vec![0.25; 2227],
        ),
        range,
    )
    .unwrap();
    assert_eq!(buffer.samples.len(), 2227);
    assert_eq!(buffer.position(7, Duration::from_millis(101)), 7);
    assert_eq!(buffer.source(7).take(2227 * 4).count(), 8908);
}
#[test]
fn bounds_invalid_samples_and_short_files_are_rejected_before_publication() {
    for range in [
        LoopRange {
            start_ms: 5,
            end_ms: 4,
        },
        LoopRange {
            start_ms: 0,
            end_ms: 99,
        },
        LoopRange {
            start_ms: 0,
            end_ms: 60001,
        },
    ] {
        assert!(range.validate(70000).is_err());
    }
    assert!(
        LoopRange {
            start_ms: 100,
            end_ms: 200
        }
        .validate(199)
        .is_err()
    );
    let range = LoopRange {
        start_ms: 0,
        end_ms: 60000,
    };
    assert!(
        LoopBuffer::decode(
            SamplesBuffer::new(
                2.try_into().unwrap(),
                192_000.try_into().unwrap(),
                vec![0.0]
            ),
            range
        )
        .err()
        .unwrap()
        .contains("64 MiB")
    );
    let range = LoopRange {
        start_ms: 0,
        end_ms: 100,
    };
    for samples in [vec![0.0; 99], vec![f32::NAN; 100]] {
        assert!(
            LoopBuffer::decode(
                SamplesBuffer::new(1.try_into().unwrap(), 1000.try_into().unwrap(), samples),
                range
            )
            .is_err()
        );
    }
}

fn audio_file() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("loop.wav");
    let size = 16000_u32;
    let mut bytes = Vec::new();
    bytes.extend(b"RIFF");
    bytes.extend((36 + size).to_le_bytes());
    bytes.extend(b"WAVEfmt ");
    bytes.extend(16_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(8000_u32.to_le_bytes());
    bytes.extend(16000_u32.to_le_bytes());
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend(16_u16.to_le_bytes());
    bytes.extend(b"data");
    bytes.extend(size.to_le_bytes());
    for i in 0..8000_i16 {
        bytes.extend(i.to_le_bytes());
    }
    std::fs::write(&path, bytes).unwrap();
    (dir, path)
}
#[test]
fn decoded_range_honors_trim_and_rejects_stale_or_failed_installation() {
    let (_dir, path) = audio_file();
    let mut transport = crate::Transport::default();
    transport.load(path, 100, 900).unwrap();
    transport.seek(350).unwrap();
    let range = LoopRange {
        start_ms: 200,
        end_ms: 600,
    };
    let prepared = transport
        .loop_request(Some(range))
        .unwrap()
        .prepare()
        .unwrap();
    let buffer = prepared.buffer.as_ref().unwrap();
    // The loop starts 300 ms into the file, at sample 2400, independent of the current cursor.
    assert!((buffer.source(200).next().unwrap() - 2400.0 / 32768.0).abs() < 0.0001);
    transport.apply_loop(prepared).unwrap();
    assert_eq!(transport.position().position_ms, 350);
    assert_eq!(transport.position().loop_range, Some(range));
    transport.stop();
    assert_eq!(transport.position().position_ms, 200);
    let stale = transport.loop_request(None).unwrap().prepare().unwrap();
    transport.pause();
    assert!(transport.apply_loop(stale).is_err());
    assert_eq!(transport.position().loop_range, Some(range));
    transport.seek(599).unwrap();
    assert_eq!(transport.position().loop_range, Some(range));
    transport.seek(600).unwrap();
    assert_eq!(transport.position().loop_range, None);
    assert_eq!(transport.position().position_ms, 600);
    transport.stop();
    assert_eq!(transport.position().position_ms, 0);
    let prepared = transport
        .loop_request(Some(range))
        .unwrap()
        .prepare()
        .unwrap();
    transport.apply_loop(prepared).unwrap();
    assert_eq!(transport.position().position_ms, 200);
    // Preparation failure keeps the last usable buffer, cursor and range.
    std::fs::remove_file(transport.loop_request(None).unwrap().file).unwrap();
    assert!(
        transport
            .loop_request(Some(range))
            .unwrap()
            .prepare()
            .is_err()
    );
    assert_eq!(transport.position().loop_range, Some(range));
    assert_eq!(transport.position().position_ms, 200);
    let prepared = transport.loop_request(None).unwrap().prepare().unwrap();
    transport.apply_loop(prepared).unwrap();
    assert_eq!(transport.position().loop_range, None);
    assert_eq!(transport.position().position_ms, 200);
    transport.clear();
    assert!(transport.loop_request(Some(range)).is_err());
}
