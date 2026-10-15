//! Independent C4 controls: provenance event composition, numerical boundaries and authorized fixture A.
use engine::{CylState, EngineInputs, EngineSnapshot, EngineState, WorkState};
use serde_json::{json, Value};
include!("common.rs");
fn emit(probe: &str, data: Value) {
    println!("{}", json!({"probe":probe,"data":data}));
}
fn event(w: &WorkState) -> Value {
    serde_json::to_value(&w.provenance).unwrap()
}
fn lineage(v: &Value) -> Vec<Value> {
    let mut a = Vec::new();
    fn walk(v: &Value, a: &mut Vec<Value>) {
        if v.is_null() {
            return;
        }
        let mut node = v.clone();
        node.as_object_mut().unwrap().remove("prior");
        a.push(node);
        if let Some(p) = v.get("prior").and_then(|x| x.as_array()) {
            for q in p {
                walk(q, a);
            }
        }
    }
    walk(v, &mut a);
    a
}
fn rt(s: &EngineSnapshot) -> EngineSnapshot {
    serde_json::from_str(&serde_json::to_string(s).unwrap()).unwrap()
}
fn provenance() {
    let mut i = engine::healthy_inputs();
    i.friction = crank::FrictionModel::ChenFlynn {
        coeffs: crank::ChenFlynnCoeffs {
            c1_pa: 20000.,
            c2_dlless: 0.05,
            c3_pa_s_m: 1000.,
            c4_pa_s2_m2: 100.,
        },
    };
    let genuine: EngineSnapshot =
        serde_json::from_str(include_str!("../fixtures/genuine-v1-cold.json")).unwrap();
    let mut w = WorkState::restore(&genuine, &i).unwrap();
    let origin = event(&w);
    let mut expected_hashes = vec![genuine.state_hash.clone()];
    emit(
        "origin",
        json!({"hash_valid":genuine.verified(),"state_hash":genuine.state_hash,"provenance":origin,"prior_absent_in_original":serde_json::from_str::<Value>(include_str!("../fixtures/genuine-v1-cold.json")).unwrap().get("provenance").is_none()}),
    );
    engine::resume(&mut w, &i, 0.5).unwrap();
    for step in 0..8 {
        let before = event(&w);
        let mut incoming = w.snapshot();
        if step % 2 == 0 {
            incoming.state.pmax_pa = None;
            incoming = EngineSnapshot::capture_with(&incoming.state, incoming.provenance.clone());
        } else {
            incoming.model = None;
        }
        let h = incoming.state_hash.clone();
        let mut restored = WorkState::restore(&incoming, &i).unwrap();
        let composed = event(&restored);
        expected_hashes.insert(0, h.clone());
        let mut same_envelope = incoming.clone();
        same_envelope.provenance = restored.provenance.clone();
        let mut reread_equal = true;
        for _ in 0..20 {
            let r = WorkState::restore(&rt(&same_envelope), &i).unwrap();
            reread_equal &= event(&r) == composed;
        }
        let mut recaptured = restored.snapshot();
        let mut recapture_equal = true;
        for _ in 0..20 {
            let r = WorkState::restore(&rt(&recaptured), &i).unwrap();
            recapture_equal &= event(&r) == composed;
            recaptured = r.snapshot();
        }
        let chain = lineage(&composed);
        let hashes: Vec<&str> = chain
            .iter()
            .map(|x| x["original_hash"].as_str().unwrap())
            .collect();
        let prior = restored
            .provenance
            .as_ref()
            .unwrap()
            .prior
            .first()
            .map(|x| serde_json::to_value(x).unwrap());
        emit(
            "provenance_transition",
            json!({"step":step,"cause":if step%2==0{"pmax_loss"}else{"accounting_marker_absent"},"incoming_verified":incoming.verified(),"origin_hash_unchanged":chain.last().unwrap()["original_hash"]==genuine.state_hash,"previous_record_preserved":prior==Some(before.clone()),"current_hash":h,"chain_hashes":hashes,"expected_hashes":expected_hashes,"chain_depth":chain.len(),"scope":composed["continuation"],"reread20_equal":reread_equal,"recapture20_equal":recapture_equal,"provenance":composed}),
        );
        let end = restored.time_s + 0.002;
        engine::resume(&mut restored, &i, end).unwrap();
        w = restored;
    }
    let cap = w.snapshot();
    let original_hash = cap.state_hash.clone();
    let mut altered = cap.clone();
    altered.provenance.as_mut().unwrap().original_hash = "METADATA_NOT_AUTHENTICATED".to_owned();
    let unchanged_hash = altered.state_hash == original_hash && altered.verified();
    emit(
        "metadata_trust",
        json!({"state_hash_still_valid_after_metadata_change":unchanged_hash,"contract":"hash covers state only; not provenance/prior/model authenticity"}),
    );
}
fn adapter() {
    let i = engine::healthy_inputs();
    let mut literal = cold(&i);
    literal.state.schema_version = 1;
    let raw = engine_types::canonical_json(&literal.state).replace(",\"thermal\":null", "");
    literal.state_hash = engine_types::sha256_hex(raw.as_bytes());
    let mut adapted = literal.clone();
    adapted.model = None;
    let genuine: EngineSnapshot =
        serde_json::from_str(include_str!("../fixtures/genuine-v1-cold.json")).unwrap();
    emit(
        "adapter_identity",
        json!({"literal_hash_valid":literal.verified(),"adapted_hash_valid":adapted.verified(),"genuine_hash_valid":genuine.verified(),"state_all_equal":literal.state==adapted.state&&literal.state==genuine.state,"hash_all_equal":literal.state_hash==adapted.state_hash&&literal.state_hash==genuine.state_hash,"provenance_all_equal":literal.provenance==adapted.provenance&&literal.provenance==genuine.provenance,"literal_accepted":WorkState::restore(&literal,&i).is_ok(),"adapted_accepted":WorkState::restore(&adapted,&i).is_ok(),"genuine_accepted":WorkState::restore(&genuine,&i).is_ok(),"state_hash":genuine.state_hash}),
    );
    for schema in [1, 2] {
        for marker in [
            None,
            Some((schema, 1)),
            Some((3 - schema, 1)),
            Some((schema, 999)),
            Some((999, 1)),
        ] {
            let mut s = cold(&i).state;
            s.schema_version = schema;
            let mut s = EngineSnapshot::capture(&s);
            s.model = marker.map(|(schema_version, combustion_accounting)| {
                engine::state::ModelIdentity {
                    schema_version,
                    combustion_accounting,
                }
            });
            let expected = marker.is_none() || marker == Some((schema, 1));
            emit(
                "schema_pair",
                json!({"schema":schema,"model":s.model,"hash_valid":s.verified(),"expected_valid":expected,"accepted":WorkState::restore(&s,&i).is_ok()}),
            );
        }
    }
}
fn battery() {
    for t in [-273.15, -273.149999, -20., 0., 10., 20., 100.] {
        for soc in [0., 0.5, 1.] {
            for (name, r0, ground) in [
                ("ideal", 0., 0.),
                ("subnormal", f64::from_bits(1), 0.),
                ("nominal", 0.008, 0.002),
                ("large_finite", f64::MAX / 16., 0.),
                ("product_overflow", f64::MAX, 0.),
                ("sum_overflow", f64::MAX, f64::MAX),
            ] {
                let base = engine::healthy_inputs();
                let mut i = base.clone();
                let b = &mut i.starter.as_mut().unwrap().2;
                b.temp_c = t;
                b.soc = soc;
                b.r0_ohm = r0;
                b.r_ground_ohm = ground;
                let bat = *b;
                let rr = bat.resistance();
                let r_finite = rr.as_ref().is_ok_and(|x| x.is_finite() && *x >= 0.);
                let terminal = bat.terminal_voltage(350.);
                let valid = bat.validated().is_ok();
                let mut w = ws(&base);
                w.m_in_amb_kg = 0.123;
                let before = format!("{w:?}");
                let resume = engine::resume(&mut w, &i, 0.);
                let restore = WorkState::restore(&cold(&base), &i);
                i.t_end_s = 0.001;
                let run = engine::run_engine(&i);
                emit(
                    "battery_boundary",
                    json!({"case":name,"temp":t,"soc":soc,"validated":valid,"resistance_ok":rr.is_ok(),"resistance_finite_nonnegative":r_finite,"resistance":rr.ok(),"terminal_ok":terminal.is_ok(),"terminal_finite":terminal.as_ref().is_ok_and(|x|x.is_finite()),"resume_ok":resume.is_ok(),"restore_ok":restore.is_ok(),"run_ok":run.is_ok(),"unchanged_on_error":resume.is_ok()||before==format!("{w:?}")}),
                );
            }
        }
    }
}
fn nodes() {
    for exhaust in [false, true] {
        for (name, m, t, v) in [
            ("nominal", 1e-3, 300., 1e-3),
            ("overflow_p", 1e-5, 1e100, 1e-214),
            ("underflow_p", 1e-200, 1e-200, 1e-5),
            ("overflow_rho", 1., 1., 1e-320),
            ("finite_extreme", 1e-5, 1e100, 1e-200),
            ("small_finite", 1e-100, 1e-50, 1e-20),
        ] {
            let base = engine::healthy_inputs();
            let mut i = base.clone();
            let mut w = ws(&base);
            let n = if exhaust { &mut w.exh } else { &mut w.plen };
            n.m = m;
            n.t = t;
            n.v = v;
            if exhaust {
                i.exhaust_v_m3 = v
            } else {
                i.plenum_v_m3 = v
            };
            let p = m * i.mix.r() * t / v;
            let rho = m / v;
            let valid = p.is_finite() && p > 0. && rho.is_finite() && rho > 0.;
            let snap = w.snapshot();
            let rest = WorkState::restore(&snap, &i);
            for end in [0., 0.0001] {
                let mut a = w.clone();
                a.m_in_amb_kg = 0.123;
                let before = format!("{a:?}");
                let r = engine::resume(&mut a, &i, end);
                emit(
                    "node_boundary",
                    json!({"exhaust":exhaust,"case":name,"end":end,"expected_valid_state":valid,"hash_valid":snap.verified(),"restore_ok":rest.is_ok(),"resume_ok":r.is_ok(),"unchanged_on_error":r.is_ok()||before==format!("{a:?}")}),
                );
            }
            // run_engine owns its factory state; parameters here remain legal positive volumes.
            i.t_end_s = 0.;
            let run = engine::run_engine(&i);
            emit(
                "node_factory",
                json!({"exhaust":exhaust,"case":name,"accepted":run.is_ok(),"audit_finite":run.as_ref().is_ok_and(|r|r.audit.mass_closure.is_finite()&&r.audit.energy_closure.is_finite())}),
            );
        }
    }
}
fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("provenance") => provenance(),
        Some("adapter") => adapter(),
        Some("battery") => battery(),
        Some("nodes") => nodes(),
        _ => panic!("mode required"),
    }
}
