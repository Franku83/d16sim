//! Independent observations, never experimental measurements or production fixes.
//! Every JSON row is a scenario, not an assertion or a Rust test target.
use engine::{CylState, EngineInputs, EngineSnapshot, EngineState, WorkState};
use serde_json::{json, Value};
use std::f64::consts::PI;

fn emit(id: &str, data: Value) {
    println!("{}", json!({"probe":id,"data":data}));
}
fn rpm(w: &WorkState) -> f64 {
    w.omega_rad_s * 30.0 / PI
}
fn idle() -> EngineInputs {
    let mut i = engine::healthy_inputs();
    i.throttle_area_m2 = 0.05e-4;
    i.idle_bypass_area_m2 = 0.02e-4;
    i
}
// Fixture equivalent to the declared ambient initial condition. It is not an oracle.
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
        thermal: i.thermal.map(|_| engine::state::ThermalSnap {
            t_wall_c: 20.0,
            t_piston_c: 20.0,
            t_oil_c: 20.0,
            t_cool_c: 20.0,
            ect_filt_c: 20.0,
        }),
    })
}
fn ws(i: &EngineInputs) -> WorkState {
    WorkState::restore(&cold(i), i).unwrap()
}
fn engine_summary(i: &EngineInputs) -> Value {
    match engine::run_engine(i) {
        Ok(t) => json!({"ok":true,"rpm":t.audit.final_rpm,
            "tail_rpm_mean":t.samples[t.samples.len()*4/5..].iter().map(|s|s.rpm).sum::<f64>()/(t.samples.len()-t.samples.len()*4/5) as f64,
            "mass":t.audit.mass_closure,
            "energy":t.audit.energy_closure,"Q_j":t.audit.heat_released_j,
            "fuel_in_kg":t.audit.fuel_injected_kg,"fuel_burn_kg":t.audit.fuel_burned_kg,
            "work_ind_j":t.audit.w_ind_j,"thermal":t.audit.thermal}),
        Err(e) => json!({"ok":false,"error":e.to_string()}),
    }
}
fn engine_probes() {
    for point in ["idle", "base"] {
        for dt in [200e-6, 100e-6, 50e-6, 25e-6] {
            for seconds in [3.0, 8.0, 30.0] {
                let mut i = if point == "idle" {
                    idle()
                } else {
                    engine::healthy_inputs()
                };
                i.dt_s = dt;
                i.t_end_s = seconds;
                emit(
                    "engine_refinement",
                    json!({"point":point,"dt":dt,"seconds":seconds,"result":engine_summary(&i)}),
                );
            }
        }
    }
    for dt in [100e-6, 50e-6, 25e-6] {
        let mut i = idle();
        i.dt_s = dt;
        i.t_end_s = 120.0;
        i.thermal = Some(thermal::ThermalParams::illustrative());
        emit(
            "thermal_long",
            json!({"dt":dt,"seconds":120,"result":engine_summary(&i)}),
        );
    }
    for dt in [12.5e-6, 6.25e-6] {
        let mut i = idle();
        i.dt_s = dt;
        i.t_end_s = 8.0;
        emit("idle_finer", json!({"dt":dt,"result":engine_summary(&i)}));
    }
    for scenario in ["no_fuel", "no_spark", "high_friction"] {
        let mut i = idle();
        i.t_end_s = 8.0;
        match scenario {
            "no_fuel" => i.control.fuel_enabled = false,
            "no_spark" => i.control.spark_enabled = false,
            _ => i.friction = engine::high_friction_inputs().friction,
        }
        emit(
            "engine_negative",
            json!({"scenario":scenario,"result":engine_summary(&i)}),
        );
    }
    let i = idle();
    let mut w = ws(&i);
    engine::resume(&mut w, &i, 3.0).unwrap();
    let before = rpm(&w);
    let q0 = w.heat_released_j;
    let mut off = i.clone();
    off.control.fuel_enabled = false;
    engine::resume(&mut w, &off, 8.0).unwrap();
    emit(
        "fuel_cut",
        json!({"rpm_before":before,"rpm_after":rpm(&w),"Q_after_cut_j":w.heat_released_j-q0}),
    );
    // Unburned fuel must be bounded by the total material in the same chamber.
    let mut ns = idle();
    ns.control.spark_enabled = false;
    let mut w = ws(&ns);
    let mut max_ratio = 0.0_f64;
    for n in 1..=200 {
        engine::resume(&mut w, &ns, n as f64 * 0.02).unwrap();
        max_ratio = max_ratio.max(w.cyls.iter().map(|c| c.fuel / c.m).fold(0.0, f64::max));
    }
    emit(
        "fuel_inventory",
        json!({"rpm":rpm(&w),"ratios":w.cyls.iter().map(|c|c.fuel/c.m).collect::<Vec<_>>(),
        "max_ratio_over_4s":max_ratio,
        "fuel_kg":w.cyls.iter().map(|c|c.fuel).collect::<Vec<_>>(),"mass_kg":w.cyls.iter().map(|c|c.m).collect::<Vec<_>>()}),
    );
    // Isolate outflow of a marked vapor species at identical mass/energy.
    let mut i = idle();
    i.control.fuel_enabled = false;
    i.control.spark_enabled = false;
    let mut w = ws(&i);
    w.omega_rad_s = 100.0;
    w.theta_rad = PI / 2.0;
    w.cyls[0].m = 200000.0 * i.geom.volume(PI / 2.0) / (i.mix.r() * 293.15);
    w.cyls[0].u = w.cyls[0].m * i.mix.cv() * 293.15;
    w.cyls[0].fuel = w.cyls[0].m * 0.1;
    let (m0, f0) = (w.cyls[0].m, w.cyls[0].fuel);
    engine::resume(&mut w, &i, i.dt_s).unwrap();
    emit(
        "fuel_outflow",
        json!({"mass_before":m0,"mass_after":w.cyls[0].m,"fuel_before":f0,
        "fuel_after":w.cyls[0].fuel,"expected_homogeneous_fuel_after":f0*w.cyls[0].m/m0}),
    );
    for bad in [
        "negative_efficiency",
        "efficiency_above_one",
        "negative_load",
        "zero_record",
    ] {
        let mut i = idle();
        i.t_end_s = 0.01;
        match bad {
            "negative_efficiency" => i.combustion_efficiency = -1.0,
            "efficiency_above_one" => i.combustion_efficiency = 2.0,
            "negative_load" => i.accessory_tau_n_m = -100.0,
            _ => i.record_every = 0,
        }
        let outcome = std::panic::catch_unwind(|| engine::run_engine(&i));
        emit(
            "engine_invalid",
            json!({"case":bad,"returned_error":matches!(outcome,Ok(Err(_))),"panic":outcome.is_err()}),
        );
    }
}
fn cam_probes() {
    for shift in [-180.0, -30.0, -6.0, 0.0, 6.0, 30.0, 180.0, 720.0] {
        let mut i = idle();
        i.intake_cam.centerline_atdc_deg += shift;
        i.t_end_s = 8.0;
        let rel = 70.0 - i.intake_cam.centerline_atdc_deg;
        let lift = i.intake_cam.lift_at(rel);
        let mut w = ws(&i);
        let r = engine::resume(&mut w, &i, 8.0);
        emit(
            "cam_integrated",
            json!({"shift_deg":shift,"lift_at_70_mm":lift,"ok":r.is_ok(),
            "rpm":rpm(&w),"fuel_kg":w.fuel_injected_kg,"heat_j":w.heat_released_j,
            "work_j":w.w_ind_j,"cyl_masses_kg":w.cyls.iter().map(|c|c.m).collect::<Vec<_>>()}),
        );
        for step in [0.25, 0.125] {
            let b = valvetrain::BeltDrive::new(100.0, shift / 7.2).unwrap();
            let inp = flow::GasExchangeInputs {
                rpm: 1500.0,
                p_amb_pa: 101325.0,
                t_amb_k: 300.0,
                plenum_v_m3: 2e-3,
                exhaust_v_m3: 1.5e-3,
                throttle_area_m2: 8e-4,
                tailpipe_area_m2: 8e-4,
                step_deg: step,
                exhaust_init: None,
            };
            let base = idle();
            match flow::run_cycle(
                &base.geom,
                base.mix,
                &base.intake_cam,
                &base.exhaust_cam,
                &b,
                inp,
            ) {
                Ok((t, a)) => emit(
                    "cam_flow",
                    json!({"shift":shift,"step":step,"trapped_kg":a.trapped_kg,
                    "mass":a.mass_closure,"energy":a.energy_closure,"max_p":a.p_max_pa,
                    "intake_signed_sum":t.iter().map(|x|x.mdot_iv_kg_s).sum::<f64>()}),
                ),
                Err(e) => emit(
                    "cam_flow",
                    json!({"shift":shift,"step":step,"error":e.to_string()}),
                ),
            }
        }
    }
    for shift in [-6.0, 0.0, 6.0] {
        let mut i = idle();
        i.intake_cam.centerline_atdc_deg += shift;
        let mut w = ws(&i);
        engine::resume(&mut w, &i, 2.0).unwrap();
        let (q0, work0) = (w.heat_released_j, w.w_ind_j);
        let mut trapped = Vec::new();
        let mut latched = Vec::new();
        let ic = i.intake_cam.centerline_atdc_deg + i.intake_cam.adv_duration_crank_deg / 2.0;
        let fire = 360.0 - i.control.spark_adv_btdc_deg;
        let crossed = |a: f64, b: f64, x: f64| {
            if b >= a {
                a < x && b >= x
            } else {
                x > a || x <= b
            }
        };
        for _ in 0..10000 {
            let previous = w.prev_local_deg;
            let end = w.time_s + i.dt_s;
            engine::resume(&mut w, &i, end).unwrap();
            for k in 0..4 {
                if crossed(previous[k], w.prev_local_deg[k], ic) {
                    trapped.push(w.cyls[k].m);
                }
                if crossed(previous[k], w.prev_local_deg[k], fire) {
                    latched.push(w.cyls[k].burn_latched);
                }
            }
        }
        let mean = |v: &Vec<f64>| v.iter().sum::<f64>() / v.len() as f64;
        emit(
            "cam_event_chain",
            json!({"shift":shift,"ivc_deg":ic,"ivc_observations":trapped.len(),
            "mean_trapped_total_kg":mean(&trapped),"spark_observations":latched.len(),
            "mean_latched_fuel_kg":mean(&latched),"delta_Q_j":w.heat_released_j-q0,
            "delta_work_j":w.w_ind_j-work0,"final_rpm":rpm(&w),
            "note":"intake centerline also changes bundled injection at IVC; not an isolated fuel-timing experiment"}),
        );
    }
}
fn replay_probes() {
    for c2 in [0.0, 0.05] {
        let mut i = idle();
        if let crank::FrictionModel::ChenFlynn { ref mut coeffs } = i.friction {
            coeffs.c2_dlless = c2;
        }
        let mut a = ws(&i);
        engine::resume(&mut a, &i, 3.0).unwrap();
        let mut b = ws(&i);
        engine::resume(&mut b, &i, 1.0).unwrap();
        let snap = b.snapshot();
        let bytes = engine_types::canonical_json(&snap);
        let parsed: EngineSnapshot = serde_json::from_str(&bytes).unwrap();
        emit(
            "snapshot_json",
            json!({"c2":c2,"same_hash":parsed.verified(),"state_equal":parsed.state==snap.state,
            "before_json":engine_types::canonical_json(&snap.state),"after_json":engine_types::canonical_json(&parsed.state)}),
        );
        let mut b = WorkState::restore(&snap, &i).unwrap();
        engine::resume(&mut b, &i, 3.0).unwrap();
        emit(
            "replay_split",
            json!({"c2":c2,"hash_match":a.snapshot().state_hash==b.snapshot().state_hash,
            "rpm_full":rpm(&a),"rpm_split":rpm(&b),"theta_diff":a.theta_rad-b.theta_rad}),
        );
    }
    let i = idle();
    let mut v1 = cold(&i);
    v1.state.schema_version = 1;
    let raw = engine_types::canonical_json(&v1.state).replace(",\"thermal\":null", "");
    v1.state_hash = engine_types::sha256_hex(raw.as_bytes());
    emit(
        "v1_hash_migration",
        json!({"old_bytes":raw,"old_hash":v1.state_hash,
        "accepted":WorkState::restore(&v1,&i).is_ok()}),
    );
    let mut invalid = cold(&i);
    invalid.state.cyls[0].m_kg = -1.0;
    invalid = EngineSnapshot::capture(&invalid.state);
    emit(
        "restore_invalid",
        json!({"negative_mass_accepted":WorkState::restore(&invalid,&i).is_ok()}),
    );
    let mut future = cold(&i);
    future.state.schema_version = 99;
    future = EngineSnapshot::capture(&future.state);
    let mut tampered = cold(&i);
    tampered.state.plen_m_kg *= 2.0;
    emit(
        "restore_negative_controls",
        json!({"schema99_rejected":WorkState::restore(&future,&i).is_err(),
        "tampered_rejected":WorkState::restore(&tampered,&i).is_err()}),
    );
}
fn leak_probes() {
    for scenario in [
        leak::LeakScenario::HealthyBaseline,
        leak::LeakScenario::UnderclampedHead,
        leak::LeakScenario::MissingHeadGasket,
        leak::LeakScenario::RingLeakCyl3,
    ] {
        for dt in [5e-4, 2.5e-4] {
            let net = leak::leak_scenario(scenario).unwrap();
            let (domains, a, tot) = leak::run_leakdown(&net, 0, 10.0, dt).unwrap();
            let mut routed = [0.0; 3];
            for (p, m) in net.ports.iter().zip(&tot) {
                if p.from == 0 {
                    match p.to {
                        1 => routed[0] += m,
                        2 => routed[2] += m,
                        _ => {}
                    }
                }
            }
            let sum = routed.iter().sum::<f64>();
            emit(
                "leak_scenario",
                json!({"scenario":format!("{scenario:?}"),"dt":dt,
                "steady_g_min":a.steady_leak_g_s*60.0,"mass":a.mass_closure,"energy":a.energy_closure,
                "physical_chamber_split":routed.map(|x|x/sum),"pcyl_pa":domains[0].pressure(0.0)}),
            );
        }
    }
    use leak::{Fluid, LeakDomain, LeakNetwork, LeakPort};
    let gas = Fluid::Gas {
        r_gas: 287.0,
        cp: 1005.0,
    };
    let net = LeakNetwork {
        domains: vec![
            LeakDomain {
                v_m3: 0.1,
                m_kg: 200000.0 * 0.1 / (287.0 * 300.0),
                t_k: 300.0,
                fluid: gas,
                fresh_frac: 1.0,
            },
            LeakDomain {
                v_m3: 0.1,
                m_kg: 100000.0 * 0.1 / (287.0 * 300.0),
                t_k: 300.0,
                fluid: gas,
                fresh_frac: 0.0,
            },
        ],
        p_set_pa: vec![0.0, 0.0],
        ports: vec![LeakPort {
            from: 0,
            to: 1,
            conductance: 1e-5,
            compressible: true,
        }],
        mu_liquid_pa_s: 0.1,
        supply_t_k: 300.0,
    };
    let (d, _, tot) = leak::run_leakdown(&net, 0, 0.1, 1e-4).unwrap();
    emit(
        "leak_species",
        json!({"mass_transferred":tot[0],"receiver_fresh":d[1].fresh_frac,
        "expected_receiver_fresh":tot[0]/d[1].m_kg}),
    );
    let net = LeakNetwork {
        domains: vec![
            LeakDomain {
                v_m3: 0.1,
                m_kg: 1.0,
                t_k: 300.0,
                fluid: gas,
                fresh_frac: 1.0,
            },
            LeakDomain {
                v_m3: 1e-3,
                m_kg: 0.87,
                t_k: 300.0,
                fluid: Fluid::Oil,
                fresh_frac: 1.0,
            },
            LeakDomain {
                v_m3: 1e-3,
                m_kg: 0.87,
                t_k: 300.0,
                fluid: Fluid::Oil,
                fresh_frac: 1.0,
            },
        ],
        p_set_pa: vec![0.0, 200000.0, 100000.0],
        ports: vec![LeakPort {
            from: 1,
            to: 2,
            conductance: 1e-12,
            compressible: false,
        }],
        mu_liquid_pa_s: 0.1,
        supply_t_k: 300.0,
    };
    let (_, _, tot) = leak::run_leakdown(&net, 0, 1.0, 1e-4).unwrap();
    emit(
        "leak_liquid_units",
        json!({"reported_mass_kg":tot[0],"volume_m3":1e-12*100000.0/0.1,
        "expected_mass_kg_at_declared_870kg_m3":870.0*1e-12*100000.0/0.1}),
    );
    use diagnostics::{diagnose_low_compression, LeakSplit};
    emit(
        "diagnosis_multicause",
        json!({"rings_plus_valve_no_coolant":format!("{:?}",diagnose_low_compression(
        Some(500.0),900.0,Some(30.0),Some(LeakSplit{to_crankcase_01:0.5,to_exhaust_01:0.5,to_coolant_01:0.0}))),
        "zero_split":format!("{:?}",diagnose_low_compression(Some(500.0),900.0,Some(30.0),
            Some(LeakSplit{to_crankcase_01:0.0,to_exhaust_01:0.0,to_coolant_01:0.0})))}),
    );
}
fn wear_probes() {
    let nominal = wear::OperatingPoint {
        rpm: 3000.0,
        load_01: 0.6,
        oil_temp_c: 100.0,
        coolant_temp_c: 88.0,
        film_0_1: 0.8,
        ring_pressure_pa: 5e6,
        bearing_pressure_pa: 5e6,
    };
    let laws = (
        wear::WearLaw::archard_ring(),
        wear::WearLaw::bearing_starvation(),
        wear::WearLaw::oil_aging(),
    );
    for film in [0.0, 0.8, 1.0] {
        for temp in [40.0, 100.0, 130.0] {
            let op = wear::OperatingPoint {
                film_0_1: film,
                oil_temp_c: temp,
                ..nominal
            };
            let mut a = wear::WearState::new_seeded(42);
            let mut b = a.clone();
            a.advance(50.0, &op, &laws.0, &laws.1, &laws.2).unwrap();
            for _ in 0..5000 {
                b.advance(0.01, &op, &laws.0, &laws.1, &laws.2).unwrap();
            }
            let gap = a.ring_gap_m[0] - 0.3e-3;
            emit(
                "wear_grid",
                json!({"film":film,"temp":temp,"gap_m":gap,"bearing_m":a.bearing_extra_m[0],
            "oxidation":a.oil_oxidation_01,"gap_subdivision_diff":a.ring_gap_m[0]-b.ring_gap_m[0]}),
            );
        }
    }
    let mut s = wear::WearState::new_seeded(1);
    let u = wear::WearLaw::unsupported("unsupported", "no equation");
    let r = s.advance(1.0, &nominal, &laws.0, &u, &u);
    emit(
        "wear_unsupported",
        json!({"accepted":r.is_ok(),"oxidation":s.oil_oxidation_01,"bearing":s.bearing_extra_m[0]}),
    );
    let mut s = wear::WearState::new_seeded(1);
    s.gasket_relax_01 = 0.1;
    let op = wear::OperatingPoint {
        coolant_temp_c: -20.0,
        ..nominal
    };
    let r = s.advance(1.0, &op, &laws.0, &laws.1, &laws.2);
    emit(
        "wear_irreversible",
        json!({"accepted":r.is_ok(),"before":0.1,"after":s.gasket_relax_01}),
    );
    let mut s = wear::WearState::new_seeded(42);
    s.advance(1.0, &nominal, &laws.0, &laws.1, &laws.2).unwrap();
    let parsed: wear::WearState = serde_json::from_str(&engine_types::canonical_json(&s)).unwrap();
    emit(
        "wear_roundtrip",
        json!({"exact":s==parsed,"hash_match":s.hash()==parsed.hash(),
        "before":engine_types::canonical_json(&s),"after":engine_types::canonical_json(&parsed)}),
    );
    for torque in [60.0, 80.0, 400.0] {
        let result = damage::bolt_yield_state(
            torque,
            0.01,
            58e-6,
            fastener::preload::LubricantCondition::OiledGeneric,
        )
        .unwrap();
        emit(
            "damage_bolt",
            json!({"torque":torque,"state":format!("{:?}",result.0),"domain":result.1.domain}),
        );
    }
    for lift in [1e-3, 8e-3] {
        let (c, r) = damage::piston_valve_state(contacts_input(lift)).unwrap();
        emit(
            "damage_geometry",
            json!({"lift":lift,"overlap":c.overlap,"state":format!("{r:?}")}),
        );
    }
    emit(
        "damage_support",
        json!({"thread":format!("{:?}",damage::thread_state().0.support),
        "starved":format!("{:?}",damage::bearing_starvation_risk(0.2,5e6).unwrap())}),
    );
}
fn contacts_input(lift: f64) -> contacts::interference::PistonValveInput {
    contacts::interference::PistonValveInput {
        closed_gap_m: 0.003,
        lift_m: lift,
        cant_rad: 0.0,
        valve_dia_m: 0.03,
        piston_x_m: 0.0005,
    }
}
fn electronics_probes() {
    let period = 1.0 / 150.0;
    let full: Vec<f64> = (0..96).map(|n| n as f64 * period).collect();
    // Different acquisition start indices, identical true CYP and CKP thereafter.
    for start in [0, 5] {
        let d = ecu::Decoder::new(
            full[start..].to_vec(),
            vec![0.16, 0.32, 0.48],
            *full.last().unwrap(),
        )
        .unwrap();
        let e = d.estimate(*full.last().unwrap());
        emit(
            "decoder_origin",
            json!({"start":start,"sync":format!("{:?}",e.state),"phase":e.cycle_deg_at,"rpm":e.rpm}),
        );
    }
    // A pulse lost in a previous cycle should not permanently shift phase after a new CYP.
    let mut skipped = full.clone();
    skipped.remove(10);
    for (label, ckp) in [("healthy", full.clone()), ("recovered_after_cyp", skipped)] {
        let d = ecu::Decoder::new(ckp, vec![0.0, 0.16, 0.32, 0.48], *full.last().unwrap()).unwrap();
        let e = d.estimate(*full.last().unwrap());
        emit(
            "decoder_recovery",
            json!({"case":label,"sync":format!("{:?}",e.state),"phase":e.cycle_deg_at,"confidence":e.confidence}),
        );
    }
    let d = ecu::Decoder::new(full.clone(), vec![0.0], *full.last().unwrap()).unwrap();
    emit(
        "decoder_cyp_dropout",
        json!({"sync_after_three_cycles_no_cyp":format!("{:?}",d.estimate(*full.last().unwrap()).state)}),
    );
    emit(
        "decoder_invalid",
        json!({"nan_accepted":ecu::Decoder::new(vec![0.0,f64::NAN],vec![],1.0).is_ok(),
        "future_accepted":ecu::Decoder::new(vec![1.0,2.0],vec![1.0],0.0).is_ok()}),
    );
    for seed in [1, 2] {
        let mut w = electrical::Wire::new(
            0.0,
            1e6,
            5.0,
            electrical::WireFault::Intermittent {
                seed,
                cut_probability: 0.5,
            },
        )
        .unwrap();
        emit(
            "wire_seed",
            json!({"seed":seed,"sequence":(0..32).map(|_|w.voltage_at_adc(2.0)).collect::<Vec<_>>()}),
        );
    }
    let mut d = ecu::Decoder::new(
        full.clone(),
        vec![0.0, 0.16, 0.32, 0.48],
        *full.last().unwrap(),
    )
    .unwrap();
    let ecu = ecu::Ecu::new(
        ecu::ReferenceMaps::illustrative().unwrap(),
        1e-3,
        13.5,
        300000.0,
    )
    .unwrap();
    let healthy = ecu.schedule(
        &d.estimate(*full.last().unwrap()),
        0.6,
        &ecu::DtcSet::default(),
    );
    d.update(2.0);
    let lost = ecu.schedule(&d.estimate(2.0), 0.6, &ecu::DtcSet::default());
    emit(
        "scheduler",
        json!({"healthy_ignitions":healthy.ignitions.len(),"lost_ignitions":lost.ignitions.len(),
        "requested":healthy.ignitions[0].requested_btdc_deg,"effective":healthy.ignitions[0].effective_btdc_deg}),
    );
}
fn analytic_probes() {
    let base = idle();
    let cycle = thermo::ClosedCycle::new(base.geom.clone(), base.mix, 101325.0, 350.0).unwrap();
    let vref = base.geom.volume(PI);
    for step in [2.0, 1.0, 0.5, 0.25] {
        let (t, a) = cycle.run(step);
        // Integral of dU=-p dV implies p V^gamma constant; no call to analytic_p.
        let err = t
            .iter()
            .map(|x| {
                (x.p_pa * x.v_m3.powf(base.mix.gamma()) / (101325.0 * vref.powf(base.mix.gamma()))
                    - 1.0)
                    .abs()
            })
            .fold(0.0, f64::max);
        emit(
            "analytic_cylinder",
            json!({"step":step,"invariant_relative_error":err,"work_net_j":a.w_net_j,"points":t.len()}),
        );
    }
    let m = flow::mass_flow(200000.0, 300.0, 50000.0, 1e-4, 287.0, 1.4).unwrap();
    let m2 = flow::mass_flow(400000.0, 300.0, 100000.0, 2e-4, 287.0, 1.4).unwrap();
    let mt = flow::mass_flow(200000.0, 1200.0, 50000.0, 1e-4, 287.0, 1.4).unwrap();
    let weak = flow::mass_flow(101325.0, 300.0, 101324.0, 1e-4, 287.0, 1.4).unwrap();
    let bernoulli = 1e-4 * (2.0_f64 * 101325.0 / (287.0 * 300.0)).sqrt();
    emit(
        "flow_metamorphic",
        json!({"pressure_area_scaling_ratio":m2/m,"temperature_scaling_ratio":mt/m,
        "bernoulli_relative_error":(weak/bernoulli-1.0).abs()}),
    );
    let mut tp = thermal::ThermalParams::illustrative();
    tp.c_wall = 0.0;
    let mut st = thermal::ThermalState::cold(20.0);
    let r = st.step(1.0, 1.0, 0.0, 20.0, &tp);
    emit(
        "thermal_invalid",
        json!({"zero_capacity_accepted":r.is_ok(),"wall_finite":st.t_wall_c.is_finite()}),
    );
    let tp = thermal::ThermalParams::illustrative();
    let mut hot = thermal::ThermalState::cold(100.0);
    let u0 = hot.stored_j(&tp);
    let loss = (tp.ha_wall_amb + tp.ha_oil_amb + tp.ha_rad_full) * 80.0;
    hot.step(0.1, 0.0, 0.0, 20.0, &tp).unwrap();
    emit(
        "thermal_boundary",
        json!({"delta_u_j":hot.stored_j(&tp)-u0,"expected_loss_j":loss*0.1,
        "balanced_j":hot.stored_j(&tp)-u0+loss*0.1}),
    );
    let bad = dyno::Correction::of(
        &dyno::Ambient {
            p_kpa: 99.0,
            t_c: -300.0,
        },
        true,
    );
    emit(
        "dyno_invalid",
        json!({"below_absolute_zero_accepted":bad.is_ok(),"factor_finite":bad.map(|x|x.factor.is_finite()).unwrap_or(true)}),
    );
    use calibration::{DataSource, Manifest, Observation, Trial};
    let trial = |id: &str, motor: &str| Trial {
        id: id.into(),
        motor_id: motor.into(),
        campaign: "".into(),
        source: DataSource::Real {
            provenance: "x".into(),
        },
        observations: vec![Observation {
            quantity: "".into(),
            value: f64::NAN,
            uncertainty_k1: -1.0,
            unit: "".into(),
        }],
    };
    let m = Manifest {
        train: vec![trial("T", "M1")],
        holdout: vec![trial("H", "M2")],
    };
    emit(
        "dataset_invalid",
        json!({"invalid_observations_accepted":m.validated().is_ok()}),
    );
    // An independent one-parameter normal matrix: J=2p, sigma_p=1/(2p).
    for start in [3.0, 6.0, 2.0] {
        let fit = calibration::least_squares(|p| vec![p[0] * p[0]], &[4.0], &[1.0], &[start], 1e-9);
        match fit {
            Ok(f) => emit(
                "fit_analytic",
                json!({"start":start,"ok":true,"param":f.params[0],
                "std_reported":f.std_k1[0],"std_independent":1.0/(2.0*f.params[0]),
                "residual":f.residual_norm}),
            ),
            Err(e) => emit(
                "fit_analytic",
                json!({"start":start,"ok":false,"error":e.to_string()}),
            ),
        }
    }
}
fn dyno_probes() {
    let cfg = dyno::DynoConfig {
        aux_tau_n_m: 2.0,
        ambient: dyno::Ambient::standard(),
        apply_correction: false,
    };
    for load in [4.0, 6.0, 8.0] {
        let point = dyno::SweepPoint {
            label: format!("load-{load}"),
            throttle_area_m2: 0.3e-4,
            brake_tau_n_m: load,
            t_end_s: 6.0,
        };
        let man = dyno::run_sweep(&cfg, &[point], "independent audit numerical fixture").unwrap();
        emit("dyno_public", json!({"load":load,"result":man.results[0]}));
        for every in [400, 40, 4, 1] {
            let mut i = engine::healthy_inputs();
            i.throttle_area_m2 = 0.3e-4;
            i.accessory_tau_n_m = load + 2.0;
            i.record_every = every;
            i.t_end_s = 6.0;
            let t = engine::run_engine(&i).unwrap();
            let tail = &t.samples[t.samples.len() * 4 / 5..];
            let n = tail.len() as f64;
            let om = |s: &engine::TraceSample| s.rpm * PI / 30.0;
            let ti = tail.iter().map(|s| s.tau_ind_n_m).sum::<f64>() / n;
            let tf = tail.iter().map(|s| s.tau_fric_n_m).sum::<f64>() / n;
            let p_i = tail.iter().map(|s| s.tau_ind_n_m * om(s)).sum::<f64>() / n;
            let p_f = tail.iter().map(|s| s.tau_fric_n_m * om(s)).sum::<f64>() / n;
            let omega = tail.iter().map(om).sum::<f64>() / n;
            let j = crank::effective_inertia_mean(&i.geom, i.inertia);
            let a = tail.first().unwrap();
            let b = tail.last().unwrap();
            let p_acc_tail = 0.5 * j * (om(b).powi(2) - om(a).powi(2)) / (b.t_s - a.t_s);
            emit(
                "dyno_sampling",
                json!({"load":load,"every":every,"samples_tail":tail.len(),"rpm_mean":omega*30.0/PI,
                "tail_torque_resid":(ti-tf-load-2.0).abs()/ti.abs(),
                "p_ind_tau_omega_w":p_i,"p_ind_mean_product_w":ti*omega,"p_fric_w":p_f,
                "p_load_w":(load+2.0)*omega,"p_acc_tail_w":p_acc_tail,
                "p_acc_global_w":t.audit.delta_ke_j/6.0,
                "global_mech_resid":(t.audit.w_ind_j+t.audit.w_starter_j-t.audit.w_fric_j-t.audit.w_load_j-t.audit.delta_ke_j).abs()/t.audit.w_ind_j.abs()}),
            );
        }
    }
}
fn main() {
    // Panics are caught only for negative controls; suppress hook noise in JSON output.
    std::panic::set_hook(Box::new(|_| {}));
    let suite = std::env::args().nth(1).unwrap_or_else(|| "all".into());
    if suite == "all" || suite == "engine" {
        engine_probes();
        cam_probes();
        replay_probes();
    }
    if suite == "all" || suite == "subsystems" {
        leak_probes();
        wear_probes();
        electronics_probes();
        analytic_probes();
    }
    if suite == "all" || suite == "dyno" {
        dyno_probes();
    }
}
