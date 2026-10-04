//! Controlled child retains the parent's cloned manager descriptor on standard input.
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

pub(super) struct Holder {
    child: Child,
    directory: PathBuf,
    finished: bool,
}
impl Holder {
    pub(super) fn start(directory: &Path, inherited: File) -> Self {
        let log = File::create(directory.join("child.log")).unwrap();
        let mut holder = Self {
            child: Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "execution::manager_lock::tests::holder::inherited_file_holder",
                    "--ignored",
                    "--nocapture",
                ])
                .env("EXEC015_LOCK_DIRECTORY", directory)
                .stdin(Stdio::from(inherited))
                .stdout(log.try_clone().unwrap())
                .stderr(log)
                .spawn()
                .unwrap(),
            directory: directory.into(),
            finished: false,
        };
        let end = Instant::now() + Duration::from_secs(6);
        while !directory.join("child-ready").exists() {
            assert!(holder.running(), "child exited early");
            assert!(Instant::now() < end, "child ready deadline");
            std::thread::sleep(Duration::from_millis(5));
        }
        holder
    }
    pub(super) fn id(&self) -> u32 {
        self.child.id()
    }
    pub(super) fn running(&mut self) -> bool {
        self.child.try_wait().unwrap().is_none()
    }
    pub(super) fn finish(&mut self) {
        fs::write(self.directory.join("child-release"), "release").unwrap();
        let status = self.child.wait().unwrap();
        self.finished = true;
        assert!(
            status.success(),
            "{}",
            fs::read_to_string(self.directory.join("child.log")).unwrap()
        );
    }
}
impl Drop for Holder {
    fn drop(&mut self) {
        if !self.finished {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

#[test]
#[ignore = "subprocess entry; exercised by released_manager_transaction_does_not_remain_owned_by_a_live_child"]
fn inherited_file_holder() {
    let dir = PathBuf::from(std::env::var_os("EXEC015_LOCK_DIRECTORY").unwrap());
    fs::write(dir.join("child-ready"), "ready").unwrap();
    let end = Instant::now() + Duration::from_secs(6);
    while !dir.join("child-release").exists() {
        assert!(Instant::now() < end, "parent release deadline");
        std::thread::sleep(Duration::from_millis(5));
    }
}
