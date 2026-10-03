#[path = "../../../apps/esp32-player/src/installation/frame_metrics.rs"]
mod frame_metrics;
#[allow(dead_code)]
mod maintenance_support;
use frame_metrics::{Metrics, Sample, fingerprint};
use maintenance_support::{Device, acquire, apply, fixture, install};
use stagemaster_runtime::{Action, Code, ProgramKey};

fn sample(device: &Device, time_us: u64, online: bool) -> Sample {
    Sample {
        start_us: time_us,
        finish_us: time_us + 120,
        online,
        backpressured: false,
        state: device.state(),
    }
}
fn render(metrics: &mut Metrics, device: &mut Device, ms: u64, online: bool) {
    device.tick(ms).unwrap();
    let mut bytes = [0xa5; 512];
    let frame = device.render(&mut bytes);
    assert!(frame.as_ref().unwrap().is_some());
    metrics.record(
        &sample(device, ms * 1000, online),
        &frame,
        fingerprint(&bytes),
    );
}

#[test]
fn actual_worker_frames_are_counted_offline_but_paused_and_stopped_are_not_running() {
    let (_dir, mut device, _memory, bytes) = fixture();
    install(&mut device, &bytes);
    device.finish_maintenance(0).unwrap();
    let lease = acquire(&mut device, 0);
    let scene = device
        .catalog()
        .iter()
        .find(|p| p.kind == stagemaster_package::Kind::Scene)
        .unwrap();
    let key = ProgramKey {
        kind: scene.kind,
        id: scene.id,
    };
    apply(&mut device, lease, Action::Select(key), 0).unwrap();
    apply(&mut device, lease, Action::Load, 0).unwrap();
    let step = device.steps()[0].id;
    apply(&mut device, lease, Action::Start { step }, 0).unwrap();
    let mut metrics = Metrics::default();
    metrics.command();
    for tick in 1..=80 {
        render(&mut metrics, &mut device, tick * 25, tick <= 40);
    }
    let running = metrics.report();
    assert_eq!(
        (running.attempts, running.frames, running.running_frames),
        (80, 80, 80)
    );
    assert_eq!(running.offline_frames, 40);
    assert_eq!(running.elapsed_ms, 2000);
    assert_eq!(running.min_gap_us, Some(25_000));
    assert_eq!(running.max_gap_us, 25_000);
    assert_eq!(
        (
            running.errors,
            running.clock_errors,
            running.over_30ms,
            running.repeated_samples
        ),
        (0, 0, 0, 0)
    );
    assert_ne!(running.instance, 0);
    apply(&mut device, lease, Action::Pause, 2000).unwrap();
    render(&mut metrics, &mut device, 2025, false);
    assert_eq!(metrics.report().elapsed_ms, 2000);
    assert_eq!(metrics.report().frames, 81);
    assert_eq!(metrics.report().running_frames, 80);
    assert_eq!(metrics.report().instance, running.instance);
    apply(&mut device, lease, Action::Stop, 2025).unwrap();
    render(&mut metrics, &mut device, 2050, false);
    assert_eq!(metrics.report().instance, 0);
    assert_eq!(metrics.report().offline_frames, 40);
    let line = metrics.report().to_string();
    assert!(line.starts_with("RUNTIME_SAMPLE v=1 "));
    assert!(line.contains("frames=82 running_frames=80 offline_frames=40"));
    assert!(line.contains("commands=1"));
}

#[test]
fn missing_and_failed_frames_never_count_stale_buffer_or_fingerprint() {
    let (_dir, device, _memory, _bytes) = fixture();
    let mut metrics = Metrics::default();
    let mut target = [0xa5; 512];
    let missing = device.render(&mut target);
    assert_eq!(missing, Ok(None));
    assert_eq!(target, [0xa5; 512]);
    metrics.record(&sample(&device, 25_000, false), &missing, 123);
    metrics.record(&sample(&device, 50_000, false), &Err(Code::Playback), 456);
    metrics.failed();
    let report = metrics.report();
    assert_eq!((report.attempts, report.absent, report.errors), (2, 1, 2));
    assert_eq!(
        (report.frames, report.running_frames, report.offline_frames),
        (0, 0, 0)
    );
    assert_eq!(report.fingerprint, 0);
}

#[test]
fn backpressure_long_gaps_and_clock_regressions_are_retained_in_diagnostics() {
    let (_dir, device, _memory, _bytes) = fixture();
    let mut metrics = Metrics::default();
    metrics.record(&sample(&device, 0, false), &Ok(None), 0);
    let mut delayed = sample(&device, 51_000, true);
    delayed.backpressured = true;
    delayed.finish_us = 52_000;
    metrics.record(&delayed, &Ok(None), 0);
    let mut regressed = sample(&device, 40_000, false);
    regressed.finish_us = 39_999;
    metrics.record(&regressed, &Ok(None), 0);
    let report = metrics.report();
    assert_eq!(
        (report.over_30ms, report.over_50ms, report.backpressure),
        (1, 1, 1)
    );
    assert_eq!(report.clock_errors, 2);
    assert_eq!(report.max_gap_us, 51_000);
    assert_eq!(report.min_gap_us, Some(51_000));
    assert_eq!(report.max_work_us, 1000);
    assert_ne!(fingerprint(&[0; 512]), fingerprint(&[1; 512]));
}
#[path = "../../../apps/esp32-player/src/diagnostics/chunked.rs"]
mod chunked_diagnostics;
