//! Non-production C4 preflight. Compare real v1 data and the unchanged retagged fixture.
use engine::{CylState, EngineInputs, EngineSnapshot, EngineState, WorkState};
use serde_json::json;
include!("common.rs");
fn main() {
    let mut i=engine::healthy_inputs();
    i.throttle_area_m2=0.05e-4;i.idle_bypass_area_m2=0.02e-4;
    let mut retagged=cold(&i);
    retagged.state.schema_version=1;
    let raw=engine_types::canonical_json(&retagged.state).replace(",\"thermal\":null","");
    retagged.state_hash=engine_types::sha256_hex(raw.as_bytes());
    let genuine:EngineSnapshot=serde_json::from_str(include_str!("../fixtures/genuine-v1-cold.json")).unwrap();
    let coherent=cold(&i);
    let mut mismatched=coherent.clone();
    mismatched.model.as_mut().unwrap().schema_version=1;
    for (name,s) in [("retagged_original_fixture",retagged.clone()),("genuine_v1",genuine.clone()),("coherent_v2",coherent),("incoherent_v2",mismatched)] {
        let pair_consistent=s.model.as_ref().is_none_or(|m|m.schema_version==s.state.schema_version);
        println!("{}",json!({"case":name,"state_schema":s.state.schema_version,"model":s.model,"state_hash":s.state_hash,"hash_verified":s.verified(),"c3_restore_accepted":WorkState::restore(&s,&i).is_ok(),"strict_pair_consistent":pair_consistent}));
    }
    println!("{}",json!({"case":"physical_equivalence","genuine_and_retagged_state_equal":genuine.state==retagged.state,"genuine_and_retagged_hash_equal":genuine.state_hash==retagged.state_hash,"difference_in_wrapper_model_only":genuine.model!=retagged.model&&genuine.provenance==retagged.provenance}));
}
