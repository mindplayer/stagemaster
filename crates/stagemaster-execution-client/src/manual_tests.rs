use crate::{Catalog, State, manual_validation as validate};
use serde_json::{Value, json};
fn fixture() -> (Value, Value) {
    (
        json!({"protocol":2,"execution":"sourceGroup","mode":"softwareOutput","physicalOutput":false,"projectId":"project","layout":"layout",
        "capabilities":["manualOwnership","semanticManualPatch"],"limits":{"manualChanges":512,"requestBytes":8192},
        "fixtures":[{"id":"00000000-0000-4000-8000-000000000001","name":"灯","profileName":"模式","universe":1,"address":1,"attributes":[{"key":"dimmer","label":"亮度","defaultValue":0,"function":null}]}],
        "sources":[{"id":"manual","name":"手动","priority":0,"selection":{"kind":"manual"},"steps":[]}]}),
        json!({"boot":"boot","layout":"layout","revision":"1","observedMs":"1","owner":null,"fault":false,"sources":[{"id":"manual","level":0,"status":null,"step":null,"held":[{"fixtureId":"00000000-0000-4000-8000-000000000001","attribute":"dimmer"}]}]}),
    )
}
#[test]
fn declared_manual_catalog_and_ownership_require_complete_consistent_bounded_data() {
    let (raw, state) = fixture();
    let catalog: Catalog = serde_json::from_value(raw.clone()).unwrap();
    validate::catalog(&catalog).unwrap();
    validate::state(&catalog, &serde_json::from_value(state.clone()).unwrap()).unwrap();
    for (path, value) in [
        ("/fixtures", Value::Null),
        ("/limits", Value::Null),
        ("/limits/manualChanges", json!(513)),
        ("/limits/requestBytes", json!(8193)),
        ("/fixtures/0/id", json!("missing")),
        ("/fixtures/0/attributes/0/key", json!("")),
    ] {
        let mut bad = raw.clone();
        *bad.pointer_mut(path).unwrap() = value;
        assert!(
            validate::catalog(&serde_json::from_value(bad).unwrap()).is_err(),
            "{path}"
        );
    }
    for changes in [
        Value::Null,
        json!([{"fixtureId":"missing","attribute":"dimmer"}]),
        json!([{"fixtureId":"00000000-0000-4000-8000-000000000001","attribute":"missing"}]),
        json!([
            state["sources"][0]["held"][0],
            state["sources"][0]["held"][0]
        ]),
    ] {
        let mut bad = state.clone();
        bad["sources"][0]["held"] = changes;
        assert!(validate::state(&catalog, &serde_json::from_value(bad).unwrap()).is_err());
    }
    let mut legacy = catalog;
    legacy.capabilities.clear();
    assert!(validate::state(&legacy, &serde_json::from_value(state.clone()).unwrap()).is_err());
    let mut state = state;
    state["sources"][0].as_object_mut().unwrap().remove("held");
    validate::state(&legacy, &serde_json::from_value::<State>(state).unwrap()).unwrap();
}
