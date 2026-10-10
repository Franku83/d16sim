//! Historical type and producer from Git 2541266, not a reconstructed v1 DTO.
use engine::{CylState, EngineInputs, EngineSnapshot, EngineState, WorkState};
fn cold(i: &EngineInputs) -> EngineSnapshot {
    use kinematics::phases::{crank_angle_for_piston, CylinderId};
    let ids = [
        CylinderId::Cyl1,
        CylinderId::Cyl2,
        CylinderId::Cyl3,
        CylinderId::Cyl4,
    ];
    let locals = ids.map(|id| kinematics::cylinder_local_angle(0.0, id));
    let cyls = locals.map(|a| {
        let v = i.geom.volume(crank_angle_for_piston(a).to_radians());
        let m = 101325.0 * v / (i.mix.r() * 293.15);
        CylState {
            m_kg: m,
            u_j: m * i.mix.cv() * 293.15,
            fresh_frac: 1.0,
            fuel_kg: 0.0,
            burn_latched_kg: 0.0,
            burn_accounted: true,
        }
    });
    EngineSnapshot::capture(&EngineState {
        schema_version: 1,
        time_s: 0.0,
        theta_global_rad: 0.0,
        omega_rad_s: 0.0,
        prev_local_deg: locals,
        cyls,
        plen_m_kg: 101325.0 * i.plenum_v_m3 / (i.mix.r() * 293.15),
        plen_t_k: 293.15,
        plen_fresh: 1.0,
        exh_m_kg: 101325.0 * i.exhaust_v_m3 / (i.mix.r() * 293.15),
        exh_t_k: 293.15,
        exh_fresh: 0.0,
        fuel_injected_kg: 0.0,
        fuel_burned_kg: 0.0,
        heat_released_j: 0.0,
        work_ind_j: 0.0,
        work_fric_j: 0.0,
        work_starter_j: 0.0,
        work_load_j: 0.0,
    })
}

fn main() {
    let i = engine::healthy_inputs();
    let cold = cold(&i);
    let mut w = WorkState::restore(&cold, &i).unwrap();
    engine::resume(&mut w, &i, 0.5).unwrap();
    let hot = w.snapshot();
    println!("{}", serde_json::to_string(&hot).unwrap());
}
