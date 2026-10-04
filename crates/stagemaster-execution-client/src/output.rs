use crate::{Catalog, Client, State, View};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OutputCatalog {
    pub uncontrolled_fixtures: usize,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputState {
    pub percent: u8,
    pub blackout: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum OutputAction {
    Level { percent: u8 },
    Blackout { enabled: bool },
}
pub(crate) fn catalog(catalog: &Catalog) -> Result<(), String> {
    let declared = catalog.capabilities.iter().any(|c| c == "outputMaster");
    if declared != catalog.output.is_some()
        || catalog.output.as_ref().is_some_and(|o| {
            catalog
                .fixtures
                .as_ref()
                .is_none_or(|f| o.uncontrolled_fixtures > f.len())
        })
    {
        return Err("后台总控能力与灯具范围不一致".into());
    }
    Ok(())
}
pub(crate) fn state(catalog: &Catalog, state: &State) -> Result<(), String> {
    if catalog.output.is_some() != state.output.is_some()
        || state.output.as_ref().is_some_and(|o| o.percent > 100)
    {
        return Err("后台总控状态与能力或数值范围不一致".into());
    }
    Ok(())
}
impl Client {
    pub async fn output(
        &mut self,
        host: &str,
        revision: &str,
        action: OutputAction,
    ) -> Result<View, String> {
        if host != self.transport.discovery.host_id || self.catalog.output.is_none() {
            return Err("后台已更换或未提供输出总控，请重新核对".into());
        }
        if matches!(action, OutputAction::Level { percent } if percent > 100) {
            return Err("后台总亮度须为 0—100% 的整数".into());
        }
        crate::validation::decimal(revision)?;
        self.send(json!({"kind":"submit","expectedRevision":revision,
            "action":{"kind":"output","action":action}}))
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn values() -> (Catalog, State) {
        let catalog = serde_json::from_value(json!({"protocol":2,"execution":"sourceGroup","mode":"softwareOutput",
            "physicalOutput":false,"projectId":"project","layout":"layout","sources":[],"fixtures":[],
            "capabilities":["outputMaster"],"output":{"uncontrolledFixtures":0}})).unwrap();
        let state = serde_json::from_value(
            json!({"boot":"boot","layout":"layout","revision":"0","observedMs":"0",
            "sources":[],"fault":false,"owner":null,"output":{"percent":100,"blackout":false}}),
        )
        .unwrap();
        (catalog, state)
    }
    #[test]
    fn declared_output_requires_complete_bounded_state_and_catalog() {
        let (mut c, mut s) = values();
        assert!(catalog(&c).is_ok());
        assert!(state(&c, &s).is_ok());
        s.output.as_mut().unwrap().percent = 101;
        assert!(state(&c, &s).is_err());
        s.output = None;
        assert!(state(&c, &s).is_err());
        c.output.as_mut().unwrap().uncontrolled_fixtures = 1;
        assert!(catalog(&c).is_err());
        c.output = None;
        assert!(catalog(&c).is_err());
        c.capabilities.clear();
        assert!(catalog(&c).is_ok());
        assert!(state(&c, &s).is_ok());
        s.output = Some(OutputState {
            percent: 0,
            blackout: true,
        });
        assert!(state(&c, &s).is_err());
    }
    #[test]
    fn wire_rejects_extra_fields_fractional_percent_wrong_boolean_or_absent_fields() {
        for invalid in [
            json!({"percent":1.5,"blackout":false}),
            json!({"percent":50}),
            json!({"percent":0,"blackout":"true"}),
            json!({"percent":100,"blackout":false,"source":"fake"}),
        ] {
            assert!(serde_json::from_value::<OutputState>(invalid).is_err());
        }
        for invalid in [
            json!({"kind":"level","percent":50,"blackout":false}),
            json!({"kind":"blackout","enabled":true,"percent":100}),
            json!({"kind":"level","percent":-1}),
        ] {
            assert!(serde_json::from_value::<OutputAction>(invalid).is_err());
        }
    }
}
