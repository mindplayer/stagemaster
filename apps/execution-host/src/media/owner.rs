use super::{OutputKind, Runner, Setup};
use serde::Serialize;
use stagemaster_live::media::Preparer;
use stagemaster_live_host::{
    Live,
    media::{LocalClock, MediaPort},
};
use stagemaster_project::Document;
use stagemaster_runtime_host::{Host, Observer};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct View {
    pub output: OutputKind,
    pub status: &'static str,
    pub position_ms: u64,
    pub duration_ms: u64,
    pub instance: Option<String>,
    pub frames: String,
    pub loop_state: Option<super::looping::LoopState>,
    pub problem: Option<String>,
}
pub(crate) struct Owner {
    pub observer: Observer<Live>,
    cancel: Arc<AtomicBool>,
    view: Arc<Mutex<View>>,
    thread: Mutex<Option<JoinHandle<()>>>,
}
impl Owner {
    pub fn start(
        setup: Setup,
        doc: Document,
        prepare: Preparer,
        port: MediaPort,
        host: &Host<Live>,
        target: stagemaster_time::Clock,
    ) -> Result<Self, String> {
        let cancel = Arc::new(AtomicBool::new(false));
        let view = Arc::new(Mutex::new(View {
            output: setup.output,
            status: "ready",
            position_ms: 0,
            duration_ms: setup.duration_ms,
            instance: None,
            frames: "0".into(),
            loop_state: None,
            problem: None,
        }));
        let observer = host.observer();
        let mut runner = Runner {
            transport: setup.transport,
            software: setup.software,
            doc,
            prepare,
            port,
            observer: observer.clone(),
            clock: host.clock(),
            mapping: LocalClock::new(host.clock(), setup.group.clock, target, 5_000_000_000)
                .map_err(|e| e.to_string())?,
            cancel: cancel.clone(),
            view: view.clone(),
            active: None,
            request: None,
            staged: None,
            pending_sample: None,
            seen: None,
            last_sample: 0,
            terminal: None,
            pending_end: None,
        };
        let thread = thread::Builder::new()
            .name("stage-media".into())
            .spawn(move || {
                if let Some(output) = &mut runner.software {
                    output.reset_clock();
                }
                while !runner.cancel.load(Ordering::Acquire) {
                    if let Err(message) = runner.tick() {
                        runner.fail(message);
                    }
                    thread::sleep(Duration::from_millis(5));
                }
                runner.cancel_job();
                runner.transport.clear();
            })
            .map_err(|e| e.to_string())?;
        Ok(Self {
            observer,
            cancel,
            view,
            thread: Mutex::new(Some(thread)),
        })
    }
    pub fn view(&self) -> serde_json::Value {
        self.view.lock().map_or_else(
            |_| serde_json::json!({"status":"unavailable"}),
            |v| serde_json::json!(*v),
        )
    }
    pub fn close(&self) -> Result<(), String> {
        self.cancel.store(true, Ordering::Release);
        if let Some(thread) = self.thread.lock().map_err(|_| "媒体线程句柄不可用")?.take()
        {
            thread.join().map_err(|_| "媒体线程异常退出")?;
        }
        Ok(())
    }
}
impl Drop for Owner {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
