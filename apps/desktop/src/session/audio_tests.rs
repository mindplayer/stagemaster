use super::*;
use crate::audio::Command;
use stagemaster_audio::LoopRange;
use stagemaster_project::{AudioAsset, AudioEdit, AudioMarker};
pub(super) fn session() -> (Session, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("rehearsal.wav");
    let mut bytes = Vec::new();
    let size = 32000_u32;
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
    bytes.resize(44 + usize::try_from(size).unwrap(), 0);
    std::fs::write(&path, bytes).unwrap();
    let mut doc = Document::new("局部排练").unwrap();
    doc.edit(EditCommand::Audio {
        command: AudioEdit::SetAsset {
            asset: AudioAsset {
                digest: "ab".repeat(32),
                file_name: "rehearsal.wav".into(),
                extension: "wav".into(),
                duration_ms: 2000,
            },
        },
    })
    .unwrap();
    let track = doc.audio_timeline().unwrap();
    let mut s = Session::default();
    s.replace(doc, None);
    s.load_audio(s.generation, path, track).unwrap();
    (s, dir)
}
#[test]
fn loop_is_transient_and_marker_edits_keep_it_but_trim_releases_it() {
    let (mut s, _dir) = session();
    let range = LoopRange {
        start_ms: 500,
        end_ms: 1500,
    };
    let original = s.document.clone();
    let request = s
        .audio_loop_request(s.generation, Some(range))
        .unwrap()
        .prepare()
        .unwrap();
    let pos = s.apply_audio_loop(s.generation, request).unwrap();
    assert_eq!(pos.position_ms, 500);
    assert_eq!(pos.loop_range, Some(range));
    assert_eq!(s.document, original);
    assert!(s.undo.is_empty());
    s.edit(
        s.generation,
        EditCommand::Audio {
            command: AudioEdit::PutMarker {
                marker: AudioMarker {
                    id: "a0000000-0000-4000-8000-000000000001".into(),
                    name: "排练点".into(),
                    time_ms: 600,
                    scene_id: None,
                    fade_ms: 0,
                },
            },
        },
    )
    .unwrap();
    assert_eq!(s.audio.position().loop_range, Some(range));
    s.edit(
        s.generation,
        EditCommand::Audio {
            command: AudioEdit::Trim {
                in_ms: 100,
                out_ms: 1900,
            },
        },
    )
    .unwrap();
    assert_eq!(s.audio.position().loop_range, None);
    assert!(!s.audio.active());
}
#[test]
fn delayed_preparation_cannot_override_new_seek_or_new_project() {
    let (mut s, _dir) = session();
    let range = Some(LoopRange {
        start_ms: 500,
        end_ms: 1500,
    });
    let prepared = s
        .audio_loop_request(s.generation, range)
        .unwrap()
        .prepare()
        .unwrap();
    s.audio_request(s.generation, Command::Seek { position_ms: 300 })
        .unwrap();
    assert!(s.apply_audio_loop(s.generation, prepared).is_err());
    assert_eq!(s.audio.position().position_ms, 300);
    assert_eq!(s.audio.position().loop_range, None);
    let prepared = s
        .audio_loop_request(s.generation, range)
        .unwrap()
        .prepare()
        .unwrap();
    let old = s.generation;
    s.replace(Document::new("另一个工程").unwrap(), None);
    assert!(s.apply_audio_loop(old, prepared).is_err());
    assert!(!s.audio.active());
}
