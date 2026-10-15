//! Independent remediation verification. Numerical fixtures, not experimental data.
use engine::{CylState, EngineInputs, EngineSnapshot, EngineState, WorkState};
use serde_json::{json, Value};
use std::f64::consts::PI;
include!("common.rs");
include!("observed_cli.rs");
fn emit(id: &str, v: Value) {
    println!("{}", json!({"probe":id,"data":v}));
}
fn stored(w: &WorkState) -> f64 {
    w.plen.fuel + w.exh.fuel + w.cyls.iter().map(|c| c.fuel).sum::<f64>()
}
fn inventory(w: &WorkState) -> Vec<f64> {
    let mut v = w.cyls.iter().map(|c| c.fuel / c.m).collect::<Vec<_>>();
    v.extend([w.plen.fuel / w.plen.m, w.exh.fuel / w.exh.m]);
    v
}
fn engine_probes() {
    for dt in [100e-6, 50e-6, 25e-6] {
        for (fuel, spark, marked) in [
            (true, false, false),
            (false, false, false),
            (false, false, true),
            (true, true, false),
        ] {
            let mut i = engine::healthy_inputs();
            i.dt_s = dt;
            i.t_end_s = 1.;
            i.record_every = 1;
            i.control.fuel_enabled = fuel;
            i.control.spark_enabled = spark;
            let mut w = ws(&i);
            if marked {
                for c in &mut w.cyls {
                    c.fuel = 0.02 * c.m;
                }
                w.plen.fuel = 0.1 * w.plen.m;
                w.exh.fuel = 0.05 * w.exh.m;
            }
            let f0 = stored(&w);
            let mut max_ratio = 0f64;
            let mut max_closure = 0f64;
            let mut max_chem_excess = 0f64;
            let mut excess_time = 0f64;
            let mut witness = json!({});
            let mut failed = None;
            for n in 1..=2000 {
                if let Err(e) = engine::resume(&mut w, &i, n as f64 * 0.0005) {
                    failed = Some(e.to_string());
                    break;
                }
                max_ratio = max_ratio.max(inventory(&w).into_iter().fold(0., f64::max));
                max_closure = max_closure.max(
                    (f0 + w.fuel_injected_kg - stored(&w) - w.fuel_burned_kg - w.fuel_emitted_kg)
                        .abs(),
                );
                let excess = w.heat_released_j
                    - w.fuel_burned_kg * i.fuel.lhv_j_per_kg * i.combustion_efficiency;
                if excess > max_chem_excess {
                    max_chem_excess = excess;
                    excess_time = w.time_s;
                    witness = json!({"Q":w.heat_released_j,"burned":w.fuel_burned_kg,"stored":stored(&w),"injected":w.fuel_injected_kg,"emitted":w.fuel_emitted_kg});
                }
            }
            emit(
                "engine_species",
                json!({"dt":dt,"fuel":fuel,"spark":spark,"marked":marked,"error":failed,"closure_kg":max_closure,"max_fuel_fraction":max_ratio,"max_chemical_excess_j":max_chem_excess,"witness_time":excess_time,"witness":witness,"final_rpm":w.omega_rad_s*30./PI,"final_stored":stored(&w),"emitted":w.fuel_emitted_kg}),
            );
        }
    }
    // Native replay: every physical state field and every per-step trace sample.
    for thermal in [false, true] {
        for c2 in [0., 0.01, 0.05] {
            for cut in [0.173, 0.5, 0.913] {
                let mut i = engine::healthy_inputs();
                i.record_every = 1;
                i.dt_s = 50e-6;
                if thermal {
                    i.thermal = Some(thermal::ThermalParams::illustrative());
                }
                if let crank::FrictionModel::ChenFlynn { ref mut coeffs } = i.friction {
                    coeffs.c2_dlless = c2;
                }
                for horizon in [1.2, 2.] {
                    let mut full = ws(&i);
                    let all = engine::resume(&mut full, &i, horizon).unwrap();
                    let mut split = ws(&i);
                    let mut parts = engine::resume(&mut split, &i, cut).unwrap();
                    let pre_q = split.th_q_amb_j;
                    let snap = split.snapshot();
                    let text = serde_json::to_string(&snap).unwrap();
                    let parsed: EngineSnapshot = serde_json::from_str(&text).unwrap();
                    let mut restored = WorkState::restore(&parsed, &i).unwrap();
                    let lost_q = pre_q - restored.th_q_amb_j;
                    parts.extend(engine::resume(&mut restored, &i, horizon).unwrap());
                    emit(
                        "native_replay",
                        json!({"thermal":thermal,"c2":c2,"cut":cut,"horizon":horizon,"hash_equal":full.snapshot().state_hash==restored.snapshot().state_hash,"state_equal":full.snapshot().state==restored.snapshot().state,"trace_equal":all==parts,"trace_lengths":[all.len(),parts.len()],"restored_pmax":snap.state.pmax_pa,"ledger_q_lost_j":lost_q}),
                    );
                }
            }
        }
    }
    // Multiple serialized cuts and independent fuel/spark controls.
    for thermal in [false, true] {
        for c2 in [0., 0.01, 0.05] {
            for mode in ["healthy", "no_fuel", "no_spark"] {
                let mut i = engine::healthy_inputs();
                i.record_every = 1;
                if thermal {
                    i.thermal = Some(thermal::ThermalParams::illustrative());
                }
                i.control.fuel_enabled = mode != "no_fuel";
                i.control.spark_enabled = mode != "no_spark";
                if let crank::FrictionModel::ChenFlynn { ref mut coeffs } = i.friction {
                    coeffs.c2_dlless = c2;
                }
                let mut full = ws(&i);
                let trace = engine::resume(&mut full, &i, 1.5).unwrap();
                let mut split = ws(&i);
                let mut parts = Vec::new();
                for cut in [0.173, 0.501, 0.913] {
                    parts.extend(engine::resume(&mut split, &i, cut).unwrap());
                    let text = serde_json::to_string(&split.snapshot()).unwrap();
                    let snap: EngineSnapshot = serde_json::from_str(&text).unwrap();
                    split = WorkState::restore(&snap, &i).unwrap();
                }
                parts.extend(engine::resume(&mut split, &i, 1.5).unwrap());
                emit(
                    "native_multiple_cuts",
                    json!({"thermal":thermal,"c2":c2,"mode":mode,"cuts":[0.173,0.501,0.913],"hash_equal":full.snapshot().state_hash==split.snapshot().state_hash,"state_equal":full.snapshot().state==split.snapshot().state,"trace_equal":trace==parts,"trace_length":trace.len()}),
                );
            }
        }
    }
    // Identical mass/energy, mixed marked species, reverse pressure gradients, simultaneous valve paths.
    for high in ["plenum", "exhaust", "cylinder"] {
        for dt in [100e-6, 50e-6, 25e-6] {
            let mut i = engine::healthy_inputs();
            i.dt_s = dt;
            i.control.fuel_enabled = false;
            i.control.spark_enabled = false;
            i.starter = None;
            let mut w = ws(&i);
            w.omega_rad_s = 100.;
            w.theta_rad = 0.;
            for c in &mut w.cyls {
                c.fuel = 0.01 * c.m;
            }
            w.plen.fuel = 0.03 * w.plen.m;
            w.exh.fuel = 0.06 * w.exh.m;
            match high {
                "plenum" => {
                    w.plen.m *= 2.;
                    w.plen.fuel *= 2.;
                }
                "exhaust" => {
                    w.exh.m *= 2.;
                    w.exh.fuel *= 2.;
                }
                _ => {
                    for c in &mut w.cyls {
                        c.m *= 2.;
                        c.u *= 2.;
                        c.fuel *= 2.;
                    }
                }
            }
            let f0 = stored(&w);
            let m0 = w.plen.m + w.exh.m + w.cyls.iter().map(|c| c.m).sum::<f64>();
            let result = engine::resume(&mut w, &i, 0.03);
            let m1 = w.plen.m + w.exh.m + w.cyls.iter().map(|c| c.m).sum::<f64>();
            emit(
                "reverse_fuel",
                json!({"high":high,"dt":dt,"ok":result.is_ok(),"error":result.err().map(|e|e.to_string()),"fuel_closure_kg":f0-stored(&w)-w.fuel_emitted_kg,"mass_closure_kg":m1-m0-w.m_in_amb_kg+w.m_out_amb_kg,"fractions":inventory(&w),"plen_fuel":w.plen.fuel,"exh_fuel":w.exh.fuel,"emitted":w.fuel_emitted_kg}),
            );
        }
    }
}
fn gas(p: f64, fresh: f64) -> leak::LeakDomain {
    leak::LeakDomain {
        v_m3: 0.01,
        m_kg: p * 0.01 / (287. * 300.),
        t_k: 300.,
        fluid: leak::Fluid::Gas {
            r_gas: 287.,
            cp: 1005.,
        },
        fresh_frac: fresh,
    }
}
fn leak_probes() {
    for reverse in [false, true] {
        for dt in [0.002, 0.001, 0.0005] {
            for (fa, fb) in [(0.0, 1.0), (0.2, 0.7), (1.0, 0.0)] {
                // Isolated regulator node has no ports: closed two-node transport, independent species invariant.
                let domains = vec![
                    gas(200000., 1.),
                    gas(if reverse { 100000. } else { 180000. }, fa),
                    gas(if reverse { 180000. } else { 100000. }, fb),
                ];
                let net = leak::LeakNetwork {
                    domains: domains.clone(),
                    p_set_pa: vec![0.; 3],
                    ports: vec![
                        leak::LeakPort {
                            from: 1,
                            to: 2,
                            conductance: 1e-7,
                            compressible: true,
                        },
                        leak::LeakPort {
                            from: 2,
                            to: 1,
                            conductance: 1e-7,
                            compressible: true,
                        },
                    ],
                    mu_liquid_pa_s: 0.02,
                    supply_t_k: 300.,
                };
                let (end, a, totals) = leak::run_leakdown(&net, 0, 0.02, dt).unwrap();
                let initial = domains.iter().map(|d| d.m_kg * d.fresh_frac).sum::<f64>();
                let final_fresh = end.iter().map(|d| d.m_kg * d.fresh_frac).sum::<f64>();
                // During this short pulse donor fraction is unchanged, and opposite edge inactive.
                let (donor, receiver, moved) = if reverse {
                    (2, 1, totals[1])
                } else {
                    (1, 2, totals[0])
                };
                let expected = (domains[receiver].m_kg * domains[receiver].fresh_frac
                    + domains[donor].fresh_frac * moved)
                    / (domains[receiver].m_kg + moved);
                emit(
                    "fresh_transport",
                    json!({"reverse":reverse,"dt":dt,"fa":fa,"fb":fb,"closure_kg":final_fresh-initial,"ledger":a.fresh_closure,"receiver":end[receiver].fresh_frac,"expected":expected,"totals":totals}),
                );
            }
        }
    }
    for reverse in [false, true] {
        for dt in [0.001, 0.0005] {
            let domains = vec![
                gas(200000., 0.2),
                gas(if reverse { 300000. } else { 100000. }, 0.7),
            ];
            let net = leak::LeakNetwork {
                domains,
                p_set_pa: vec![0.; 2],
                ports: vec![
                    leak::LeakPort {
                        from: 0,
                        to: 1,
                        conductance: 1e-7,
                        compressible: true,
                    },
                    leak::LeakPort {
                        from: 1,
                        to: 0,
                        conductance: 1e-7,
                        compressible: true,
                    },
                ],
                mu_liquid_pa_s: 0.02,
                supply_t_k: 300.,
            };
            let (end, a, totals) = leak::run_leakdown(&net, 0, 0.01, dt).unwrap();
            emit(
                "fresh_regulator",
                json!({"reverse":reverse,"dt":dt,"ledger":a.fresh_closure,"mass":a.mass_closure,"fractions":end.iter().map(|d|d.fresh_frac).collect::<Vec<_>>(),"totals":totals}),
            );
        }
    }
    for fluid in [leak::Fluid::Oil, leak::Fluid::Coolant] {
        for reverse in [false, true] {
            for dt in [0.001, 0.0005] {
                let rho = fluid.liquid_density_kg_m3().unwrap();
                let domains = vec![
                    gas(200000., 1.),
                    leak::LeakDomain {
                        v_m3: 0.001,
                        m_kg: 1.,
                        t_k: 300.,
                        fluid,
                        fresh_frac: 1.,
                    },
                    leak::LeakDomain {
                        v_m3: 0.001,
                        m_kg: 1.,
                        t_k: 300.,
                        fluid,
                        fresh_frac: 1.,
                    },
                ];
                let (from, to) = if reverse { (2, 1) } else { (1, 2) };
                let mut ps = vec![0., 100000., 100000.];
                ps[from] = 300000.;
                let net = leak::LeakNetwork {
                    domains,
                    p_set_pa: ps,
                    ports: vec![leak::LeakPort {
                        from,
                        to,
                        conductance: 1e-13,
                        compressible: false,
                    }],
                    mu_liquid_pa_s: 0.02,
                    supply_t_k: 300.,
                };
                let (end, a, moved) = leak::run_leakdown(&net, 0, 0.01, dt).unwrap();
                let expected_volume = 1e-13 * 200000. / 0.02 * 0.01;
                emit(
                    "liquid_units",
                    json!({"fluid":format!("{fluid:?}"),"rho":rho,"reverse":reverse,"dt":dt,"mass":moved[0],"expected_mass":rho*expected_volume,"volume":moved[0]/rho,"expected_volume":expected_volume,"mass_closure":a.mass_closure,"energy_closure":a.energy_closure,"donor_m":end[from].m_kg}),
                );
            }
        }
    }
    // Production function copied byte-exact; actual solver supplies port totals.
    for exhaust in [false, true] {
        for flows in [
            vec![0.001, 0.002, 0.007],
            vec![0., 0., 0.01],
            vec![0., 0., 0.],
        ] {
            let net = leak::LeakNetwork {
                domains: vec![
                    gas(500000., 1.),
                    gas(100000., 0.),
                    gas(100000., 0.),
                    gas(100000., 0.),
                ],
                p_set_pa: vec![0.; 4],
                ports: vec![
                    leak::LeakPort {
                        from: 0,
                        to: 1,
                        conductance: 1e-7,
                        compressible: true,
                    },
                    leak::LeakPort {
                        from: 0,
                        to: 2,
                        conductance: 1e-7,
                        compressible: true,
                    },
                ]
                .into_iter()
                .chain(if exhaust {
                    Some(leak::LeakPort {
                        from: 0,
                        to: 3,
                        conductance: 1e-7,
                        compressible: true,
                    })
                } else {
                    None
                })
                .collect(),
                mu_liquid_pa_s: 0.02,
                supply_t_k: 300.,
            };
            let actual_totals = if exhaust {
                flows.clone()
            } else {
                flows[..2].to_vec()
            };
            let (s, flag) = observable_chamber_split(&net, &actual_totals);
            let verdict =
                diagnostics::diagnose_low_compression(Some(500.), 900., Some(30.), Some(s));
            emit(
                "diagnostic_channels",
                json!({"exhaust_port":exhaust,"flows":actual_totals,"split":[s.to_crankcase_01,s.to_exhaust_01,s.to_coolant_01],"exhaust_flag":flag,"verdict":format!("{verdict:?}")}),
            );
        }
    }
    let net = leak::LeakNetwork {
        domains: vec![
            gas(500000., 1.),
            gas(100000., 0.),
            gas(100000., 0.),
            gas(100000., 0.),
        ],
        p_set_pa: vec![0.; 4],
        ports: vec![
            leak::LeakPort {
                from: 0,
                to: 1,
                conductance: 1e-8,
                compressible: true,
            },
            leak::LeakPort {
                from: 0,
                to: 2,
                conductance: 2e-8,
                compressible: true,
            },
            leak::LeakPort {
                from: 0,
                to: 3,
                conductance: 7e-8,
                compressible: true,
            },
        ],
        mu_liquid_pa_s: 0.02,
        supply_t_k: 300.,
    };
    let (_, _, totals) = leak::run_leakdown(&net, 0, 0.01, 0.0005).unwrap();
    let (s, flag) = observable_chamber_split(&net, &totals);
    let total: f64 = totals.iter().sum();
    emit(
        "diagnostic_physical_exhaust",
        json!({"totals":totals,"reported":[s.to_crankcase_01,s.to_exhaust_01,s.to_coolant_01],"expected":totals.iter().map(|m|m/total).collect::<Vec<_>>(),"exhaust_flag":flag}),
    );
}
fn thermal_probes() {
    let p = thermal::ThermalParams::illustrative();
    for initial in [0., 20., 84., 100.] {
        for amb in [20., 120.] {
            for dt in [0.1, 0.05, 0.025] {
                let mut s = thermal::ThermalState::cold(initial);
                let u0 = s.stored_j(&p);
                let mut qsum = 0.;
                let mut local_error = 0f64;
                for _ in 0..(2. / dt) as usize {
                    let pre = s;
                    let u = pre.stored_j(&p);
                    let q = s.step(dt, 10. * dt, 3., amb, &p).unwrap();
                    qsum += q;
                    local_error = local_error.max((s.stored_j(&p) - u - 13. * dt + q).abs());
                }
                emit(
                    "thermal_ledger",
                    json!({"initial":initial,"ambient":amb,"dt":dt,"ambient_q_j":qsum,"closure_j":s.stored_j(&p)-u0-26.+qsum,"max_local_error_j":local_error,"final_wall":s.t_wall_c,"final_coolant":s.t_cool_c}),
                );
            }
        }
    }
    for mode in ["equilibrium", "adiabatic"] {
        for dt in [0.1, 0.05, 0.025] {
            let mut p = thermal::ThermalParams::illustrative();
            if mode == "adiabatic" {
                p.ha_piston_wall = 0.;
                p.ha_wall_cool = 0.;
                p.ha_wall_oil = 0.;
                p.ha_wall_amb = 0.;
                p.ha_oil_amb = 0.;
                p.ha_rad_full = 0.;
            }
            let mut s = thermal::ThermalState::cold(if mode == "equilibrium" { 20. } else { 100. });
            let u0 = s.stored_j(&p);
            let mut total = 0.;
            let mut supplied = 0.;
            for _ in 0..(2. / dt) as usize {
                let (qb, pf) = if mode == "equilibrium" {
                    (0., 0.)
                } else {
                    (10. * dt, 3.)
                };
                total += s.step(dt, qb, pf, 20., &p).unwrap();
                supplied += qb + pf * dt;
            }
            emit(
                "thermal_zero_boundary",
                json!({"mode":mode,"dt":dt,"ambient_q_j":total,"closure_j":s.stored_j(&p)-u0-supplied}),
            );
        }
    }
    // Independent exponential solution: isolated wall coupled only to ambient.
    for dt in [0.1, 0.05, 0.025] {
        let mut p = thermal::ThermalParams::illustrative();
        p.ha_piston_wall = 0.;
        p.ha_wall_cool = 0.;
        p.ha_wall_oil = 0.;
        p.ha_oil_amb = 0.;
        p.ha_rad_full = 0.;
        let mut s = thermal::ThermalState::cold(100.);
        let n = (10. / dt) as usize;
        for _ in 0..n {
            s.step(dt, 0., 0., 20., &p).unwrap();
        }
        let exact = 20. + 80. * (-p.ha_wall_amb * 10. / p.c_wall).exp();
        emit(
            "thermal_analytic",
            json!({"dt":dt,"error_c":(s.t_wall_c-exact).abs(),"observed":s.t_wall_c,"exact":exact}),
        );
    }
}
fn snapshot_probes() {
    let i = engine::healthy_inputs();
    let mut w = ws(&i);
    engine::resume(&mut w, &i, 0.5).unwrap();
    for option in [false, true] {
        for x in [
            f64::from_bits(0x3ff0000000000001),
            1.2345678901234567,
            293.15000000000003,
            1e-200,
            1e100,
        ] {
            let mut s = w.snapshot().state;
            s.time_s = x;
            if !option {
                s.pmax_pa = None;
                s.plen_fuel_kg = 0.;
                s.exh_fuel_kg = 0.;
                s.fuel_emitted_kg = 0.;
            }
            let snap = EngineSnapshot::capture(&s);
            let text = serde_json::to_string(&snap).unwrap();
            let restored: EngineSnapshot = serde_json::from_str(&text).unwrap();
            emit(
                "snapshot_bits",
                json!({"x":x,"optionals":option,"bits_equal":s.time_s.to_bits()==restored.state.time_s.to_bits(),"hash_equal":snap.state_hash==restored.state_hash,"verified":restored.verified(),"state_equal":s==restored.state}),
            );
        }
    }
    let mut ti = i.clone();
    ti.thermal = Some(thermal::ThermalParams::illustrative());
    let mut physical = ws(&ti).snapshot().state;
    physical.time_s = 1.2345678901234567;
    physical.plen_fuel_kg = 1.2345678901234567e-6;
    physical.exh_fuel_kg = 1e-200;
    physical.fuel_emitted_kg = 1.2345678901234567e-5;
    physical.cyls[0].fuel_kg = 1.2345678901234567e-8;
    physical.thermal.as_mut().unwrap().t_wall_c = 20.123456789012347;
    physical.pmax_pa = Some(123456.78901234567);
    let snap = EngineSnapshot::capture(&physical);
    let rt: EngineSnapshot = serde_json::from_str(&serde_json::to_string(&snap).unwrap()).unwrap();
    fn bit_fields(a: &Value, b: &Value) -> (usize, bool) {
        match (a, b) {
            (Value::Number(x), Value::Number(y)) if x.is_f64() => (
                1,
                x.as_f64().unwrap().to_bits() == y.as_f64().unwrap().to_bits(),
            ),
            (Value::Array(x), Value::Array(y)) => x
                .iter()
                .zip(y)
                .map(|(a, b)| bit_fields(a, b))
                .fold((0, true), |(n, p), (m, q)| (n + m, p && q)),
            (Value::Object(x), Value::Object(y)) => x
                .iter()
                .map(|(k, a)| bit_fields(a, &y[k]))
                .fold((0, true), |(n, p), (m, q)| (n + m, p && q)),
            _ => (0, true),
        }
    }
    let (fields, bits) = bit_fields(
        &serde_json::to_value(&snap.state).unwrap(),
        &serde_json::to_value(&rt.state).unwrap(),
    );
    emit(
        "snapshot_full_physical",
        json!({"float_fields":fields,"all_bits_equal":bits,"hash_equal":snap.state_hash==rt.state_hash,"restore_accepted":WorkState::restore(&rt,&ti).is_ok(),"thermal":true}),
    );
    for field in ["plen_fuel_kg", "exh_fuel_kg", "fuel_emitted_kg"] {
        let mut s = w.snapshot().state;
        match field {
            "plen_fuel_kg" => s.plen_fuel_kg = -0.,
            "exh_fuel_kg" => s.exh_fuel_kg = -0.,
            _ => s.fuel_emitted_kg = -0.,
        };
        let snap = EngineSnapshot::capture(&s);
        let text = serde_json::to_string(&snap).unwrap();
        let rt: EngineSnapshot = serde_json::from_str(&text).unwrap();
        let after = match field {
            "plen_fuel_kg" => rt.state.plen_fuel_kg,
            "exh_fuel_kg" => rt.state.exh_fuel_kg,
            _ => rt.state.fuel_emitted_kg,
        };
        emit(
            "snapshot_signed_zero",
            json!({"field":field,"before_bits":(-0f64).to_bits().to_string(),"after_bits":after.to_bits().to_string(),"hash_equal":snap.state_hash==rt.state_hash,"restore_accepted":WorkState::restore(&snap,&i).is_ok()}),
        );
    }
    let mut wear = wear::WearState::new_seeded(4);
    wear.hours = 1.2345678901234567;
    wear.ring_gap_m[0] = 0.00030000000000000003;
    let text = serde_json::to_string(&wear).unwrap();
    let back: wear::WearState = serde_json::from_str(&text).unwrap();
    emit(
        "wear_roundtrip",
        json!({"equal":wear==back,"hours_bits":wear.hours.to_bits()==back.hours.to_bits(),"gap_bits":wear.ring_gap_m[0].to_bits()==back.ring_gap_m[0].to_bits()}),
    );
    // Deliberately remove newly persisted history from a native running state. This is an information-loss control, not a historical-v1 fixture.
    let mut params = i.clone();
    if let crank::FrictionModel::ChenFlynn { ref mut coeffs } = params.friction {
        coeffs.c2_dlless = 0.05;
    }
    let mut native = ws(&params);
    engine::resume(&mut native, &params, 0.5).unwrap();
    let snap = native.snapshot();
    let mut absent = snap.state.clone();
    absent.pmax_pa = None;
    absent.plen_fuel_kg = 0.;
    absent.exh_fuel_kg = 0.;
    absent.fuel_emitted_kg = 0.;
    let mut missing = WorkState::restore(&EngineSnapshot::capture(&absent), &params).unwrap();
    let dp = native.pmax_pa - missing.pmax_pa;
    let df = stored(&native) - stored(&missing);
    engine::resume(&mut native, &params, 1.5).unwrap();
    engine::resume(&mut missing, &params, 1.5).unwrap();
    emit(
        "missing_history_control",
        json!({"schema":2,"lost_pmax":dp,"lost_fuel_kg":df,"native_rpm":native.omega_rad_s*30./PI,"missing_rpm":missing.omega_rad_s*30./PI,"same_hash":native.snapshot().state_hash==missing.snapshot().state_hash}),
    );
    if let Ok(text) = std::fs::read_to_string(
        std::path::Path::new(
            &std::env::var("D16_REVIEW_RESULTS")
                .unwrap_or_else(|_| "docs/remediation/review/results".into()),
        )
        .join("historical-v1-hot.json"),
    ) {
        let snap: EngineSnapshot = serde_json::from_str(&text).unwrap();
        let restored = WorkState::restore(&snap, &i);
        emit(
            "authentic_v1",
            json!({"hash_verified":snap.verified(),"accepted":restored.is_ok(),"error":restored.as_ref().err().map(|e|e.to_string()),"time":snap.state.time_s,"new_fields":[snap.state.plen_fuel_kg,snap.state.exh_fuel_kg,snap.state.fuel_emitted_kg],"pmax_present":snap.state.pmax_pa.is_some(),"migrated_schema":restored.as_ref().ok().map(|w|w.snapshot().state.schema_version)}),
        );
        if let Ok(mut hist) = restored {
            let pre = hist.snapshot();
            engine::resume(&mut hist, &i, 1.).unwrap();
            let post = hist.snapshot();
            let rt: EngineSnapshot =
                serde_json::from_str(&serde_json::to_string(&post).unwrap()).unwrap();
            emit(
                "authentic_v1_continuation",
                json!({"pre_hash":pre.state_hash,"post_hash":post.state_hash,"post_verifies":rt.verified(),"rpm":hist.omega_rad_s*30./PI}),
            );
        }
        for variant in ["bad_hash", "tampered", "unknown_schema", "missing_field"] {
            let mut s = serde_json::from_str::<Value>(&text).unwrap();
            match variant {
                "bad_hash" => s["state_hash"] = json!("bad"),
                "tampered" => s["state"]["time_s"] = json!(12.),
                "unknown_schema" => s["state"]["schema_version"] = json!(99),
                _ => {
                    s["state"].as_object_mut().unwrap().remove("cyls");
                }
            }
            let parsed = serde_json::from_value::<EngineSnapshot>(s);
            let rejected = match parsed {
                Ok(s) => WorkState::restore(&s, &i).is_err(),
                Err(_) => true,
            };
            emit(
                "legacy_negative",
                json!({"variant":variant,"rejected":rejected}),
            );
        }
    }
}
fn boundary_probes() {
    let i = engine::healthy_inputs();
    let base = ws(&i);
    for bad in ["NaN", "Inf", "below_absolute"] {
        let mut p = thermal::ThermalParams::illustrative();
        let mut s = thermal::ThermalState::cold(if bad == "NaN" {
            f64::NAN
        } else if bad == "Inf" {
            f64::INFINITY
        } else {
            -300.
        });
        let pre = format!("{s:?}");
        let e = s.step(0.1, 0., 0., 20., &p);
        emit(
            "thermal_invalid_state",
            json!({"bad":bad,"accepted":e.is_ok(),"unchanged":pre==format!("{s:?}"),"state":format!("{s:?}")}),
        );
        p.c_wall = 0.;
    }
    let p = thermal::ThermalParams::illustrative();
    let mut s = thermal::ThermalState::cold(20.);
    let e = s.step(0.1, 0., 0., -300., &p);
    emit(
        "thermal_invalid_ambient",
        json!({"accepted":e.is_ok(),"state":format!("{s:?}")}),
    );
    let mut p = thermal::ThermalParams::illustrative();
    p.c_wall = 1e-308;
    let mut s = thermal::ThermalState::cold(20.);
    let pre = format!("{s:?}");
    let e = s.step(0.1, 100., 0., 20., &p);
    emit(
        "thermal_overflow_atomic",
        json!({"rejected":e.is_err(),"unchanged":pre==format!("{s:?}"),"state":format!("{s:?}")}),
    );
    for field in [
        "thermal_abszero",
        "thermal_nan",
        "counter_nan",
        "counter_negative",
        "history_nan",
        "node_volume",
        "record_zero",
    ] {
        let mut inp = i.clone();
        let mut st = base.snapshot().state;
        match field {
            "thermal_abszero" | "thermal_nan" => {
                inp.thermal = Some(thermal::ThermalParams::illustrative());
                st.thermal = Some(engine::state::ThermalSnap {
                    t_wall_c: if field == "thermal_nan" {
                        f64::NAN
                    } else {
                        -300.
                    },
                    t_piston_c: 20.,
                    t_oil_c: 20.,
                    t_cool_c: 20.,
                    ect_filt_c: 20.,
                });
            }
            "counter_nan" => st.heat_released_j = f64::NAN,
            "counter_negative" => st.fuel_emitted_kg = -1.,
            "history_nan" => st.prev_local_deg[0] = f64::NAN,
            "node_volume" => inp.plenum_v_m3 = 0.,
            _ => inp.record_every = 0,
        }
        let snap = EngineSnapshot::capture(&st);
        let e = WorkState::restore(&snap, &inp);
        emit(
            "restore_boundary",
            json!({"field":field,"verified":snap.verified(),"accepted":e.is_ok(),"error":e.err().map(|e|e.to_string())}),
        );
    }
    for bad in [
        "negative_area",
        "bad_wiebe",
        "friction_nan",
        "bad_state",
        "counter_nan",
    ] {
        let mut inp = i.clone();
        let mut state = base.clone();
        state.m_in_amb_kg = 0.12;
        match bad {
            "negative_area" => inp.tailpipe_area_m2 = -1.,
            "bad_wiebe" => inp.wiebe_a = -1.,
            "friction_nan" => inp.friction = crank::FrictionModel::Viscous { c_n_m_s: f64::NAN },
            "bad_state" => state.cyls[0].m = -1.,
            _ => state.heat_released_j = f64::NAN,
        }
        let pre = format!("{state:?}");
        let e = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            engine::resume(&mut state, &inp, 0.001)
        }));
        emit(
            "engine_boundary_atomic",
            json!({"bad":bad,"panicked":e.is_err(),"accepted":e.as_ref().map(|r|r.is_ok()).unwrap_or(false),"unchanged":pre==format!("{state:?}"),"error":e.ok().and_then(|r|r.err()).map(|e|e.to_string())}),
        );
    }
    let amb = dyno::Ambient {
        p_kpa: 1e-308,
        t_c: 25.,
    };
    let c = dyno::Correction::of(&amb, true);
    emit(
        "dyno_overflow",
        json!({"accepted":c.is_ok(),"finite":c.as_ref().map(|r|r.factor.is_finite()).unwrap_or(false)}),
    );
    let mut d = ecu::Decoder::new(vec![0., 0.01, 0.02, 0.03], vec![], 0.03).unwrap();
    d.update(f64::NAN);
    let est = d.estimate(f64::INFINITY);
    emit(
        "decoder_boundary",
        json!({"nonfinite_estimate_time":!est.at_time_s.is_finite(),"finite_phase":est.cycle_deg_at.is_finite(),"state":format!("{:?}",est.state)}),
    );
    for field in [
        "value_nan",
        "value_inf",
        "sigma_nan",
        "sigma_negative",
        "unit_missing",
        "campaign_missing",
        "provenance_missing",
    ] {
        use calibration::{DataSource, Manifest, Observation, Trial};
        let trial = |id: &str, motor: &str| Trial {
            id: id.into(),
            motor_id: motor.into(),
            campaign: "C".into(),
            source: DataSource::Real {
                provenance: "bench/operator/date".into(),
            },
            observations: vec![Observation {
                quantity: "rpm".into(),
                value: 1200.,
                uncertainty_k1: 1.,
                unit: "rpm".into(),
            }],
        };
        let mut m = Manifest {
            train: vec![trial("T", "M1")],
            holdout: vec![trial("H", "M2")],
        };
        match field {
            "value_nan" => m.train[0].observations[0].value = f64::NAN,
            "value_inf" => m.train[0].observations[0].value = f64::INFINITY,
            "sigma_nan" => m.train[0].observations[0].uncertainty_k1 = f64::NAN,
            "sigma_negative" => m.train[0].observations[0].uncertainty_k1 = -1.,
            "unit_missing" => m.train[0].observations[0].unit = "".into(),
            "campaign_missing" => m.train[0].campaign = "".into(),
            _ => {
                m.train[0].source = DataSource::Real {
                    provenance: "".into(),
                }
            }
        }
        emit(
            "manifest_boundary",
            json!({"field":field,"rejected":m.validated().is_err()}),
        );
    }
    let ecu = ecu::Ecu::new(
        ecu::ReferenceMaps::illustrative().unwrap(),
        0.001,
        12.,
        300000.,
    )
    .unwrap();
    for bad in ["load_nan", "rpm_nan", "phase_nan"] {
        let mut est = ecu::decoder::SyncEstimate {
            state: ecu::SyncState::Synced720,
            rpm: 1200.,
            confidence: 1.,
            cycle_deg_at: 100.,
            at_time_s: 1.,
        };
        let mut load = 0.5;
        match bad {
            "load_nan" => load = f64::NAN,
            "rpm_nan" => est.rpm = f64::NAN,
            _ => est.cycle_deg_at = f64::NAN,
        }
        let plan = ecu.schedule(&est, load, &ecu::DtcSet::default());
        emit(
            "ecu_boundary",
            json!({"bad":bad,"injection_count":plan.injections.len(),"ignition_count":plan.ignitions.len(),"finite_commands":plan.injections.iter().all(|x|x.fuel_kg.is_finite()&&x.pulse_width_s.is_finite())&&plan.ignitions.iter().all(|x|x.effective_btdc_deg.is_finite()),"state":format!("{:?}",plan.ecu_state)}),
        );
    }
    let fit = std::panic::catch_unwind(|| {
        calibration::least_squares(|_| vec![], &[1., 2.], &[1., 1.], &[1.], 1e-8)
    });
    emit(
        "calibration_dimension",
        json!({"panicked":fit.is_err(),"rejected":fit.as_ref().map(|v|v.is_err()).unwrap_or(false)}),
    );
    let fit = calibration::least_squares(
        |p| vec![p[0], 2. * p[0]],
        &[1., 2.],
        &[-1., 1.],
        &[0.5],
        1e-8,
    );
    emit(
        "calibration_negative_weight",
        json!({"accepted":fit.is_ok()}),
    );
}
fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("engine") => engine_probes(),
        Some("leak") => leak_probes(),
        Some("thermal") => thermal_probes(),
        Some("snapshot") => snapshot_probes(),
        Some("boundary") => boundary_probes(),
        _ => panic!("mode: engine/leak/thermal/snapshot/boundary"),
    }
}
