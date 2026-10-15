//! Test the owner-authorized metadata-only v1 fixture adaptation.
use engine::{CylState, EngineInputs, EngineSnapshot, EngineState, WorkState};
use serde_json::json;
include!("common.rs");
fn main() {
    let mut i = engine::healthy_inputs();
    i.throttle_area_m2 = 0.05e-4;
    i.idle_bypass_area_m2 = 0.02e-4;
    let mut literal = cold(&i);
    literal.state.schema_version = 1;
    let raw = engine_types::canonical_json(&literal.state).replace(",\"thermal\":null", "");
    literal.state_hash = engine_types::sha256_hex(raw.as_bytes());
    let mut adapted = literal.clone();
    adapted.model = None;
    let genuine: EngineSnapshot = serde_json::from_str(include_str!("../fixtures/genuine-v1-cold.json")).unwrap();
    assert_eq!(literal.state, adapted.state);
    assert_eq!(literal.state_hash, adapted.state_hash);
    assert_eq!(adapted.state, genuine.state);
    assert_eq!(adapted.state_hash, genuine.state_hash);
    assert!(literal.verified() && adapted.verified() && genuine.verified());
    assert!(WorkState::restore(&literal, &i).is_err());
    assert!(WorkState::restore(&adapted, &i).is_ok());
    assert!(WorkState::restore(&genuine, &i).is_ok());
    for (name, snapshot) in [("literal", literal), ("adapted", adapted), ("genuine", genuine)] {
        println!("{}", json!({"case":name,"state_hash":snapshot.state_hash,"verified":snapshot.verified(),"model":snapshot.model,"accepted":WorkState::restore(&snapshot,&i).is_ok()}));
    }
}
