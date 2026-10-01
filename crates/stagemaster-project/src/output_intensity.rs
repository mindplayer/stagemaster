//! Conservative recognition of numeric intensity; never scale function/control channels.
use crate::{array, text};
use serde_json::Value;

pub(crate) fn keys(profile: &Value) -> Vec<&str> {
    let numeric = |key: &str| {
        array(profile, "attributes")
            .iter()
            .any(|a| a["key"] == key && a["valueType"]["kind"] == "normalized")
            && array(profile, "channels")
                .iter()
                .any(|c| c["attribute"] == key && c.get("functions").is_none())
    };
    if array(profile, "attributes")
        .iter()
        .any(|a| a["key"] == "dimmer")
    {
        return if numeric("dimmer") {
            vec!["dimmer"]
        } else {
            vec![]
        };
    }
    if ["red", "green", "blue"].iter().all(|key| numeric(key)) {
        vec!["red", "green", "blue"]
    } else {
        vec![]
    }
}

impl crate::Document {
    /// Fixtures without a recognized numeric dimmer or complete numeric RGB set.
    #[must_use]
    pub fn uncontrolled_intensity_fixtures(&self) -> usize {
        let root = &self.root;
        let lighting = &root["lighting"];
        array(lighting, "fixtures")
            .iter()
            .filter(|fixture| {
                array(lighting, "profiles")
                    .iter()
                    .find(|profile| text(profile, "id") == text(fixture, "profileId"))
                    .is_none_or(|profile| keys(profile).is_empty())
            })
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn function_dimmer_is_never_scaled_or_bypassed_by_rgb_fallback() {
        let mut profile = json!({"attributes":[
            {"key":"dimmer","valueType":{"kind":"function"}},
            {"key":"red","valueType":{"kind":"normalized"}},
            {"key":"green","valueType":{"kind":"normalized"}},
            {"key":"blue","valueType":{"kind":"normalized"}}],
            "channels":[{"attribute":"dimmer","functions":[{}]}, {"attribute":"red"},{"attribute":"green"},{"attribute":"blue"}]});
        assert!(keys(&profile).is_empty());
        profile["attributes"][0]["valueType"]["kind"] = json!("normalized");
        assert!(keys(&profile).is_empty());
        profile["channels"][0]
            .as_object_mut()
            .unwrap()
            .remove("functions");
        assert_eq!(keys(&profile), vec!["dimmer"]);
    }
}
