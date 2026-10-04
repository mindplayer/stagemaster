use super::*;
use std::{fs, sync::atomic::AtomicBool, thread, time::Duration};
// All mounts share the actual two-decoder budget in this test executable.
static MOUNTS: Mutex<()> = Mutex::new(());
fn observed(
    transport: &Transport,
    predicate: impl Fn(&stagemaster_audio::PerformanceObservation) -> bool,
) -> stagemaster_audio::PerformanceObservation {
    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        match transport.performance_observation() {
            Ok(Some(value)) if value.snapshot.render.is_some() && predicate(&value) => {
                return value;
            }
            Ok(_) => {}
            Err(error) if error == "音乐观测正在更新，请重试读取" => {}
            Err(error) => panic!("{error}"),
        }
        assert!(
            Instant::now() < deadline,
            "real callback did not confirm the original request"
        );
        thread::sleep(Duration::from_millis(1));
    }
}

fn mounted() -> (tempfile::TempDir, Transport, SoftwareOutput) {
    let directory =
        tempfile::tempdir_in(concat!(env!("CARGO_MANIFEST_DIR"), "/../../tmp")).unwrap();
    let path = directory.path().join("silent.wav");
    let mut bytes = Vec::new();
    bytes.extend(b"RIFF");
    bytes.extend(32036_u32.to_le_bytes());
    bytes.extend(b"WAVEfmt ");
    bytes.extend(16_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend(8000_u32.to_le_bytes());
    bytes.extend(32000_u32.to_le_bytes());
    bytes.extend(4_u16.to_le_bytes());
    bytes.extend(16_u16.to_le_bytes());
    bytes.extend(b"data");
    bytes.extend(32000_u32.to_le_bytes());
    bytes.resize(32044, 0);
    fs::write(&path, bytes).unwrap();
    let (mut transport, software) = SoftwareOutput::prepare(OutputKind::Software).unwrap();
    let load = transport
        .load_performance_request(path, 0, 1000, None)
        .unwrap()
        .prepare(&AtomicBool::new(false))
        .unwrap();
    transport.apply_load(load).unwrap();
    transport.prime_performance().unwrap();
    (directory, transport, software.unwrap())
}
#[test]
fn paused_callback_health_survives_slow_owner_work_without_invented_consumption() {
    let _serial = MOUNTS.lock().unwrap();
    let (_directory, transport, output) = mounted();
    output.check().unwrap();
    let before = observed(&transport, |v| {
        v.snapshot.render.unwrap().applied == v.request
    });
    assert!(before.snapshot.consumption.is_none());
    // Simulate the owner compiling a large lighting plan; no health check/read in between.
    thread::sleep(Duration::from_millis(650));
    output
        .check()
        .expect("independent software callback must remain serviced");
    let after = observed(&transport, |_| true);
    assert_eq!(before.instance, after.instance);
    assert_eq!(before.snapshot.position, after.snapshot.position);
    assert_eq!(before.snapshot.consumption, after.snapshot.consumption);
    assert!(after.snapshot.render.unwrap().sequence > before.snapshot.render.unwrap().sequence);
}

#[test]
fn a_real_consumer_gap_remains_terminal_without_catch_up() {
    let (_, source) = rodio::mixer::mixer(2.try_into().unwrap(), 48_000.try_into().unwrap());
    let mut consumer = Consumer {
        source,
        origin: Instant::now(),
        frames: 0,
    };
    consumer.pull_elapsed(Duration::from_millis(500)).unwrap();
    let before = consumer.frames;
    assert_eq!(before, 24000);
    assert_eq!(
        consumer
            .pull_elapsed(Duration::from_millis(1001))
            .unwrap_err(),
        "软件音频消费超过 500 毫秒未运行"
    );
    assert_eq!(consumer.frames, before);
}

#[test]
fn playing_consumes_real_pcm_while_the_owner_is_busy_then_pause_holds_position() {
    let _serial = MOUNTS.lock().unwrap();
    let (_directory, mut transport, output) = mounted();
    transport.play().unwrap();
    let before = observed(&transport, |v| {
        v.snapshot.consumption.is_some() && v.snapshot.render.unwrap().applied == v.request
    });
    thread::sleep(Duration::from_millis(650));
    output.check().unwrap();
    let played = observed(&transport, |_| true);
    assert_eq!(before.instance, played.instance);
    assert!(
        played.snapshot.consumption.unwrap().frames > before.snapshot.consumption.unwrap().frames
    );
    assert!(played.snapshot.position.tick > before.snapshot.position.tick);
    transport.pause();
    let paused = observed(&transport, |v| {
        v.snapshot.render.unwrap().applied == v.request
    });
    thread::sleep(Duration::from_millis(20));
    let held = observed(&transport, |_| true);
    assert_eq!(paused.snapshot.position, held.snapshot.position);
    assert_eq!(paused.snapshot.consumption, held.snapshot.consumption);
    assert!(held.snapshot.render.unwrap().sequence > paused.snapshot.render.unwrap().sequence);
}

#[test]
fn dropping_the_output_stops_and_joins_the_owned_consumer() {
    let _serial = MOUNTS.lock().unwrap();
    let (_directory, transport, output) = mounted();
    let cancel = output.cancel.clone();
    drop(output);
    assert!(cancel.load(Ordering::Acquire));
    assert!(transport.position().problem.is_some());
    let (_next_directory, _next_transport, next) = mounted();
    assert!(!next.cancel.load(Ordering::Acquire));
    next.check().unwrap();
}

#[test]
fn a_failed_consumer_reports_failure_to_the_same_output_binding() {
    let (mixer, source) = rodio::mixer::mixer(2.try_into().unwrap(), 48_000.try_into().unwrap());
    let binding = OutputBinding::new(mixer);
    let transport = Transport::with_output(binding.clone());
    let consumer = Consumer {
        source,
        origin: Instant::now()
            .checked_sub(Duration::from_millis(501))
            .unwrap(),
        frames: 0,
    };
    let output = SoftwareOutput::start(consumer, binding).unwrap();
    let deadline = Instant::now() + Duration::from_secs(1);
    while output.check().is_ok() {
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(
        output.check().unwrap_err(),
        "软件音频消费超过 500 毫秒未运行"
    );
    assert!(transport.position().problem.is_some());
}
