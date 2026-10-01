//! Sample the actual native lighting engine on a music timeline. Never outputs to hardware.
use serde_json::json;
use stagemaster_playback::Player;
use stagemaster_project::{Document, MAX_BYTES};
use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os().nth(1).ok_or("需要音乐工程路径")?;
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take((MAX_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    let doc = Document::decode(&bytes)?;
    let track = doc.audio_timeline().ok_or("工程没有音乐")?;
    let mut samples = Vec::new();
    for marker in track.markers.iter().filter(|m| m.scene_id.is_some()) {
        let end = track
            .markers
            .iter()
            .find(|m| m.scene_id.is_some() && m.time_ms > marker.time_ms)
            .map_or(track.duration_ms(), |m| m.time_ms);
        let compiled = doc.compile_audio_marker(Some(&marker.id))?;
        let mut player = Player::new(compiled.plan, 0);
        player.execute(0, 0)?;
        let mut maximum_lit = 0;
        let mut minimum_lit = usize::MAX;
        let mut sum = 0_u64;
        let mut count = 0;
        for elapsed in (0..end - marker.time_ms).step_by(10) {
            player.advance(elapsed)?;
            let output = compiled.output.render(player.values())?;
            let levels: Vec<_> = output
                .fixtures
                .iter()
                .flat_map(|f| &f.attributes)
                .filter(|v| v.key == "dimmer")
                .map(|v| v.value)
                .collect();
            let lit = levels.iter().filter(|v| **v > 0).count();
            maximum_lit = maximum_lit.max(lit);
            minimum_lit = minimum_lit.min(lit);
            sum += levels.iter().map(|v| u64::from(*v)).sum::<u64>();
            count += 1;
        }
        samples.push(json!({"timeMs":marker.time_ms,"name":marker.name,
            "minLit":minimum_lit,"maxLit":maximum_lit,
            "meanDimmerSum":sum / count.max(1)}));
    }
    serde_json::to_writer_pretty(std::io::stdout().lock(), &samples)?;
    Ok(())
}
