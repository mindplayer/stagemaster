use super::Process;
use std::{
    io::Read,
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

/// Refusal tests must fail promptly if a malformed input accidentally starts a resident process.
pub fn rejected(command: &mut Command, run: &Path) -> String {
    let mut child = Process(
        command
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(10);
    let status = loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            break status;
        }
        assert!(
            !run.join("discovery.json").exists(),
            "错误输入启动了后台进程"
        );
        assert!(Instant::now() < deadline, "错误输入未被及时拒绝");
        std::thread::sleep(Duration::from_millis(10));
    };
    assert!(!status.success(), "错误输入被成功接纳");
    assert!(!run.join("discovery.json").exists());
    let mut error = String::new();
    child
        .0
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut error)
        .unwrap();
    error
}
