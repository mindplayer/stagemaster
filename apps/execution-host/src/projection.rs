use serde_json::{Value, json};
use stagemaster_runtime::{ProgramKey, State};
use stagemaster_runtime_host::{Observation, Phase};
use uuid::Uuid;

fn key(key: ProgramKey) -> Value {
    json!({"kind":format!("{:?}",key.kind),"id":Uuid::from_bytes(key.id).to_string()})
}
/// Deliberately project fields; never serialize the core Owner/Lease directly.
pub(crate) fn state(state: State) -> Value {
    json!({
        "boot":Uuid::from_bytes(state.boot).to_string(),
        "revision":state.revision.to_string(),"observedMs":state.observed_ms.to_string(),
        "mode":format!("{:?}",state.mode),"status":state.status.map(|s|format!("{s:?}")),
        "selected":state.selected.map(key),"loaded":state.loaded.map(key),
        "instance":state.instance.map(|i| json!({"boot":Uuid::from_bytes(i.boot).to_string(),"number":i.number.to_string()})),
        "step":state.step.map(|id|Uuid::from_bytes(id).to_string()),"elapsedMs":state.elapsed_ms.to_string(),
        "owner":state.owner.map(|o|json!({"sessionId":Uuid::from_bytes(o.principal).to_string(),"expiresMs":o.expires_ms.to_string()}))
    })
}
pub(crate) fn observation(value: &Observation) -> Value {
    json!({
        "phase":match value.phase {Phase::Running=>"running",Phase::Stopping=>"stopping",Phase::Stopped=>"stopped",Phase::Faulted=>"faulted"},
        "fault":value.fault.map(|f|format!("{f:?}")),
        "snapshot":value.snapshot.map(|s|json!({
            "state":state(s.state),"cycles":s.cycles.to_string(),
            "missedPeriods":s.missed_periods.to_string(),"maxLatenessMs":s.max_lateness_ms.to_string(),
            "skippedPublications":s.skipped_publications.to_string(),
            "frame":s.frame.map(|f|json!({"kind":"softwareSample","universe":f.info.universe,"revision":f.info.revision.to_string(),"sampledMs":f.info.sampled_ms.to_string(),"slots":f.slots.as_slice()}))
        }))
    })
}
