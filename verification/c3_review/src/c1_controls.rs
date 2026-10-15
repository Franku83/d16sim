//! C1 independent extensions: raw observations, no changes to historical oracles.
use engine::{CylState, EngineInputs, EngineSnapshot, EngineState, WorkState};
use serde_json::{json, Value};
use std::f64::consts::PI;
include!("common.rs");
include!("observed_cli.rs");
fn emit(id: &str, data: Value) {
    println!("{}", json!({"probe":id,"data":data}));
}
fn fuel(w: &WorkState) -> f64 {
    w.plen.fuel + w.exh.fuel + w.cyls.iter().map(|c| c.fuel).sum::<f64>()
}
fn boundary() {
    for case in [
        "control_afr_nan",
        "control_afr_zero",
        "control_eta_nan",
        "spark_adv_nan",
        "fuel_lhv_nan",
        "fuel_density_negative",
        "charge_nan",
        "dwell_nan",
        "starter_cut_nan",
        "starter_bad",
        "inertia_nan",
        "inertia_zero",
        "thermal_capacity_zero",
        "thermal_capacity_overflow",
        "thermal_wall_nan",
        "wiebe_duration_inf",
        "friction_overflow",
    ] {
        let valid = engine::healthy_inputs();
        let mut i = valid.clone();
        let mut w = ws(&valid);
        w.m_in_amb_kg = 0.12;
        match case {
            "control_afr_nan" => i.control.afr_target = f64::NAN,
            "control_afr_zero" => i.control.afr_target = 0.,
            "control_eta_nan" => i.control.eta_v_assumed = f64::NAN,
            "spark_adv_nan" => i.control.spark_adv_btdc_deg = f64::NAN,
            "fuel_lhv_nan" => i.fuel.lhv_j_per_kg = f64::NAN,
            "fuel_density_negative" => i.fuel.density_kg_m3 = -1.,
            "charge_nan" => i.charge_volts = f64::NAN,
            "dwell_nan" => i.dwell_s = f64::NAN,
            "starter_cut_nan" => i.starter_cutout_rpm = f64::NAN,
            "starter_bad" => i.starter.as_mut().unwrap().0.stall_torque_n_m = f64::NAN,
            "inertia_nan" => i.inertia.rotating_j_kg_m2 = f64::NAN,
            "inertia_zero" => {
                i.inertia.rotating_j_kg_m2 = 0.;
                i.inertia.recip_mass_kg = 0.;
            }
            "thermal_capacity_zero" => {
                let mut p = thermal::ThermalParams::illustrative();
                p.c_wall = 0.;
                i.thermal = Some(p);
            }
            "thermal_capacity_overflow" => {
                let mut p = thermal::ThermalParams::illustrative();
                p.c_wall = f64::MAX;
                i.thermal = Some(p);
            }
            "thermal_wall_nan" => {
                let mut p = thermal::ThermalParams::illustrative();
                p.wall_frac = f64::NAN;
                i.thermal = Some(p);
            }
            "wiebe_duration_inf" => i.wiebe_duration_deg = f64::INFINITY,
            _ => i.friction = crank::FrictionModel::Viscous { c_n_m_s: f64::MAX },
        }
        let before = format!("{w:?}");
        let snap = cold(&valid);
        let restore = std::panic::catch_unwind(|| WorkState::restore(&snap, &i));
        let run = std::panic::catch_unwind(|| {
            let mut q = i.clone();
            q.t_end_s = 0.001;
            engine::run_engine(&q)
        });
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            engine::resume(&mut w, &i, 0.001)
        }));
        emit(
            "extended_engine_config",
            json!({"case":case,"restore_panicked":restore.is_err(),"restore_accepted":restore.as_ref().is_ok_and(|r|r.is_ok()),"run_panicked":run.is_err(),"run_accepted":run.as_ref().is_ok_and(|r|r.is_ok()),"run_energy_finite":run.as_ref().ok().and_then(|r|r.as_ref().ok()).map(|r|r.audit.energy_closure.is_finite()),"resume_panicked":result.is_err(),"resume_accepted":result.as_ref().is_ok_and(|r|r.is_ok()),"unchanged":before==format!("{w:?}"),"resume_error":result.as_ref().ok().and_then(|r|r.as_ref().err()).map(|e|e.to_string()),"omega":w.omega_rad_s,"state_finite":w.omega_rad_s.is_finite()&&w.theta_rad.is_finite()&&w.w_load_j.is_finite()}),
        );
    }
    for case in [
        "node_v_zero",
        "node_cv_nan",
        "node_v_nan",
        "thermal_presence_mismatch",
        "thermal_stored_overflow",
        "thermal_params_bad",
    ] {
        let mut i = engine::healthy_inputs();
        let mut w = ws(&i);
        match case {
            "node_v_zero" => w.plen.v = 0.,
            "node_cv_nan" => w.plen.cv = f64::NAN,
            "node_v_nan" => w.plen.v = f64::NAN,
            "thermal_presence_mismatch" => {
                i.thermal = Some(thermal::ThermalParams::illustrative());
            }
            "thermal_stored_overflow" => {
                let mut p = thermal::ThermalParams::illustrative();
                p.c_wall = f64::MAX;
                w.therm = Some((thermal::ThermalState::cold(20.), p));
                i.thermal = Some(p);
            }
            _ => {
                let mut p = thermal::ThermalParams::illustrative();
                p.c_wall = 0.;
                w.therm = Some((thermal::ThermalState::cold(20.), p));
                i.thermal = Some(p);
            }
        }
        w.m_in_amb_kg = 0.12;
        let pre = format!("{w:?}");
        let r = engine::resume(&mut w, &i, 0.);
        emit(
            "extended_work_state",
            json!({"case":case,"accepted":r.is_ok(),"unchanged":pre==format!("{w:?}"),"m_initial":w.m_sys_0_kg,"u_initial":w.u_sys_0_j}),
        );
    }
    for case in ["capacity_zero", "wall_frac_nan", "capacity_overflow"] {
        let mut i = engine::healthy_inputs();
        let mut p = thermal::ThermalParams::illustrative();
        match case {
            "capacity_zero" => p.c_wall = 0.,
            "wall_frac_nan" => p.wall_frac = f64::NAN,
            _ => p.c_wall = f64::MAX,
        };
        i.thermal = Some(p);
        let snap = cold(&i);
        let r = WorkState::restore(&snap, &i);
        emit(
            "thermal_restore_config",
            json!({"case":case,"hash_valid":snap.verified(),"accepted":r.is_ok(),"stored_finite":r.as_ref().ok().map(|w|w.th_u0_j.is_finite())}),
        );
    }
    for kind in ["flow", "thermal"] {
        let mut i = engine::healthy_inputs();
        if kind == "thermal" {
            let mut p = thermal::ThermalParams::illustrative();
            p.c_wall = 1e-308;
            i.thermal = Some(p);
        }
        let mut w = ws(&i);
        w.m_in_amb_kg = 0.12;
        if kind == "flow" {
            i.throttle_area_m2 = 1e4;
            w.plen.m *= 0.5;
        } else {
            w.cyls[0].fuel = 1e-6;
            w.cyls[0].burn_latched = 1e-6;
            w.cyls[0].burn_accounted = false;
            w.theta_rad = 365f64.to_radians();
            w.omega_rad_s = 100.;
        }
        let before = format!("{w:?}");
        let r = engine::resume(&mut w, &i, 0.001);
        emit(
            "runtime_error_atomic",
            json!({"kind":kind,"rejected":r.is_err(),"unchanged":before==format!("{w:?}"),"error":r.err().map(|e|e.to_string())}),
        );
    }
}
fn chemistry() {
    for dt in [100e-6, 50e-6, 25e-6] {
        for mode in ["nominal", "interrupt_future_ignition", "starved", "reflux"] {
            let mut i = engine::healthy_inputs();
            i.dt_s = dt;
            i.record_every = 1;
            let mut w = ws(&i);
            if mode == "starved" {
                i.control.fuel_enabled = false;
                for c in &mut w.cyls {
                    c.fuel = 1e-4 * c.m;
                }
            }
            if mode == "reflux" {
                w.exh.m *= 2.;
                w.exh.fuel = 0.02 * w.exh.m;
                w.plen.fuel = 0.01 * w.plen.m;
            }
            let f0 = fuel(&w);
            let mut budget = 0f64;
            let mut relation = 0f64;
            let mut max_fraction = 0f64;
            let mut steps = 0usize;
            let mut reacting = 0usize;
            let mut error = None;
            let mut replay_equal = true;
            let mut cut_count = 0;
            let mut interrupted = false;
            while w.time_s < 0.7 - 1e-15 {
                let pre = w.clone();
                let t = w.time_s + dt * 0.5;
                if let Err(e) = engine::resume(&mut w, &i, t) {
                    error = Some(e.to_string());
                    break;
                }
                steps += 1;
                let dq = w.heat_released_j - pre.heat_released_j;
                // Independent inventory balance: storage + emitted difference, allowing injection.
                let consumed = fuel(&pre) + (w.fuel_injected_kg - pre.fuel_injected_kg)
                    - fuel(&w)
                    - (w.fuel_emitted_kg - pre.fuel_emitted_kg);
                relation = relation
                    .max((dq - consumed * i.fuel.lhv_j_per_kg * i.combustion_efficiency).abs());
                budget = budget.max(
                    (f0 + w.fuel_injected_kg - fuel(&w) - w.fuel_burned_kg - w.fuel_emitted_kg)
                        .abs(),
                );
                for c in &w.cyls {
                    max_fraction = max_fraction.max(c.fuel / c.m);
                }
                for n in [&w.plen, &w.exh] {
                    max_fraction = max_fraction.max(n.fuel / n.m);
                }
                if dq > 0. {
                    reacting += 1;
                    if cut_count < 3 && reacting % 113 == 0 {
                        let text = serde_json::to_string(&w.snapshot()).unwrap();
                        let snap: EngineSnapshot = serde_json::from_str(&text).unwrap();
                        let mut a = w.clone();
                        let mut b = WorkState::restore(&snap, &i).unwrap();
                        let stop = w.time_s + dt * 17.5;
                        let x = engine::resume(&mut a, &i, stop).unwrap();
                        let y = engine::resume(&mut b, &i, stop).unwrap();
                        replay_equal &= a.snapshot() == b.snapshot() && x == y;
                        cut_count += 1;
                    }
                    if mode == "interrupt_future_ignition" && !interrupted && reacting > 100 {
                        i.control.spark_enabled = false;
                        i.control.fuel_enabled = false;
                        interrupted = true;
                    }
                }
            }
            emit(
                "step_chemical_budget",
                json!({"dt":dt,"mode":mode,"steps":steps,"reacting_steps":reacting,"error":error,"inventory_closure_kg":budget,"max_step_inventory_heat_error_j":relation,"max_fraction":max_fraction,"replay_equal":replay_equal,"burn_window_cuts":cut_count,"interrupted":interrupted,"Q":w.heat_released_j,"burned":w.fuel_burned_kg,"rpm":w.omega_rad_s*30./PI}),
            );
        }
    }
}
fn diagnostic() {
    use leak::{Fluid, LeakDomain, LeakNetwork, LeakPort};
    let gas = |p: f64| LeakDomain {
        v_m3: 0.01,
        m_kg: p * 0.01 / (287. * 300.),
        t_k: 300.,
        fluid: Fluid::Gas {
            r_gas: 287.,
            cp: 1005.,
        },
        fresh_frac: 1.,
    };
    let net = LeakNetwork {
        domains: vec![gas(500000.), gas(100000.), gas(100000.), gas(100000.)],
        p_set_pa: vec![0.; 4],
        ports: (1..=3)
            .map(|to| LeakPort {
                from: 0,
                to,
                conductance: to as f64 * 1e-8,
                compressible: true,
            })
            .collect(),
        mu_liquid_pa_s: 0.02,
        supply_t_k: 300.,
    };
    for label in ["renamed_A", "swapped_B"] {
        for flows in [vec![1., 2., 7.], vec![7., 2., 1.], vec![0., 0., 0.]] {
            let (s, flag) = observable_chamber_split(&net, &flows);
            emit(
                "diagnostic_labels",
                json!({"label":label,"flows":flows,"split":[s.to_crankcase_01,s.to_exhaust_01,s.to_coolant_01],"flag":flag}),
            );
        }
    }
    for (comp, leak, split) in [
        (None, None, None),
        (Some(500.), Some(30.), None),
        (
            None,
            Some(30.),
            Some(diagnostics::LeakSplit {
                to_crankcase_01: 0.1,
                to_exhaust_01: 0.7,
                to_coolant_01: 0.2,
            }),
        ),
    ] {
        emit(
            "diagnostic_incomplete",
            json!({"verdict":format!("{:?}",diagnostics::diagnose_low_compression(comp,900.,leak,split))}),
        );
    }
    // Domain negative controls: these are invalid measurements, not multiple-cause identifiability tests.
    for (case, comp, pct, s) in [
        ("compression_nan", f64::NAN, 30., [1., 0., 0.]),
        ("leak_inf", 500., f64::INFINITY, [1., 0., 0.]),
        ("split_negative", 500., 30., [2., -1., 0.]),
        ("split_partial", 500., 30., [0.1, 0., 0.]),
    ] {
        let split = diagnostics::LeakSplit {
            to_crankcase_01: s[0],
            to_exhaust_01: s[1],
            to_coolant_01: s[2],
        };
        emit(
            "diagnostic_invalid",
            json!({"case":case,"verdict":format!("{:?}",diagnostics::diagnose_low_compression(Some(comp),900.,Some(pct),Some(split)))}),
        );
    }
}
fn migration() {
    let out = std::env::var("D16_REVIEW_RESULTS").unwrap();
    let i = engine::healthy_inputs();
    for name in [
        "historical-v1-cold.json",
        "historical-v1-hot.json",
        "historical-v2-cold.json",
        "historical-v2-hot.json",
    ] {
        let text = std::fs::read_to_string(format!("{out}/{name}")).unwrap();
        let s: EngineSnapshot = serde_json::from_str(&text).unwrap();
        let mut w = WorkState::restore(&s, &i).unwrap();
        let migrated = w.snapshot();
        let json = serde_json::to_string(&migrated).unwrap();
        let parsed: EngineSnapshot = serde_json::from_str(&json).unwrap();
        let mut again = WorkState::restore(&parsed, &i).unwrap();
        let stop = w.time_s + 0.05;
        let a = engine::resume(&mut w, &i, stop).unwrap();
        let b = engine::resume(&mut again, &i, stop).unwrap();
        emit(
            "historical_provenance",
            json!({"name":name,"origin_schema":s.state.schema_version,"origin_hash":s.state_hash,"verified":s.verified(),"provenance":migrated.provenance,"roundtrip_verified":parsed.verified(),"second_restore_equal":w.snapshot()==again.snapshot()&&a==b,"continuation_preserved":w.snapshot().provenance==migrated.provenance}),
        );
    }
    // Native produced under 8a remains a valid input to C1 (new sign policy does not rewrite positive zero).
    let native: EngineSnapshot = serde_json::from_str(
        &std::fs::read_to_string(format!("{out}/historical-r02-native.json")).unwrap(),
    )
    .unwrap();
    emit(
        "r02_hash_compatibility",
        json!({"verified":native.verified(),"restore_accepted":WorkState::restore(&native,&i).is_ok(),"hash":native.state_hash}),
    );
    let mut w = ws(&i);
    engine::resume(&mut w, &i, 0.5).unwrap();
    for field in ["plen", "exh", "emitted"] {
        for neg in [false, true] {
            let mut s = w.snapshot().state;
            let value = if neg { -0.0 } else { 0.0 };
            match field {
                "plen" => s.plen_fuel_kg = value,
                "exh" => s.exh_fuel_kg = value,
                _ => s.fuel_emitted_kg = value,
            };
            let snap = EngineSnapshot::capture(&s);
            let parsed: EngineSnapshot =
                serde_json::from_str(&serde_json::to_string(&snap).unwrap()).unwrap();
            let restored = WorkState::restore(&parsed, &i).unwrap();
            let after = match field {
                "plen" => restored.plen.fuel,
                "exh" => restored.exh.fuel,
                _ => restored.fuel_emitted_kg,
            };
            emit(
                "signed_zero_restore",
                json!({"field":field,"negative":neg,"before_bits":value.to_bits(),"after_bits":after.to_bits(),"hash_equal":snap.state_hash==restored.snapshot().state_hash}),
            );
        }
    }
    let historical: EngineSnapshot = serde_json::from_str(
        &std::fs::read_to_string(format!("{out}/historical-r02-mid-burn.json")).unwrap(),
    )
    .unwrap();
    let mut legacy = WorkState::restore(&historical, &i).unwrap();
    let before = legacy.snapshot();
    let old_excess = legacy.heat_released_j
        - legacy.fuel_burned_kg * i.fuel.lhv_j_per_kg * i.combustion_efficiency;
    engine::resume(&mut legacy, &i, historical.state.time_s + 0.02).unwrap();
    emit(
        "r02_midburn_migration",
        json!({"historical_hash":historical.state_hash,"hash_verified":historical.verified(),"time":historical.state.time_s,"old_burn_accounted":historical.state.cyls.map(|c|c.burn_accounted),"old_chemical_excess_j":old_excess,"restored_provenance":before.provenance,"continued_provenance":legacy.snapshot().provenance,"continued_chemical_excess_j":legacy.heat_released_j-legacy.fuel_burned_kg*i.fuel.lhv_j_per_kg*i.combustion_efficiency}),
    );
    // Metadata tampering is separate from state-hash contract (hash covers state only).
    let legacy: EngineSnapshot = serde_json::from_str(
        &std::fs::read_to_string(format!("{out}/historical-v1-hot.json")).unwrap(),
    )
    .unwrap();
    let mut marked = WorkState::restore(&legacy, &i).unwrap().snapshot();
    marked.provenance.as_mut().unwrap().continuation = "exact_neutral".into();
    marked.provenance.as_mut().unwrap().original_hash = "forged".into();
    let accepted = WorkState::restore(&marked, &i);
    emit(
        "provenance_tamper_control",
        json!({"state_hash_still_valid":marked.verified(),"accepted":accepted.is_ok(),"continued_label":accepted.ok().unwrap().snapshot().provenance}),
    );
}
fn thermal() {
    for case in [
        "nan",
        "inf",
        "below_abs",
        "at_abs",
        "cap_zero",
        "cap_tiny",
        "capacity_large",
        "ambient_inf",
        "ambient_cold",
        "huge_dt",
    ] {
        let mut p = thermal::ThermalParams::illustrative();
        let mut s = thermal::ThermalState::cold(20.);
        let mut amb = 20.;
        let mut dt = 0.1;
        let mut q = 10.;
        match case {
            "nan" => s.t_piston_c = f64::NAN,
            "inf" => s.t_oil_c = f64::INFINITY,
            "below_abs" => s.t_cool_c = -300.,
            "at_abs" => s.ect_filt_c = -273.15,
            "cap_zero" => p.c_cool = 0.,
            "cap_tiny" => {
                p.c_wall = 1e-308;
                q = 100.;
            }
            "capacity_large" => p.c_wall = f64::MAX,
            "ambient_inf" => amb = f64::INFINITY,
            "ambient_cold" => amb = -300.,
            _ => {
                dt = f64::MAX;
                s.t_wall_c = 100.;
            }
        }
        let pre = format!("{s:?}");
        let r = s.step(dt, q, 3., amb, &p);
        emit(
            "extended_thermal",
            json!({"case":case,"accepted":r.is_ok(),"unchanged":pre==format!("{s:?}"),"stored_energy_finite":s.stored_j(&p).is_finite(),"ambient_Q_finite":r.ok().map(|v|v.is_finite())}),
        );
    }
    for temp in [-272., -100., 20., 84., 100.] {
        let p = thermal::ThermalParams::illustrative();
        let mut s = thermal::ThermalState::cold(temp);
        let u = s.stored_j(&p);
        let q = s.step(0.01, 0.1, 3., 20., &p).unwrap();
        emit(
            "thermal_physical_control",
            json!({"initial":temp,"closure_j":s.stored_j(&p)-u-0.13+q,"all_above_abs":s.t_wall_c> -273.15&&s.t_cool_c> -273.15}),
        );
    }
}
fn electronics() {
    use ecu::{
        decoder::{Decoder, SyncEstimate, SyncState},
        dtc::DtcSet,
        maps::ReferenceMaps,
        scheduler::Ecu,
    };
    let e = Ecu::new(ReferenceMaps::illustrative().unwrap(), 0.001, 12., 300000.).unwrap();
    for field in ["rpm", "confidence", "phase", "time", "load"] {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut s = SyncEstimate {
                state: SyncState::Synced720,
                rpm: 1200.,
                confidence: 1.,
                cycle_deg_at: 100.,
                at_time_s: 1.,
            };
            let mut load = 0.5;
            match field {
                "rpm" => s.rpm = bad,
                "confidence" => s.confidence = bad,
                "phase" => s.cycle_deg_at = bad,
                "time" => s.at_time_s = bad,
                _ => load = bad,
            };
            let p = e.schedule(&s, load, &DtcSet { active: vec![] });
            emit(
                "ecu_nonfinite_all",
                json!({"field":field,"value":format!("{bad:?}"),"injections":p.injections.len(),"ignitions":p.ignitions.len(),"state":format!("{:?}",p.ecu_state)}),
            );
        }
    }
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut d = Decoder::new(vec![0., 0.01, 0.02, 0.03], vec![], 0.03).unwrap();
        d.update(bad);
        let est = d.estimate(bad);
        let p = e.schedule(&est, 0.5, &DtcSet { active: vec![] });
        emit(
            "decoder_nonfinite_all",
            json!({"value":format!("{bad:?}"),"finite":est.rpm.is_finite()&&est.cycle_deg_at.is_finite()&&est.at_time_s.is_finite(),"state":format!("{:?}",est.state),"commands":p.injections.len()+p.ignitions.len()}),
        );
    }
}
fn fit() {
    for case in [
        "short_first",
        "long_first",
        "short_later",
        "long_later",
        "nan_first",
        "inf_later",
        "y_nan",
        "w_inf",
        "p_nan",
        "w_mismatch",
    ] {
        use std::cell::Cell;
        let calls = Cell::new(0);
        let mut y = vec![1., 2.];
        let mut weight = vec![1., 1.];
        let mut p = vec![0.5];
        match case {
            "y_nan" => y[0] = f64::NAN,
            "w_inf" => weight[1] = f64::INFINITY,
            "p_nan" => p[0] = f64::NAN,
            "w_mismatch" => {
                weight.pop();
            }
            _ => {}
        }
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            calibration::least_squares(
                |p| {
                    let n = calls.get() + 1;
                    calls.set(n);
                    match case {
                        "short_first" => vec![],
                        "long_first" => vec![1.; 3],
                        "short_later" if n > 1 => vec![1.],
                        "long_later" if n > 1 => vec![1.; 3],
                        "nan_first" => vec![f64::NAN; 2],
                        "inf_later" if n > 1 => vec![f64::INFINITY; 2],
                        _ => vec![p[0], 2. * p[0]],
                    }
                },
                &y,
                &weight,
                &p,
                1e-8,
            )
        }));
        emit(
            "fitter_boundaries",
            json!({"case":case,"calls":calls.get(),"panicked":r.is_err(),"rejected":r.as_ref().is_ok_and(|v|v.is_err()),"error":r.as_ref().ok().and_then(|v|v.as_ref().err()).map(|e|e.to_string())}),
        );
    }
}
fn dyno() {
    for (p, t) in [
        (1e-308, 25.),
        (f64::MIN_POSITIVE, 25.),
        (99., f64::MAX),
        (f64::NAN, 25.),
        (99., f64::INFINITY),
        (99., -300.),
        (99., 25.),
    ] {
        for apply in [false, true] {
            let r = dyno::Correction::of(&dyno::Ambient { p_kpa: p, t_c: t }, apply);
            emit(
                "dyno_boundaries",
                json!({"p":format!("{p:?}"),"t":format!("{t:?}"),"apply":apply,"accepted":r.is_ok(),"finite":r.as_ref().ok().map(|x|x.factor.is_finite()),"factor":r.as_ref().ok().map(|x|x.factor)}),
            );
        }
    }
}
fn analytic_burn() {
    for dt in [100e-6, 50e-6, 25e-6, 12.5e-6, 6.25e-6] {
        for efficiency in [0.25, 0.5, 1.0] {
            let mut i = engine::healthy_inputs();
            i.dt_s = dt;
            i.combustion_efficiency = efficiency;
            i.starter = None;
            i.control.fuel_enabled = false;
            i.control.spark_enabled = false;
            i.inertia.rotating_j_kg_m2 = 1e9;
            let mut w = ws(&i);
            w.theta_rad = 345f64.to_radians();
            w.omega_rad_s = 100.;
            w.prev_local_deg = [
                kinematics::CylinderId::Cyl1,
                kinematics::CylinderId::Cyl2,
                kinematics::CylinderId::Cyl3,
                kinematics::CylinderId::Cyl4,
            ]
            .map(|c| kinematics::cylinder_local_angle(345., c));
            w.cyls[0].fuel = 1e-6;
            w.cyls[0].burn_latched = 1e-6;
            w.cyls[0].burn_accounted = false;
            let result = engine::resume(&mut w, &i, 80f64.to_radians() / 100.);
            // Integrate the rate against its independent closed-form antiderivative.
            let exact = 1e-6 * (1. - (-i.wiebe_a).exp());
            emit(
                "burn_analytic",
                json!({"dt":dt,"efficiency":efficiency,"accepted":result.is_ok(),"expected_burned_kg":exact,"burned_kg":w.fuel_burned_kg,"error_kg":(w.fuel_burned_kg-exact).abs(),"energy_error_j":(w.heat_released_j-w.fuel_burned_kg*i.fuel.lhv_j_per_kg*efficiency).abs(),"remaining_cyl_kg":w.cyls[0].fuel,"omega_delta":w.omega_rad_s-100.}),
            );
        }
    }
}
fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("boundary") => boundary(),
        Some("chemistry") => chemistry(),
        Some("analytic") => analytic_burn(),
        Some("diagnostic") => diagnostic(),
        Some("migration") => migration(),
        Some("thermal") => thermal(),
        Some("electronics") => electronics(),
        Some("fit") => fit(),
        Some("dyno") => dyno(),
        _ => panic!("mode required"),
    }
}
