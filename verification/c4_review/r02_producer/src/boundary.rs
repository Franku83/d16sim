use engine::{CylState, EngineInputs, EngineSnapshot, EngineState, WorkState};
use serde_json::{json,Value};
include!("common.rs");
fn emit(id:&str,data:Value){println!("{}",json!({"probe":id,"data":data}));}
fn boundary(){
 for case in ["control_afr_nan","control_afr_zero","control_eta_nan","spark_adv_nan","fuel_lhv_nan","fuel_density_negative","charge_nan","dwell_nan","starter_cut_nan","starter_bad","inertia_nan","inertia_zero","thermal_capacity_zero","thermal_capacity_overflow","thermal_wall_nan","wiebe_duration_inf","friction_overflow"]{
  let valid=engine::healthy_inputs();let mut i=valid.clone();let mut w=ws(&valid);w.m_in_amb_kg=0.12;
  match case{
   "control_afr_nan"=>i.control.afr_target=f64::NAN,"control_afr_zero"=>i.control.afr_target=0.,"control_eta_nan"=>i.control.eta_v_assumed=f64::NAN,"spark_adv_nan"=>i.control.spark_adv_btdc_deg=f64::NAN,
   "fuel_lhv_nan"=>i.fuel.lhv_j_per_kg=f64::NAN,"fuel_density_negative"=>i.fuel.density_kg_m3=-1.,"charge_nan"=>i.charge_volts=f64::NAN,"dwell_nan"=>i.dwell_s=f64::NAN,"starter_cut_nan"=>i.starter_cutout_rpm=f64::NAN,
   "starter_bad"=>i.starter.as_mut().unwrap().0.stall_torque_n_m=f64::NAN,
   "inertia_nan"=>i.inertia.rotating_j_kg_m2=f64::NAN,"inertia_zero"=>{i.inertia.rotating_j_kg_m2=0.;i.inertia.recip_mass_kg=0.;},
   "thermal_capacity_zero"=>{let mut p=thermal::ThermalParams::illustrative();p.c_wall=0.;i.thermal=Some(p);},
   "thermal_capacity_overflow"=>{let mut p=thermal::ThermalParams::illustrative();p.c_wall=f64::MAX;i.thermal=Some(p);},
   "thermal_wall_nan"=>{let mut p=thermal::ThermalParams::illustrative();p.wall_frac=f64::NAN;i.thermal=Some(p);},
   "wiebe_duration_inf"=>i.wiebe_duration_deg=f64::INFINITY,
   _=>i.friction=crank::FrictionModel::Viscous{c_n_m_s:f64::MAX},
  }
  let before=format!("{w:?}");let snap=cold(&valid);
  let restore=std::panic::catch_unwind(||WorkState::restore(&snap,&i));
  let run=std::panic::catch_unwind(||{let mut q=i.clone();q.t_end_s=0.001;engine::run_engine(&q)});
  let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||engine::resume(&mut w,&i,0.001)));
  emit("extended_engine_config",json!({"case":case,"restore_panicked":restore.is_err(),"restore_accepted":restore.as_ref().is_ok_and(|r|r.is_ok()),"run_panicked":run.is_err(),"run_accepted":run.as_ref().is_ok_and(|r|r.is_ok()),"run_energy_finite":run.as_ref().ok().and_then(|r|r.as_ref().ok()).map(|r|r.audit.energy_closure.is_finite()),"resume_panicked":result.is_err(),"resume_accepted":result.as_ref().is_ok_and(|r|r.is_ok()),"unchanged":before==format!("{w:?}"),"resume_error":result.as_ref().ok().and_then(|r|r.as_ref().err()).map(|e|e.to_string()),"omega":w.omega_rad_s,"state_finite":w.omega_rad_s.is_finite()&&w.theta_rad.is_finite()&&w.w_load_j.is_finite()}));
 }
 for case in ["node_v_zero","node_cv_nan","node_v_nan","thermal_presence_mismatch","thermal_stored_overflow","thermal_params_bad"]{
  let mut i=engine::healthy_inputs();let mut w=ws(&i);
  match case{"node_v_zero"=>w.plen.v=0.,"node_cv_nan"=>w.plen.cv=f64::NAN,"node_v_nan"=>w.plen.v=f64::NAN,"thermal_presence_mismatch"=>{i.thermal=Some(thermal::ThermalParams::illustrative());},"thermal_stored_overflow"=>{let mut p=thermal::ThermalParams::illustrative();p.c_wall=f64::MAX;w.therm=Some((thermal::ThermalState::cold(20.),p));i.thermal=Some(p);},_=>{let mut p=thermal::ThermalParams::illustrative();p.c_wall=0.;w.therm=Some((thermal::ThermalState::cold(20.),p));i.thermal=Some(p);}}
  w.m_in_amb_kg=0.12;let pre=format!("{w:?}");let r=engine::resume(&mut w,&i,0.);
  emit("extended_work_state",json!({"case":case,"accepted":r.is_ok(),"unchanged":pre==format!("{w:?}"),"m_initial":w.m_sys_0_kg,"u_initial":w.u_sys_0_j}));
 }
 for case in ["capacity_zero", "wall_frac_nan", "capacity_overflow"] {
 let mut i=engine::healthy_inputs(); let mut p=thermal::ThermalParams::illustrative();match case{"capacity_zero"=>p.c_wall=0.,"wall_frac_nan"=>p.wall_frac=f64::NAN,_=>p.c_wall=f64::MAX}; i.thermal=Some(p);let snap=cold(&i);let r=WorkState::restore(&snap,&i);emit("thermal_restore_config",json!({"case":case,"hash_valid":snap.verified(),"accepted":r.is_ok(),"stored_finite":r.as_ref().ok().map(|w|w.th_u0_j.is_finite())}));
 }
}

fn main(){boundary();}
