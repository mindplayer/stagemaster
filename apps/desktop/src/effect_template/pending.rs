use serde::Serialize;
use stagemaster_project::{EffectTemplateReview, EffectTemplateReviewView};
use std::{
    sync::Mutex,
    time::{Duration, Instant},
};
const TTL: Duration = Duration::from_mins(5);
#[derive(Default)]
pub(crate) struct Service {
    pub(super) gate: Mutex<()>,
    pending: Mutex<Option<Pending>>,
}
struct Pending {
    generation: u32,
    token: String,
    created: Instant,
    review: EffectTemplateReview,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Imported {
    generation: u32,
    token: String,
    file_name: String,
    review: EffectTemplateReviewView,
}
impl Service {
    pub(super) fn clear(&self) -> Result<(), String> {
        *self.pending.lock().map_err(|_| "灯效模板会话发生错误")? = None;
        Ok(())
    }
    pub(super) fn prepare(
        &self,
        generation: u32,
        file_name: String,
        review: EffectTemplateReview,
    ) -> Result<Imported, String> {
        let token = uuid::Uuid::new_v4().to_string();
        let response = Imported {
            generation,
            token: token.clone(),
            file_name,
            review: review.view().clone(),
        };
        *self.pending.lock().map_err(|_| "灯效模板会话发生错误")? = Some(Pending {
            generation,
            token,
            created: Instant::now(),
            review,
        });
        Ok(response)
    }
    // Never waits on gate/recovery/session while holding the cache mutex.
    pub(crate) fn take(
        &self,
        generation: u32,
        token: &str,
    ) -> Result<EffectTemplateReview, String> {
        let mut pending = self.pending.lock().map_err(|_| "灯效模板会话发生错误")?;
        let current = pending.as_ref().ok_or("请重新导入并检查灯效模板")?;
        if current.created.elapsed() >= TTL {
            *pending = None;
            return Err("灯效模板检查已过期，请重新导入".into());
        }
        if current.generation != generation || current.token != token {
            return Err("灯效模板检查上下文已变化，请重新导入".into());
        }
        Ok(pending.take().expect("checked pending").review)
    }
    pub(crate) fn cancel(&self, token: &str) -> Result<(), String> {
        let mut pending = self.pending.lock().map_err(|_| "灯效模板会话发生错误")?;
        if pending.as_ref().is_some_and(|p| p.token == token) {
            *pending = None;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use stagemaster_project::{Document, EditCommand, EffectTemplateFile};
    fn review() -> EffectTemplateReview {
        let mut doc = Document::new("审阅").unwrap();
        let v = doc.view();
        doc.edit(EditCommand::AddFixture {
            name: "灯".into(),
            profile_id: v.profiles[0].id.clone(),
            domain_id: v.domains[0].id.clone(),
            universe: 1,
            address: 1,
        })
        .unwrap();
        doc.edit(EditCommand::AddScene {
            name: "场景".into(),
        })
        .unwrap();
        let v = doc.view();
        let file = EffectTemplateFile::create(
            serde_json::from_value(json!({
                "name": "模板",
                "recipe": {
                    "kind": "intensity-wave", "waveform": "smooth",
                    "low": 0, "high": 65535, "dutyPercent": 50
                },
                "timing": {
                    "periodMs": 2000, "phaseDegrees": 0,
                    "spreadDegrees": 0, "reverseOrder": false
                }
            }))
            .unwrap(),
        )
        .unwrap();
        doc.review_effect_template(&file, &v.scenes[0].id, &[v.fixtures[0].id.clone()])
            .unwrap()
    }
    #[test]
    fn only_exact_context_consumes_review_and_cancellation_cannot_clear_a_newer_ticket() {
        let service = Service::default();
        let a = service.prepare(2, "a".into(), review()).unwrap();
        let b = service.prepare(3, "b".into(), review()).unwrap();
        service.cancel(&a.token).unwrap();
        assert!(service.take(2, &b.token).is_err());
        assert!(service.take(3, &a.token).is_err());
        assert!(service.take(3, &b.token).is_ok());
        assert!(service.take(3, &b.token).is_err());
        let c = service.prepare(4, "c".into(), review()).unwrap();
        service.cancel(&c.token).unwrap();
        assert!(service.take(4, &c.token).is_err());
    }
    #[test]
    fn expiration_and_a_cancelled_new_import_release_the_single_cached_candidate() {
        let service = Service::default();
        let a = service.prepare(2, "a".into(), review()).unwrap();
        service.pending.lock().unwrap().as_mut().unwrap().created =
            Instant::now().checked_sub(TTL).unwrap();
        assert!(service.take(2, &a.token).err().unwrap().contains("过期"));
        assert!(service.pending.lock().unwrap().is_none());
        let b = service.prepare(3, "b".into(), review()).unwrap();
        service.clear().unwrap();
        assert!(service.take(3, &b.token).is_err());
    }
}
