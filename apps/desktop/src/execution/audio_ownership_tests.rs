use super::{
    media_tests::{music, operation},
    tests::{Cleanup, binary, connected, runtime},
    *,
};
use stagemaster_audio::{OutputBinding, OutputScope, Transport};
use stagemaster_execution_client::MediaAction;

fn editor(scope: &OutputScope, source: PathBuf) -> (Transport, rodio::mixer::MixerSource) {
    let (mixer, samples) = rodio::mixer::mixer(1.try_into().unwrap(), 8000.try_into().unwrap());
    let mut transport = Transport::with_output(OutputBinding::new(mixer));
    transport.set_output_scope(scope.clone()).unwrap();
    transport.load(source, 0, 5000).unwrap();
    (transport, samples)
}

#[test]
fn continuing_editor_voice_blocks_background_after_command_guard_returns() {
    runtime().block_on(async {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("execution");
        let scope = OutputScope::new(root.clone()).unwrap();
        let (doc, input) = music(temp.path());
        let (mut voice, mut samples) = editor(&scope, input.source.clone());
        let mut first = Manager::new(root.clone(), binary());
        let guard = first.reserve_editor_audio().unwrap();
        voice.play().unwrap();
        for _ in 0..160 {
            assert!(samples.next().is_some());
        }
        drop(guard); // This was the old hole: the player still owns its output.
        let mut rival = Manager::new(root.clone(), binary());
        let selection = vec![Selection::AudioTimeline {}];
        let retry = || super::super::media::AudioInput {
            source: input.source.clone(),
            output: input.output,
        };
        assert!(
            rival
                .prepare(doc.clone(), selection.clone(), Some(retry()))
                .await
                .err()
                .unwrap()
                .contains("占用")
        );
        assert!(!root.join("current").exists());
        voice.pause();
        assert!(
            rival
                .prepare(doc.clone(), selection.clone(), Some(retry()))
                .await
                .is_err()
        );
        voice.stop();
        rival.prepare(doc, selection, Some(input)).await.unwrap();
        let _cleanup = Cleanup(rival.child.take().unwrap());
        let view = connected(&mut rival).await.runtime.unwrap();
        assert!(voice.play().unwrap_err().contains("占用"));
        assert!(first.reserve_editor_audio().is_err());
        rival.acquire(false).await.unwrap();
        for action in [
            MediaAction::Play {},
            MediaAction::Pause {},
            MediaAction::Stop {},
        ] {
            operation(&mut rival, action).await;
            assert!(scope.reserve().is_err());
        }
        drop(rival);
        let mut reopened = Manager::new(root, binary());
        assert!(!connected(&mut reopened).await.runtime.unwrap().controlling);
        assert!(voice.play().is_err());
        reopened.shutdown(&view.host_id).await.unwrap();
        assert!(reopened.reserve_editor_audio().is_ok());
        voice.play().unwrap();
        voice.stop();
        assert!(scope.reserve().is_ok());
    });
}

#[test]
fn real_process_death_releases_output_without_deleting_the_lock_file() {
    runtime().block_on(async {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("execution");
        let scope = OutputScope::new(root.clone()).unwrap();
        let (doc, input) = music(temp.path());
        let mut manager = Manager::new(root.clone(), binary());
        manager
            .prepare(doc, vec![Selection::AudioTimeline {}], Some(input))
            .await
            .unwrap();
        connected(&mut manager).await;
        let mut process = Cleanup(manager.child.take().unwrap());
        let run = manager.run.clone().unwrap();
        assert!(scope.reserve().is_err());
        // A missing discovery file is not process death and must not release ownership.
        std::fs::remove_file(run.join("host/discovery.json")).unwrap();
        assert!(process.0.try_wait().unwrap().is_none());
        assert!(manager.reserve_editor_audio().is_err());
        assert!(scope.reserve().is_err());
        process.0.kill().unwrap();
        process.0.wait().unwrap();
        assert!(files::ended(&run));
        assert!(root.join("audio-output.lock").is_file());
        let reservation = manager.reserve_editor_audio().unwrap();
        assert!(scope.reserve().is_ok());
        drop(reservation);
    });
}
