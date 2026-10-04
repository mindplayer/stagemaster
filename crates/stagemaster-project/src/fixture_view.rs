//! Fixture-library and function metadata projections, derived from validated project data.
use crate::view::{FunctionAttributeView, ProfileView};
use crate::{array, text};
use serde_json::Value;

pub(super) fn profile(p: &Value) -> ProfileView {
    ProfileView {
        emitters: p
            .get("emitters")
            .map(|v| serde_json::from_value(v.clone()).expect("validated emitters")),
        positioning: crate::position::model(p).expect("validated model"),
        revision: text(p, "revision").into(),
        manufacturer: text(p, "manufacturer").into(),
        model: text(p, "model").into(),
        mode: text(p, "mode").into(),
        authorable: crate::fixture::supported_keys(
            array(p, "attributes").iter().map(|a| text(a, "key")),
        ) && (p.get("positioning").is_none()
            || array(p, "attributes").iter().any(|a| a["key"] == "pan"))
            && array(p, "attributes").iter().all(|a| {
                a["mix"]
                    == if crate::fixture_emitter::base(text(a, "key")) == "dimmer" {
                        "htp"
                    } else {
                        "ltp"
                    }
            }),
        channels: array(p, "channels")
            .iter()
            .map(|c| crate::ProfileChannel {
                attribute: text(c, "attribute").into(),
                coarse: u16::try_from(c["offsets"][0].as_u64().unwrap_or_default())
                    .expect("validated offset")
                    + 1,
                fine: c["offsets"][1]
                    .as_u64()
                    .map(|n| u16::try_from(n).expect("validated offset") + 1),
                default_value: array(p, "attributes")
                    .iter()
                    .find(|a| a["key"] == c["attribute"])
                    .map(|a| {
                        crate::fixture_value::read_default(&a["default"])
                            .expect("validated default")
                    })
                    .expect("validated attribute"),
                functions: crate::fixture_value::functions(c).expect("validated functions"),
            })
            .collect(),
        id: text(p, "id").into(),
        name: text(p, "name").into(),
        footprint: p["footprint"].as_u64().unwrap_or_default(),
    }
}

pub(super) fn function_attribute(
    profile: &Value,
    attribute: &Value,
) -> Option<FunctionAttributeView> {
    if attribute["valueType"]["kind"] != "function" {
        return None;
    }
    let channel = array(profile, "channels")
        .iter()
        .find(|c| c["attribute"] == attribute["key"])
        .expect("validated channel");
    Some(FunctionAttributeView {
        functions: crate::fixture_value::functions(channel)
            .expect("validated functions")
            .expect("function channel"),
        default: crate::fixture_value::selection(&attribute["default"])
            .expect("validated selection"),
        fine: channel["encoding"] == "u16-be",
    })
}
