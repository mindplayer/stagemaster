use super::Catalog;
use serde_json::{Value, json};
use stagemaster_live_host::{Live, State};
use stagemaster_runtime_host::Frame;
use std::fmt::Write;
use uuid::Uuid;

pub(super) fn state(s: &State, catalog: &Catalog) -> Value {
    let sources: Vec<_> = s
        .sources
        .iter()
        .enumerate()
        .filter_map(|(index, source)| source.as_ref().map(|source| (index, source)))
        .map(|(index, source)| {
            let entry = catalog.entries.iter().find(|e| e.key == source.key);
            let step = source.step.and_then(|i| entry.and_then(|e| e.steps.get(i)));
            let mut value = json!({"id":Uuid::from_bytes(source.id).to_string(),"level":source.level,
            "status":source.status.map(|s|format!("{s:?}")),"step":step.map(|s|&s.id),
            "progress": source.progress.map(|p| progress(p, entry))});
            if let Some(values) = &s.manual_values[index] {
                let (held, readings): (Vec<_>, Vec<_>) = catalog.output.manual_readings(values)
                    .map(|(fixture, attribute, reading)| (json!({"fixtureId":fixture,"attribute":attribute}), reading))
                    .unzip();
                value["held"] = held.into();
                value["heldValues"] = readings.into();
            }
            value
        })
        .collect();
    let value = json!({"boot":Uuid::from_bytes(s.boot).to_string(),"layout":identity(s.layout),
        "revision":s.revision.to_string(),"observedMs":s.observed_ms.to_string(),"sources":sources,"fault":s.fault,
        "owner":s.owner.map(|o|json!({"sessionId":Uuid::from_bytes(o.principal).to_string(),"expiresMs":o.expires_ms.to_string()}))});
    #[cfg(feature = "audio")]
    {
        let mut value = value;
        if let Some(owner) = &catalog.media {
            value["media"] = crate::media::wire::state(s);
            value["audio"] = owner.view();
        }
        value
    }
    #[cfg(not(feature = "audio"))]
    value
}
fn progress(p: stagemaster_live::Progress, entry: Option<&super::Entry>) -> Value {
    use stagemaster_live::Phase;
    let phase = match p.phase {
        Phase::Idle => "idle",
        Phase::Delay => "delay",
        Phase::Fade => "fade",
        Phase::Wait => "wait",
        Phase::Hold => "hold",
        Phase::Finished => "finished",
    };
    json!({"phase":phase,"elapsedMs":p.elapsed_ms.to_string(),
        "phaseElapsedMs":p.phase_elapsed_ms.to_string(),
        "phaseDurationMs":p.phase_duration_ms.map(|v|v.to_string()),
        "nextStep":p.next_step.and_then(|i|entry.and_then(|e|e.steps.get(i))).map(|s|&s.id),
        "nextWrap":p.next_wrap})
}
pub(super) fn frame(f: &Frame<Live>) -> Value {
    json!({"kind":"softwareSample","boot":Uuid::from_bytes(f.info.boot).to_string(),"layout":identity(f.info.layout),
        "universe":f.info.universe,"revision":f.info.revision.to_string(),"sampledMs":f.info.sampled_ms.to_string(),
        "compositionVersion":f.info.sequence.to_string(),"slots":f.slots.as_slice()})
}
pub(super) fn identity(bytes: [u8; 32]) -> String {
    bytes
        .iter()
        .fold(String::with_capacity(64), |mut value, byte| {
            let _ = write!(&mut value, "{byte:02x}");
            value
        })
}
