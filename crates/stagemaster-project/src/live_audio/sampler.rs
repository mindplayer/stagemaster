use super::LiveSourceBudget;
use crate::AudioSegmentPlan;
use stagemaster_playback::{HeldScene, SceneCrossfade};

pub(super) enum Sampler {
    Single(HeldScene),
    Crossfade(SceneCrossfade),
}
impl Sampler {
    pub fn prepare(
        plan: AudioSegmentPlan,
        budget: &mut LiveSourceBudget,
        owned: &mut [bool],
    ) -> Result<Self, String> {
        let mut charge = |plan: &stagemaster_playback::Plan| {
            budget.add(LiveSourceBudget::from_plan(plan))?;
            for effect in plan.effects().iter().flatten() {
                owned[effect.index] = true;
            }
            Ok::<_, String>(())
        };
        Ok(match plan {
            AudioSegmentPlan::Single(plan) => {
                charge(&plan)?;
                Self::Single(HeldScene::new(plan)?)
            }
            AudioSegmentPlan::Crossfade {
                source,
                target,
                timing,
            } => {
                charge(&source)?;
                charge(&target)?;
                Self::Crossfade(SceneCrossfade::new(source, target, timing)?)
            }
        })
    }
    pub fn sample(&mut self, position: u64) -> Result<(), String> {
        match self {
            Self::Single(p) => p.sample(position),
            Self::Crossfade(p) => p.sample(position),
        }
        .map(|_| ())
    }
    pub fn values(&self) -> &[u16] {
        match self {
            Self::Single(p) => p.values(),
            Self::Crossfade(p) => p.values(),
        }
    }
}
