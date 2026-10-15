//! Additional independent C2 matrices. Raw observations; historical oracles unchanged.
use engine::{CylState, EngineInputs, EngineSnapshot, EngineState, WorkState};
use serde_json::{json, Value};
include!("common.rs");
fn emit(kind: &str, data: Value) {
    println!("{}", json!({"probe":kind,"data":data}));
}
fn input_set(i: &mut EngineInputs, k: &str, v: f64) {
    match k {
        "afr" => i.control.afr_target = v,
        "eta_v" => i.control.eta_v_assumed = v,
        "spark" => i.control.spark_adv_btdc_deg = v,
        "lhv" => i.fuel.lhv_j_per_kg = v,
        "density" => i.fuel.density_kg_m3 = v,
        "stoich" => i.fuel.stoich_afr = v,
        "charge" => i.charge_volts = v,
        "dwell" => i.dwell_s = v,
        "cutout" => i.starter_cutout_rpm = v,
        "timeout" => i.crank_timeout_s = v,
        "j" => i.inertia.rotating_j_kg_m2 = v,
        "recip" => i.inertia.recip_mass_kg = v,
        "wiebe_a" => i.wiebe_a = v,
        "wiebe_m" => i.wiebe_m = v,
        "duration" => i.wiebe_duration_deg = v,
        "eff" => i.combustion_efficiency = v,
        "dp" => i.regulator_dp_pa = v,
        "throttle" => i.throttle_area_m2 = v,
        "bypass" => i.idle_bypass_area_m2 = v,
        "tailpipe" => i.tailpipe_area_m2 = v,
        "load" => i.accessory_tau_n_m = v,
        "plen_v" => i.plenum_v_m3 = v,
        "exh_v" => i.exhaust_v_m3 = v,
        "dt" => i.dt_s = v,
        "end" => i.t_end_s = v,
        "starter_torque" => i.starter.as_mut().unwrap().0.stall_torque_n_m = v,
        "starter_stall_i" => i.starter.as_mut().unwrap().0.stall_current_a = v,
        "starter_free_i" => i.starter.as_mut().unwrap().0.free_current_a = v,
        "starter_free_w" => i.starter.as_mut().unwrap().0.free_speed_rad_s = v,
        "ratio" => i.starter.as_mut().unwrap().1.ratio = v,
        "soc" => i.starter.as_mut().unwrap().2.soc = v,
        "battery_t" => i.starter.as_mut().unwrap().2.temp_c = v,
        "v_full" => i.starter.as_mut().unwrap().2.v_full = v,
        "v_slope" => i.starter.as_mut().unwrap().2.v_slope = v,
        "v_coeff" => i.starter.as_mut().unwrap().2.v_temp_coeff = v,
        "r0" => i.starter.as_mut().unwrap().2.r0_ohm = v,
        "r_ground" => i.starter.as_mut().unwrap().2.r_ground_ohm = v,
        "injector_q" => i.injector.q_ref_m3_s = v,
        "injector_p" => i.injector.p_ref_pa = v,
        "deadtime" => i.injector.deadtime_s_at_v[3] = v,
        "deadtime_unc" => i.injector.deadtime_unc_s = v,
        "cam_lift" => i.intake_cam.valve_lift_mm = v,
        "cam_duration" => i.intake_cam.adv_duration_crank_deg = v,
        "cam_flank" => i.intake_cam.flank = v,
        "cam_center" => i.intake_cam.centerline_atdc_deg = v,
        _ => panic!("unknown field"),
    }
}
fn matrix() {
    let fields = [
        "afr",
        "eta_v",
        "spark",
        "lhv",
        "density",
        "stoich",
        "charge",
        "dwell",
        "cutout",
        "timeout",
        "j",
        "recip",
        "wiebe_a",
        "wiebe_m",
        "duration",
        "eff",
        "dp",
        "throttle",
        "bypass",
        "tailpipe",
        "load",
        "plen_v",
        "exh_v",
        "dt",
        "end",
        "starter_torque",
        "starter_stall_i",
        "starter_free_i",
        "starter_free_w",
        "ratio",
        "soc",
        "battery_t",
        "v_full",
        "v_slope",
        "v_coeff",
        "r0",
        "r_ground",
        "injector_q",
        "injector_p",
        "deadtime",
        "deadtime_unc",
        "cam_lift",
        "cam_duration",
        "cam_flank",
        "cam_center",
    ];
    for k in fields {
        for (name, v) in [
            ("nan", f64::NAN),
            ("inf", f64::INFINITY),
            ("minus_inf", f64::NEG_INFINITY),
            ("zero", 0.),
            ("negative", -1.),
        ] {
            let valid = engine::healthy_inputs();
            let mut i = valid.clone();
            input_set(&mut i, k, v);
            for (label, end) in [("noop", 0.), ("partial", 0.001)] {
                let mut w = ws(&valid);
                w.m_in_amb_kg = 0.123;
                let before = format!("{w:?}");
                let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    engine::resume(&mut w, &i, end)
                }));
                let rest = std::panic::catch_unwind(|| WorkState::restore(&cold(&valid), &i));
                let mut ri = i.clone();
                if k != "end" {
                    ri.t_end_s = end;
                }
                let run = std::panic::catch_unwind(|| engine::run_engine(&ri));
                emit(
                    "input",
                    json!({"field":k,"value":name,"interval":label,"accepted":r.as_ref().is_ok_and(|q|q.is_ok()),"panic":r.is_err(),"unchanged":before==format!("{w:?}"),"error":r.as_ref().ok().and_then(|q|q.as_ref().err()).map(|e|e.to_string()),"restore_ok":rest.as_ref().is_ok_and(|q|q.is_ok()),"run_ok":run.as_ref().is_ok_and(|q|q.is_ok()),"run_panic":run.is_err()}),
                );
            }
        }
    }
    for (field, value) in [
        ("eta_v", 1.01),
        ("eff", 1.01),
        ("soc", 1.01),
        ("starter_free_i", 350.),
    ] {
        let valid = engine::healthy_inputs();
        let mut i = valid.clone();
        input_set(&mut i, field, value);
        let mut w = ws(&valid);
        let before = format!("{w:?}");
        let r = engine::resume(&mut w, &i, 0.001);
        emit(
            "relational_input",
            json!({"field":field,"value":value,"accepted":r.is_ok(),"unchanged":before==format!("{w:?}")}),
        );
    }
    // Cold battery is legitimate Celsius; injector/cam contrafactuals long enough to engage mechanisms.
    for (case, field, value) in [
        ("negative_injector", "injector_q", -3e-6),
        ("nan_cam", "cam_center", f64::NAN),
        ("cold_battery", "battery_t", -20.),
    ] {
        let mut i = engine::healthy_inputs();
        input_set(&mut i, field, value);
        i.t_end_s = 0.5;
        let r = engine::run_engine(&i);
        emit(
            "active",
            json!({"case":case,"accepted":r.is_ok(),"error":r.as_ref().err().map(|e|e.to_string()),"audit":r.ok().map(|x|json!({"rpm":x.audit.final_rpm,"injected":x.audit.fuel_injected_kg,"heat":x.audit.heat_released_j,"energy":x.audit.energy_closure}))}),
        );
    }
    for case in [
        "cylinder_derived_overflow",
        "node_energy_overflow",
        "thermal_params_mismatch",
        "thermal_presence",
        "counter_nan",
        "history_inf",
        "late_flow",
    ] {
        let mut i = engine::healthy_inputs();
        let mut w = ws(&i);
        match case {
            "cylinder_derived_overflow" => {
                w.cyls[0].m = f64::MIN_POSITIVE;
                w.cyls[0].u = f64::MAX;
            }
            "node_energy_overflow" => {
                w.plen.cv = f64::MAX;
            }
            "thermal_params_mismatch" => {
                let p = thermal::ThermalParams::illustrative();
                i.thermal = Some(p);
                let mut q = p;
                q.c_wall *= 2.;
                w.therm = Some((thermal::ThermalState::cold(20.), q));
            }
            "thermal_presence" => i.thermal = Some(thermal::ThermalParams::illustrative()),
            "counter_nan" => w.w_ind_j = f64::NAN,
            "history_inf" => w.prev_local_deg[1] = f64::INFINITY,
            "late_flow" => {
                i.throttle_area_m2 = 1e4;
                w.plen.m *= 0.5;
            }
            _ => unreachable!(),
        }
        for end in [0., 0.001] {
            let mut a = w.clone();
            let before = format!("{a:?}");
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                engine::resume(&mut a, &i, end)
            }));
            emit(
                "state",
                json!({"case":case,"end":end,"accepted":r.as_ref().is_ok_and(|q|q.is_ok()),"panic":r.is_err(),"unchanged":before==format!("{a:?}"),"error":r.as_ref().ok().and_then(|q|q.as_ref().err()).map(|e|e.to_string())}),
            );
        }
    }
}
fn diagnostics() {
    use diagnostics::mech::{diagnose_low_compression, LeakSplit};
    let full = LeakSplit {
        to_crankcase_01: 0.1,
        to_exhaust_01: 0.7,
        to_coolant_01: 0.2,
    };
    for k in ["comp", "threshold", "pct", "crank", "exhaust", "coolant"] {
        for (name, v) in [
            ("nan", f64::NAN),
            ("inf", f64::INFINITY),
            ("negative", -0.1),
            ("zero", 0.),
            ("above", 101.),
        ] {
            let (mut c, mut t, mut l, mut s) = (500., 900., 30., full);
            match k {
                "comp" => c = v,
                "threshold" => t = v,
                "pct" => l = v,
                "crank" => s.to_crankcase_01 = v,
                "exhaust" => s.to_exhaust_01 = v,
                _ => s.to_coolant_01 = v,
            };
            emit(
                "diagnostic",
                json!({"field":k,"value":name,"verdict":format!("{:?}",diagnose_low_compression(Some(c),t,Some(l),Some(s)))}),
            );
        }
    }
    for delta in [-1.1e-6, -0.9e-6, 0., 0.9e-6, 1.1e-6] {
        let mut s = full;
        s.to_exhaust_01 += delta;
        emit(
            "split_tolerance",
            json!({"delta":delta,"verdict":format!("{:?}",diagnose_low_compression(Some(500.),900.,Some(30.),Some(s)))}),
        );
    }
    for bits in 0..8 {
        emit(
            "missing",
            json!({"mask":bits,"verdict":format!("{:?}",diagnose_low_compression(if bits&1!=0{Some(500.)}else{None},900.,if bits&2!=0{Some(30.)}else{None},if bits&4!=0{Some(full)}else{None}))}),
        );
    }
}
fn thermal() {
    use thermal::{ThermalParams as P, ThermalState as S};
    for field in [
        "c_wall",
        "c_piston",
        "c_oil",
        "c_cool",
        "wall_frac",
        "ha_pw",
        "ha_wc",
        "ha_wo",
        "ha_wa",
        "ha_oa",
        "ha_rad",
        "lo",
        "hi",
        "tau",
    ] {
        for (name, v) in [
            ("zero", 0.),
            ("negative", -1.),
            ("nan", f64::NAN),
            ("inf", f64::INFINITY),
            ("tiny", 1e-308),
            ("large", 1e306),
            ("max", f64::MAX),
        ] {
            let mut p = P::illustrative();
            match field {
                "c_wall" => p.c_wall = v,
                "c_piston" => p.c_piston = v,
                "c_oil" => p.c_oil = v,
                "c_cool" => p.c_cool = v,
                "wall_frac" => p.wall_frac = v,
                "ha_pw" => p.ha_piston_wall = v,
                "ha_wc" => p.ha_wall_cool = v,
                "ha_wo" => p.ha_wall_oil = v,
                "ha_wa" => p.ha_wall_amb = v,
                "ha_oa" => p.ha_oil_amb = v,
                "ha_rad" => p.ha_rad_full = v,
                "lo" => p.tstat_lo_c = v,
                "hi" => p.tstat_hi_c = v,
                _ => p.ect_tau_s = v,
            };
            let mut s = S::cold(20.);
            let before = format!("{s:?}");
            let u = s.stored_j_checked(&p);
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                s.step(1e-4, 0., 0., 20., &p)
            }));
            let valid = engine::healthy_inputs();
            let mut i = valid.clone();
            i.thermal = Some(p);
            i.t_end_s = 0.;
            let mut w = ws(&valid);
            w.therm = Some((thermal::ThermalState::cold(20.), p));
            let snap = w.snapshot();
            let before_work = format!("{w:?}");
            let restore_ok = WorkState::restore(&snap, &i).is_ok();
            let run_ok = engine::run_engine(&i).is_ok();
            let resume_ok = engine::resume(&mut w, &i, 0.001).is_ok();
            let work_unchanged = before_work == format!("{w:?}");
            emit(
                "thermal_param",
                json!({"field":field,"value":name,"restore_ok":restore_ok,"run_ok":run_ok,"resume_ok":resume_ok,"work_unchanged":work_unchanged,"params_valid":p.validated().is_ok(),"stored_finite":u.is_some(),"accepted":r.as_ref().is_ok_and(|q|q.is_ok()),"panic":r.is_err(),"unchanged":before==format!("{s:?}")}),
            );
        }
    }
    for capacity in [1e-308, 1e300] {
        let mut p = P::illustrative();
        p.c_wall = capacity;
        let mut s = S::cold(0.);
        let r = s.step(1e-4, 0., 0., 0., &p);
        emit(
            "thermal_extreme_valid",
            json!({"capacity":capacity,"accepted":r.is_ok(),"stored":s.stored_j_checked(&p)}),
        );
    }
    for t in [
        -273.15,
        -300.,
        -272.,
        -20.,
        0.,
        100.,
        f64::MAX,
        f64::NAN,
        f64::INFINITY,
    ] {
        let p = P::illustrative();
        let mut s = S::cold(t);
        let before = format!("{s:?}");
        let r = s.step(1e-4, 0., 0., 20., &p);
        emit(
            "thermal_temperature",
            json!({"t":t,"accepted":r.is_ok(),"unchanged":before==format!("{s:?}")}),
        );
    }
}
fn snapshots() {
    let out = std::env::var("D16_C2_FIXTURES").unwrap();
    for entry in std::fs::read_dir(&out).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let original: EngineSnapshot =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let i = engine::healthy_inputs();
        let r = WorkState::restore(&original, &i);
        let mut row = json!({"fixture":path.file_name().unwrap().to_str().unwrap(),"hash":original.state_hash,"verified":original.verified(),"structure":original.state.schema_version,"model":original.model,"provenance_before":original.provenance,"restore_ok":r.is_ok(),"error":r.as_ref().err().map(|e|e.to_string())});
        if let Ok(mut w) = r {
            let cap = w.snapshot();
            let rt: EngineSnapshot =
                serde_json::from_str(&serde_json::to_string(&cap).unwrap()).unwrap();
            let again = WorkState::restore(&rt, &i);
            row["provenance_after"] = json!(cap.provenance);
            row["recapture_hash"] = json!(cap.state_hash);
            row["recapture_model"] = json!(cap.model);
            row["roundtrip_verified"] = json!(rt.verified());
            row["reread_provenance"] =
                json!(again.as_ref().ok().and_then(|a| a.snapshot().provenance));
            let t = w.time_s + 0.01;
            row["continuation_ok"] = json!(engine::resume(&mut w, &i, t).is_ok());
        }
        emit("snapshot_producer", row);
        for case in [
            "unknown_model",
            "mismatch_model",
            "missing_pmax",
            "forged_native",
        ] {
            let mut s = original.clone();
            match case {
                "unknown_model" => {
                    s.model = Some(engine::state::ModelIdentity {
                        schema_version: s.state.schema_version,
                        combustion_accounting: 999,
                    })
                }
                "mismatch_model" => {
                    s.model = Some(engine::state::ModelIdentity {
                        schema_version: 999,
                        combustion_accounting: 1,
                    })
                }
                "missing_pmax" => {
                    s.state.pmax_pa = None;
                    let p = s.provenance.clone();
                    s = EngineSnapshot::capture_with(&s.state, p);
                }
                _ => {
                    s.model = Some(engine::state::ModelIdentity {
                        schema_version: s.state.schema_version,
                        combustion_accounting: 1,
                    });
                    s.provenance = None;
                }
            }
            let r = WorkState::restore(&s, &i);
            emit(
                "semantic_negative",
                json!({"fixture":path.file_name().unwrap().to_str().unwrap(),"case":case,"hash_valid":s.verified(),"accepted":r.is_ok(),"provenance_after":r.as_ref().ok().and_then(|w|w.snapshot().provenance),"error":r.err().map(|e|e.to_string())}),
            );
        }
    }
}
fn focused() {
    let i = engine::healthy_inputs();
    let mut w = ws(&i);
    w.cyls[0].m = f64::MIN_POSITIVE;
    w.cyls[0].u = f64::MAX;
    let t = w.cyls[0].u / (w.cyls[0].m * i.mix.cv());
    let snap = w.snapshot();
    let restore = WorkState::restore(&snap, &i);
    let before = format!("{w:?}");
    let r = engine::resume(&mut w, &i, 0.);
    emit(
        "derived_cylinder",
        json!({"temperature_finite":t.is_finite(),"hash_valid":snap.verified(),"restore_ok":restore.is_ok(),"noop_ok":r.is_ok(),"unchanged":before==format!("{w:?}")}),
    );
    // Finite actual energy versus overflow at the unrelated factory temperature 20 C.
    for capacity in [1e307, f64::MAX] {
        let mut p = thermal::ThermalParams::illustrative();
        p.c_wall = capacity;
        let mut q = i.clone();
        q.thermal = Some(p);
        let mut w = ws(&i);
        w.therm = Some((thermal::ThermalState::cold(0.), p));
        let actual = w.therm.as_ref().unwrap().0.stored_j_checked(&p);
        let before = format!("{w:?}");
        let r = engine::resume(&mut w, &q, 0.001);
        let restore = WorkState::restore(&w.snapshot(), &q);
        let mut st = thermal::ThermalState::cold(0.);
        let step = st.step(1e-4, 0., 0., 20., &p);
        emit(
            "finite_actual_thermal",
            json!({"capacity":capacity,"actual_energy":actual,"params_valid":p.validated().is_ok(),"standalone_step_ok":step.is_ok(),"standalone_stored":st.stored_j_checked(&p),"resume_ok":r.is_ok(),"error":r.err().map(|e|e.to_string()),"restore_ok":restore.is_ok(),"unchanged":before==format!("{w:?}")}),
        );
    }
    // All physical quantities remain unchanged; only loss of required native pressure history.
    let mut q = i.clone();
    q.friction = crank::FrictionModel::ChenFlynn {
        coeffs: crank::ChenFlynnCoeffs {
            c1_pa: 20000.,
            c2_dlless: 0.05,
            c3_pa_s_m: 1000.,
            c4_pa_s2_m2: 100.,
        },
    };
    let mut w = ws(&q);
    engine::resume(&mut w, &q, 0.5).unwrap();
    let native = w.snapshot();
    let mut stripped = native.clone();
    stripped.state.pmax_pa = None;
    stripped = EngineSnapshot::capture(&stripped.state);
    let mut a = WorkState::restore(&native, &q).unwrap();
    let mut b = WorkState::restore(&stripped, &q).unwrap();
    let before_p = (a.pmax_pa, b.pmax_pa);
    engine::resume(&mut a, &q, 0.55).unwrap();
    engine::resume(&mut b, &q, 0.55).unwrap();
    emit(
        "native_missing_history",
        json!({"hash_valid":stripped.verified(),"model":stripped.model,"provenance":b.snapshot().provenance,"pressure_original":before_p.0,"pressure_reconstructed":before_p.1,"same_trajectory":a.snapshot().state==b.snapshot().state,"omega_original":a.omega_rad_s,"omega_reconstructed":b.omega_rad_s}),
    );
}
fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("focused") => focused(),
        Some("matrix") => matrix(),
        Some("thermal") => thermal(),
        Some("diagnostics") => diagnostics(),
        Some("snapshots") => snapshots(),
        _ => panic!("mode required"),
    }
}
