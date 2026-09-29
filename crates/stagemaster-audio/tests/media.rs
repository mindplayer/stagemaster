use stagemaster_audio::{Resources, analyze, verify};
use std::{fs, path::Path, sync::atomic::AtomicBool};
fn wav(path: &Path, channels: u16) {
    let rate = 8000_u32;
    let samples = 16000_u32 * u32::from(channels);
    let mut bytes = Vec::new();
    bytes.extend(b"RIFF");
    bytes.extend((36 + samples * 2).to_le_bytes());
    bytes.extend(b"WAVEfmt ");
    bytes.extend(16_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(channels.to_le_bytes());
    bytes.extend(rate.to_le_bytes());
    bytes.extend((rate * u32::from(channels) * 2).to_le_bytes());
    bytes.extend((channels * 2).to_le_bytes());
    bytes.extend(16_u16.to_le_bytes());
    bytes.extend(b"data");
    bytes.extend((samples * 2).to_le_bytes());
    for sample in 0..samples {
        bytes.extend(
            if sample < samples / 2 {
                0_i16
            } else {
                16384_i16
            }
            .to_le_bytes(),
        );
    }
    fs::write(path, bytes).unwrap();
}
#[test]
fn peaks_are_streamed_at_fixed_resolution_and_invalid_input_is_rejected() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("test.wav");
    wav(&path, 2);
    let waveform = analyze(&path, &AtomicBool::new(false)).unwrap();
    assert_eq!(waveform.duration_ms, 2000);
    assert_eq!(waveform.channels.len(), 2);
    assert_eq!(waveform.channels[0].len(), 400);
    assert!(waveform.channels[0][..200].iter().all(|p| *p == 0.0));
    assert!(
        waveform.channels[0][200..]
            .chunks_exact(2)
            .all(|p| (p[0] - 0.5).abs() < 0.001 && (p[1] - 0.5).abs() < 0.001)
    );
    assert!(
        analyze(&path, &AtomicBool::new(true))
            .unwrap_err()
            .contains("取消")
    );
    wav(&path, 3);
    assert!(
        analyze(&path, &AtomicBool::new(false))
            .unwrap_err()
            .contains("声道")
    );
    fs::write(&path, b"not audio").unwrap();
    assert!(analyze(&path, &AtomicBool::new(false)).is_err());
}
#[test]
fn signed_stereo_envelopes_keep_channels_and_fractional_bucket_boundaries() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("stereo.wav");
    wav(&path, 2);
    let mut bytes = fs::read(&path).unwrap();
    bytes[24..28].copy_from_slice(&22050_u32.to_le_bytes());
    bytes[28..32].copy_from_slice(&88200_u32.to_le_bytes());
    for frame in bytes[44..].chunks_exact_mut(4) {
        frame[..2].copy_from_slice(&(-16384_i16).to_le_bytes());
        frame[2..].copy_from_slice(&8192_i16.to_le_bytes());
    }
    fs::write(&path, bytes).unwrap();
    let wave = analyze(&path, &AtomicBool::new(false)).unwrap();
    assert_eq!(wave.duration_ms, 725);
    assert_eq!(wave.bucket_ms, 10);
    assert_eq!(wave.channels[0].len(), 146);
    assert_eq!(wave.channels[1].len(), 146);
    assert!(wave.channels[0].iter().all(|v| (*v + 0.5).abs() < 0.001));
    assert!(wave.channels[1].iter().all(|v| (*v - 0.25).abs() < 0.001));
    // The first complete 22.05 kHz bucket ends slightly after 10 ms.
    // It must not create an empty, infinite min/max pair at EOF.
    let mut bytes = fs::read(&path).unwrap();
    let data_bytes = 221_u32 * 4;
    bytes.truncate(44 + data_bytes as usize);
    bytes[4..8].copy_from_slice(&(36 + data_bytes).to_le_bytes());
    bytes[40..44].copy_from_slice(&data_bytes.to_le_bytes());
    fs::write(&path, bytes).unwrap();
    let wave = analyze(&path, &AtomicBool::new(false)).unwrap();
    assert_eq!(wave.duration_ms, 10);
    assert_eq!(wave.channels, vec![vec![-0.5, -0.5], vec![0.25, 0.25]]);
}
#[test]
fn hour_long_envelopes_stay_bounded_and_excess_duration_is_rejected() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("long.wav");
    wav(&path, 2);
    let mut header = fs::read(&path).unwrap()[..44].to_vec();
    let size = 8000_u32 * 3600 * 4;
    header[4..8].copy_from_slice(&(36 + size).to_le_bytes());
    header[40..44].copy_from_slice(&size.to_le_bytes());
    fs::write(&path, &header).unwrap();
    fs::OpenOptions::new()
        .write(true)
        .open(&path)
        .unwrap()
        .set_len(u64::from(size) + 44)
        .unwrap();
    let wave = analyze(&path, &AtomicBool::new(false)).unwrap();
    assert_eq!(wave.duration_ms, 3_600_000);
    assert_eq!(wave.channels.len(), 2);
    for channel in &wave.channels {
        assert_eq!(channel.len(), 720_000);
        assert!(channel.iter().all(|v| *v == 0.0));
    }
    header[4..8].copy_from_slice(&(40 + size).to_le_bytes());
    header[40..44].copy_from_slice(&(size + 4).to_le_bytes());
    fs::write(&path, &header).unwrap();
    fs::OpenOptions::new()
        .write(true)
        .open(&path)
        .unwrap()
        .set_len(u64::from(size) + 48)
        .unwrap();
    assert!(
        analyze(&path, &AtomicBool::new(false))
            .unwrap_err()
            .contains("1 小时")
    );
}
#[test]
fn resource_archive_is_portable_and_changed_music_cannot_silently_replace_it() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let source = root.join("song.wav");
    wav(&source, 1);
    let cache = root.join("cache");
    let resources = Resources::new(cache.clone());
    let cancel = AtomicBool::new(false);
    let (digest, path) = resources.import(&source, "wav", None, &cancel).unwrap();
    verify(&path, &digest, &cancel).unwrap();
    let target = root.join("节目.json");
    resources.archive(&digest, "wav", None, &target).unwrap();
    fs::remove_dir_all(cache).unwrap();
    fs::remove_file(source).unwrap();
    let archived = resources.resolve(&digest, "wav", Some(&target)).unwrap();
    assert!(archived.to_string_lossy().contains("节目.json.assets"));
    verify(&archived, &digest, &cancel).unwrap();
    let original = fs::read(&archived).unwrap();
    fs::write(&archived, b"different music").unwrap();
    assert!(verify(&archived, &digest, &cancel).is_err());
    assert!(
        resources
            .import(&archived, "wav", Some(&digest), &cancel)
            .unwrap_err()
            .contains("不同")
    );
    assert!(
        resources
            .resolve("../../bad", "wav", Some(&target))
            .is_err()
    );
    assert!(
        resources
            .resolve(&digest, "../../bad", Some(&target))
            .is_err()
    );
    let restored = root.join("restored.wav");
    fs::write(&restored, original).unwrap();
    resources
        .import(&restored, "wav", Some(&digest), &cancel)
        .unwrap();
    resources.archive(&digest, "wav", None, &target).unwrap();
    verify(&archived, &digest, &cancel).unwrap();
    fs::remove_dir_all(root.join("cache")).unwrap();
    fs::remove_file(&archived).unwrap();
    assert!(resources.resolve(&digest, "wav", Some(&target)).is_err());
}
#[test]
fn cancelled_or_oversized_import_does_not_publish_a_resource() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let path = root.join("song.wav");
    wav(&path, 1);
    let resources = Resources::new(root.join("cache"));
    assert!(
        resources
            .import(&path, "wav", None, &AtomicBool::new(true))
            .is_err()
    );
    assert_eq!(fs::read_dir(root.join("cache")).unwrap().count(), 0);
    fs::File::create(&path)
        .unwrap()
        .set_len(stagemaster_audio::MAX_FILE_BYTES + 1)
        .unwrap();
    assert!(
        resources
            .import(&path, "wav", None, &AtomicBool::new(false))
            .unwrap_err()
            .contains("512")
    );
    assert_eq!(fs::read_dir(root.join("cache")).unwrap().count(), 0);
}
