//! Independent C3 root-cause controls. No production writes, no historical oracle edits.
use engine::{CylState, EngineInputs, EngineSnapshot, EngineState, WorkState};
use serde_json::{json, Value};
include!("common.rs");
fn emit(probe: &str, data: Value) {
    println!("{}", json!({"probe":probe,"data":data}));
}
fn fields() {
    for field in [
        "q_ref",
        "p_ref",
        "unc",
        "dead0",
        "dead1",
        "dead2",
        "dead3",
        "dead4",
        "in_lift",
        "in_adv",
        "in_at050",
        "in_flank",
        "in_center",
        "in_lsa",
        "ex_lift",
        "ex_adv",
        "ex_at050",
        "ex_flank",
        "ex_center",
        "ex_lsa",
    ] {
        for (value, v) in [
            ("nan", f64::NAN),
            ("inf", f64::INFINITY),
            ("minus_inf", f64::NEG_INFINITY),
            ("zero", 0.),
            ("negative", -1.),
            ("positive", 1.),
        ] {
            let valid = engine::healthy_inputs();
            let mut i = valid.clone();
            match field {
                "q_ref" => i.injector.q_ref_m3_s = if value == "positive" { 3e-6 } else { v },
                "p_ref" => i.injector.p_ref_pa = if value == "positive" { 250000. } else { v },
                "unc" => i.injector.deadtime_unc_s = if value == "positive" { 0.0001 } else { v },
                k if k.starts_with("dead") => {
                    i.injector.deadtime_s_at_v[k[4..].parse::<usize>().unwrap()] =
                        if value == "positive" { 0.001 } else { v }
                }
                k => {
                    let c = if k.starts_with("in_") {
                        &mut i.intake_cam
                    } else {
                        &mut i.exhaust_cam
                    };
                    match &k[3..] {
                        "lift" => c.valve_lift_mm = v,
                        "adv" => {
                            c.adv_duration_crank_deg = if value == "positive" { 220. } else { v };
                            if value == "positive" {
                                c.at050_duration_crank_deg = 180.;
                            }
                        }
                        "at050" => c.at050_duration_crank_deg = v,
                        "flank" => c.flank = v,
                        "center" => c.centerline_atdc_deg = v,
                        _ => c.lsa_deg = v,
                    }
                }
            }
            let zero_ok = field == "unc" || field.starts_with("dead") || field.ends_with("center");
            let negative_ok = field.ends_with("center");
            let expected_valid = value == "positive"
                || (value == "zero" && zero_ok)
                || (value == "negative" && negative_ok);
            for end in [0., 0.002] {
                let mut w = ws(&valid);
                w.m_in_amb_kg = 0.123;
                let before = format!("{w:?}");
                let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    engine::resume(&mut w, &i, end)
                }));
                let a = std::panic::catch_unwind(|| WorkState::restore(&cold(&valid), &i));
                let mut q = i.clone();
                q.t_end_s = end;
                let b = std::panic::catch_unwind(|| engine::run_engine(&q));
                emit(
                    "parameter",
                    json!({"field":field,"value":value,"end":end,"expected_valid":expected_valid,"resume_ok":r.as_ref().is_ok_and(|x|x.is_ok()),"restore_ok":a.as_ref().is_ok_and(|x|x.is_ok()),"run_ok":b.as_ref().is_ok_and(|x|x.is_ok()),"panic":r.is_err()||a.is_err()||b.is_err(),"unchanged":before==format!("{w:?}")}),
                );
            }
        }
    }
    for kind in ["intake", "exhaust"] {
        let valid = engine::healthy_inputs();
        let mut i = valid.clone();
        let c = if kind == "intake" {
            &mut i.intake_cam
        } else {
            &mut i.exhaust_cam
        };
        c.at050_duration_crank_deg = c.adv_duration_crank_deg + 1.;
        let mut w = ws(&valid);
        let before = format!("{w:?}");
        let r = engine::resume(&mut w, &i, 0.);
        emit(
            "cam_relation",
            json!({"kind":kind,"resume_rejected":r.is_err(),"restore_rejected":WorkState::restore(&cold(&valid),&i).is_err(),"unchanged":before==format!("{w:?}")}),
        );
    }
}
fn thermo() {
    let base = engine::healthy_inputs();
    for node in ["plenum", "exhaust"] {
        for case in [
            "nominal",
            "pressure_overflow",
            "density_overflow",
            "pressure_underflow",
            "pressure_finite_extreme",
        ] {
            let mut i = base.clone();
            let mut w = ws(&base);
            let n = if node == "plenum" {
                &mut w.plen
            } else {
                &mut w.exh
            };
            match case {
                "pressure_overflow" => {
                    n.m = 1e-5;
                    n.t = 1e100;
                    n.v = 1e-214;
                }
                "density_overflow" => {
                    n.m = 1.;
                    n.t = 1.;
                    n.v = 1e-320;
                }
                "pressure_underflow" => {
                    n.m = 1e-200;
                    n.t = 1e-200;
                    n.v = 1e-5;
                }
                "pressure_finite_extreme" => {
                    n.m = 1e-5;
                    n.t = 1e100;
                    n.v = 1e-200;
                }
                _ => {}
            }
            if node == "plenum" {
                i.plenum_v_m3 = n.v;
            } else {
                i.exhaust_v_m3 = n.v;
            }
            let p = n.m * i.mix.r() * n.t / n.v;
            let rho = n.m / n.v;
            let u = n.m * n.cv * n.t;
            let snap = w.snapshot();
            let rest = WorkState::restore(&snap, &i);
            for end in [0., 0.0001] {
                let mut a = w.clone();
                a.m_in_amb_kg = 0.123;
                let before = format!("{a:?}");
                let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    engine::resume(&mut a, &i, end)
                }));
                emit(
                    "derived_node",
                    json!({"node":node,"case":case,"end":end,"pressure":p,"pressure_finite_positive":p.is_finite()&&p>0.,"density_finite_positive":rho.is_finite()&&rho>0.,"energy_finite":u.is_finite(),"hash_valid":snap.verified(),"restore_ok":rest.is_ok(),"resume_ok":r.as_ref().is_ok_and(|x|x.is_ok()),"panic":r.is_err(),"unchanged":before==format!("{a:?}"),"error":r.as_ref().ok().and_then(|x|x.as_ref().err()).map(|e|e.to_string())}),
                );
            }
        }
    }
    for case in [
        "temperature_overflow",
        "temperature_underflow",
        "pressure_overflow",
    ] {
        let mut w = ws(&base);
        match case {
            "temperature_overflow" => {
                w.cyls[0].m = f64::MIN_POSITIVE;
                w.cyls[0].u = f64::MAX;
            }
            "temperature_underflow" => {
                w.cyls[0].m = 1e100;
                w.cyls[0].u = 1e-300;
            }
            _ => {
                w.cyls[0].m = 1.;
                w.cyls[0].u = 1e307;
            }
        }
        let s = w.snapshot();
        let before = format!("{w:?}");
        let r = engine::resume(&mut w, &base, 0.);
        emit(
            "derived_cylinder_new",
            json!({"case":case,"hash_valid":s.verified(),"restore_ok":WorkState::restore(&s,&base).is_ok(),"resume_ok":r.is_ok(),"unchanged":before==format!("{w:?}")}),
        );
    }
}
fn thermal() {
    for cap in [1e-308, 1e300, 1e307, f64::MAX] {
        for t in [-200., -1., 0., 1., 20., 1e100] {
            let base = engine::healthy_inputs();
            let mut p = thermal::ThermalParams::illustrative();
            p.c_wall = cap;
            let mut i = base.clone();
            i.thermal = Some(p);
            i.t_end_s = 0.;
            let mut w = ws(&base);
            w.therm = Some((thermal::ThermalState::cold(t), p));
            w.th_u0_j = 0.;
            let u = w.therm.as_ref().unwrap().0.stored_j_checked(&p);
            let snap = w.snapshot();
            let before = format!("{w:?}");
            let r = engine::resume(&mut w, &i, 0.);
            emit(
                "thermal_context",
                json!({"capacity":cap,"t_c":t,"actual_stored":u,"expected_valid":u.is_some(),"restore_ok":WorkState::restore(&snap,&i).is_ok(),"resume_ok":r.is_ok(),"unchanged_on_error":r.is_ok()||before==format!("{w:?}"),"run_factory_ok":engine::run_engine(&i).is_ok()}),
            );
        }
    }
}
fn battery() {
    for t in [
        -300.,
        -273.15,
        -273.149999,
        -20.,
        -1.,
        0.,
        20.,
        100.,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ] {
        let mut i = engine::healthy_inputs();
        i.starter.as_mut().unwrap().2.temp_c = t;
        let b = i.starter.as_ref().unwrap().2;
        let rr = b.resistance();
        let vv = b.terminal_voltage(350.);
        i.t_end_s = 0.5;
        let run = engine::run_engine(&i);
        let base = engine::healthy_inputs();
        let mut w = ws(&base);
        let before = format!("{w:?}");
        let re = engine::resume(&mut w, &i, 0.001);
        emit(
            "battery",
            json!({"t_c":t,"label":format!("{t:?}"),"expected_valid":t.is_finite()&&t> -273.15,"validated":b.validated().is_ok(),"resistance_ok":rr.is_ok(),"resistance":rr.ok(),"terminal_voltage_ok":vv.is_ok(),"run_ok":run.is_ok(),"resume_ok":re.is_ok(),"unchanged_on_error":re.is_ok()||before==format!("{w:?}")}),
        );
    }
}
fn battery_extreme() {
    for t in [-20., 10., 20.] {
        for r0 in [f64::MAX / 4., f64::MAX] {
            let mut i = engine::healthy_inputs();
            i.starter.as_mut().unwrap().2.temp_c = t;
            i.starter.as_mut().unwrap().2.r0_ohm = r0;
            let b = i.starter.as_ref().unwrap().2;
            let r = b.resistance();
            i.t_end_s = 0.001;
            let run = engine::run_engine(&i);
            emit(
                "battery_extreme",
                json!({"temp_c":t,"r0":r0,"validated":b.validated().is_ok(),"resistance_ok":r.is_ok(),"resistance_finite":r.as_ref().is_ok_and(|x|x.is_finite()),"run_ok":run.is_ok()}),
            );
        }
    }
}

fn replay() {
    for c2 in [0., 0.01, 0.05] {
        for cut in [0.173, 0.35955, 0.5, 0.913] {
            let mut i = engine::healthy_inputs();
            i.record_every = 1;
            i.friction = crank::FrictionModel::ChenFlynn {
                coeffs: crank::ChenFlynnCoeffs {
                    c1_pa: 20000.,
                    c2_dlless: c2,
                    c3_pa_s_m: 1000.,
                    c4_pa_s2_m2: 100.,
                },
            };
            let mut w = ws(&i);
            engine::resume(&mut w, &i, cut).unwrap();
            let native = w.snapshot();
            let mut missing = native.clone();
            missing.state.pmax_pa = None;
            missing = EngineSnapshot::capture(&missing.state);
            let mut a = WorkState::restore(&native, &i).unwrap();
            let mut b = WorkState::restore(&missing, &i).unwrap();
            let provenance = b.snapshot().provenance;
            let ta = engine::resume(&mut a, &i, 1.2).unwrap();
            let tb = engine::resume(&mut b, &i, 1.2).unwrap();
            let mut a_state = a.snapshot().state;
            let b_state = b.snapshot().state;
            let pmax_equal = a_state.pmax_pa == b_state.pmax_pa;
            a_state.pmax_pa = b_state.pmax_pa;
            emit(
                "history_control",
                json!({"c2":c2,"cut":cut,"provenance":provenance,"trace_equal":ta==tb,"remaining_state_equal":a_state==b_state,"pmax_equal":pmax_equal,"omega_a":a.omega_rad_s,"omega_b":b.omega_rad_s}),
            );
        }
    }
    // Genuine legacy cold -> exact_neutral migration -> native hot. Preserve existing provenance when losing pmax.
    let mut i = engine::healthy_inputs();
    i.record_every = 1;
    i.friction = crank::FrictionModel::ChenFlynn {
        coeffs: crank::ChenFlynnCoeffs {
            c1_pa: 20000.,
            c2_dlless: 0.05,
            c3_pa_s_m: 1000.,
            c4_pa_s2_m2: 100.,
        },
    };
    let legacy: EngineSnapshot =
        serde_json::from_str(include_str!("../fixtures/genuine-v1-cold.json")).unwrap();
    let mut w = WorkState::restore(&legacy, &i).unwrap();
    let original_provenance = w.snapshot().provenance;
    engine::resume(&mut w, &i, 0.5).unwrap();
    let full = w.snapshot();
    let mut payload = full.state.clone();
    payload.pmax_pa = None;
    let missing = EngineSnapshot::capture_with(&payload, full.provenance.clone());
    let mut a = WorkState::restore(&full, &i).unwrap();
    let mut b = WorkState::restore(&missing, &i).unwrap();
    let returned_provenance = b.snapshot().provenance;
    let ta = engine::resume(&mut a, &i, 0.55).unwrap();
    let tb = engine::resume(&mut b, &i, 0.55).unwrap();
    emit(
        "inherited_provenance_loss",
        json!({"origin_hash_valid":legacy.verified(),"missing_hash_valid":missing.verified(),"original_provenance":original_provenance,"returned_provenance":returned_provenance,"trace_equal":ta==tb,"omega_a":a.omega_rad_s,"omega_b":b.omega_rad_s}),
    );
    // c2=0 controls do not remove combustion/event history from native snapshots.
    for field in [
        "burn_latched",
        "burn_accounted",
        "prev_local",
        "fuel_inventory",
    ] {
        let mut i = engine::healthy_inputs();
        i.record_every = 1;
        let mut w = ws(&i);
        engine::resume(&mut w, &i, 0.35955).unwrap();
        let s = w.snapshot();
        let rt: EngineSnapshot = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        let exact = WorkState::restore(&rt, &i).unwrap();
        let mut absent = serde_json::to_value(&s).unwrap();
        match field {
            "prev_local" => {
                absent["state"]
                    .as_object_mut()
                    .unwrap()
                    .remove("prev_local_deg");
            }
            k => {
                let key = match k {
                    "burn_latched" => "burn_latched_kg",
                    "burn_accounted" => "burn_accounted",
                    _ => "fuel_kg",
                };
                absent["state"]["cyls"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove(key);
            }
        }
        let missing_field_rejected = serde_json::from_value::<EngineSnapshot>(absent).is_err();

        emit(
            "history_persisted",
            json!({"field":field,"json_has_field":serde_json::to_string(&s).unwrap().contains(match field{"burn_latched"=>"burn_latched_kg","burn_accounted"=>"burn_accounted","prev_local"=>"prev_local_deg",_=>"fuel_kg"}),"state_equal":w.snapshot()==exact.snapshot(),"missing_field_rejected":missing_field_rejected}),
        );
    }
}
fn semantics() {
    let base = engine::healthy_inputs();
    let mut w = ws(&base);
    engine::resume(&mut w, &base, 0.5).unwrap();
    let source = w.snapshot();
    for schema in [1, 2] {
        for model in [
            None,
            Some((1, 1)),
            Some((2, 1)),
            Some((999, 1)),
            Some((1, 999)),
        ] {
            let mut state = source.state.clone();
            state.schema_version = schema;
            let mut s = EngineSnapshot::capture(&state);
            s.model =
                model.map(
                    |(schema_version, combustion_accounting)| engine::state::ModelIdentity {
                        schema_version,
                        combustion_accounting,
                    },
                );
            let r = WorkState::restore(&s, &base);
            emit(
                "semantic_cross",
                json!({"schema":schema,"model":s.model,"hash_valid":s.verified(),"accepted":r.is_ok(),"provenance":r.as_ref().ok().and_then(|x|x.snapshot().provenance)}),
            );
        }
    }
}
fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("fields") => fields(),
        Some("thermo") => thermo(),
        Some("thermal") => thermal(),
        Some("battery") => {
            battery();
            battery_extreme();
        }
        Some("replay") => replay(),
        Some("semantics") => semantics(),
        _ => panic!("mode required"),
    }
}
