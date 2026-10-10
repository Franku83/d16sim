use engine::{CylState,EngineState,EngineSnapshot,EngineInputs,WorkState};
include!("common.rs");
fn main(){let i=engine::healthy_inputs();let mut w=ws(&i);engine::resume(&mut w,&i,0.35955).unwrap();println!("{}",serde_json::to_string(&w.snapshot()).unwrap());}
