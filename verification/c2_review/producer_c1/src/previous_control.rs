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
        schema_version: 2,
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
        plen_fuel_kg: 0.0,
        exh_fuel_kg: 0.0,
        fuel_emitted_kg: 0.0,
        pmax_pa: None,
        thermal: i.thermal.map(|_| engine::state::ThermalSnap {
            t_wall_c: 20.0,
            t_piston_c: 20.0,
            t_oil_c: 20.0,
            t_cool_c: 20.0,
            ect_filt_c: 20.0,
        }),
    })
}
fn main() {
    let i = engine::healthy_inputs();
    for capacity in [1e307, f64::MAX] {
        let mut q = i.clone();
        let mut p = thermal::ThermalParams::illustrative();
        p.c_wall = capacity;
        q.thermal = Some(p);
        let mut w = WorkState::restore(&cold(&i), &i).unwrap();
        w.therm = Some((thermal::ThermalState::cold(0.), p));
        let rest = WorkState::restore(&w.snapshot(), &q);
        let r = engine::resume(&mut w, &q, 0.001);
        println!(
            "{}",
            serde_json::json!({"case":"finite_actual_thermal","capacity":capacity,"restore_ok":rest.is_ok(),"resume_ok":r.is_ok()})
        );
    }
    let mut q = i.clone();
    q.friction = crank::FrictionModel::ChenFlynn {
        coeffs: crank::ChenFlynnCoeffs {
            c1_pa: 20000.,
            c2_dlless: 0.05,
            c3_pa_s_m: 1000.,
            c4_pa_s2_m2: 100.,
        },
    };
    let mut w = WorkState::restore(&cold(&q), &q).unwrap();
    w.provenance = None;
    engine::resume(&mut w, &q, 0.5).unwrap();
    let mut s = w.snapshot();
    s.state.pmax_pa = None;
    s = EngineSnapshot::capture(&s.state);
    let w2 = WorkState::restore(&s, &q).unwrap();
    println!(
        "{}",
        serde_json::json!({"case":"missing_native_history","provenance":w2.snapshot().provenance})
    );
}
