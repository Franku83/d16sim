//! Las 4 cadenas causales S30-T3: cada eslabón calculado, perturbación que
//! se propaga o test que la declara ausente con motivo.

use thiserror::Error;

/// Error de auditoría.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum AuditError {
    /// Eslabón roto o entrada inválida (fail-closed, nunca PASS tapado).
    #[error("audit failed: {reason}")]
    Invalid { reason: String },
}

/// Reporte de una cadena (eslabones nombrados + veredicto).
#[derive(Debug, Clone, PartialEq)]
pub struct ChainReport {
    pub name: String,
    pub links: Vec<String>,
    pub pass: bool,
}

/// Ejecuta las 4 cadenas; `Err` si alguna rompe.
///
/// # Errors
///
/// `Invalid` si cualquier cadena falla.
pub fn audit_chains() -> Result<Vec<ChainReport>, AuditError> {
    Ok(vec![
        chain_torque_to_sealing()?,
        chain_cam_to_compression()?,
        chain_ckp_to_spark()?,
        chain_ring_to_blowby()?,
    ])
}

/// Cadena 1: par de culata → precarga (S08) → campo de junta → área de
/// fuga (S23). Más par ⇒ menos fuga (monótono por construcción).
fn chain_torque_to_sealing() -> Result<ChainReport, AuditError> {
    let bad = |r: &str| AuditError::Invalid {
        reason: r.to_owned(),
    };
    let segs = |a: f64| {
        (0..4)
            .map(|_| fastener::SealSegment {
                area_m2: a,
                stiffness_weight: 1.0,
            })
            .collect::<Vec<_>>()
    };
    // Ancla p_ref a clamp completo (convención S23).
    let full = fastener::distribute_clamp(200_000.0, &segs(400e-6)).map_err(|_| bad("clamp"))?;
    let p_ref = full
        .contact_pressure_pa
        .iter()
        .fold(0.0f64, |a, b| a.max(*b));
    let mut areas = Vec::new();
    for load in [100_000.0, 200_000.0] {
        let f = fastener::distribute_clamp(load, &segs(400e-6)).map_err(|_| bad("clamp"))?;
        let a: f64 = f
            .contact_pressure_pa
            .iter()
            .map(|p| {
                leak::gasket_leak_area(
                    leak::SealContactState {
                        contact_pressure_pa: *p,
                        area_m2: 400e-6,
                    },
                    p_ref,
                )
            })
            .sum();
        areas.push(a);
    }
    if !(areas[0] > areas[1] && areas[1] == 0.0) {
        return Err(bad("torque must reduce leak area to zero at full clamp"));
    }
    Ok(ChainReport {
        name: "torque->sealing".to_owned(),
        links: vec![
            "torque 100/200 kN (S08)".to_owned(),
            "preload field (S08)".to_owned(),
            "gasket leak area (S23)".to_owned(),
            format!("area {:.3e} -> {:.3e} m2", areas[0], areas[1]),
        ],
        pass: true,
    })
}

/// Cadena 2: calado de leva (S11) → eventos de válvula → motor
/// integrado (S16). Sin acoplos inventados: el calado entra al motor por
/// su campo `intake_cam` y el régimen emerge (trazabilidad real).
fn chain_cam_to_compression() -> Result<ChainReport, AuditError> {
    let bad = |r: &str| AuditError::Invalid {
        reason: r.to_owned(),
    };
    let run_at = |shift_deg: f64| -> Result<f64, AuditError> {
        let mut inp = engine::healthy_inputs();
        inp.throttle_area_m2 = 0.3e-4;
        inp.accessory_tau_n_m = 8.0;
        inp.t_end_s = 3.0;
        inp.intake_cam.centerline_atdc_deg += shift_deg;
        let t = engine::run_engine(&inp).map_err(|_| bad("engine"))?;
        if t.audit.mass_closure.abs() > 1e-9 {
            return Err(bad("mass"));
        }
        Ok(t.audit.final_rpm)
    };
    let (r0, r1) = (run_at(0.0)?, run_at(6.0)?);
    if !((r1 - r0).abs() > 1.0) {
        return Err(bad("centerline shift must move idle rpm"));
    }
    Ok(ChainReport {
        name: "cam->compression".to_owned(),
        links: vec![
            "centerline +6° (S11 proxy)".to_owned(),
            "intake_cam field (S16 input)".to_owned(),
            format!("idle rpm {r0:.0} -> {r1:.0} (emergent, no coupling invented)"),
        ],
        pass: true,
    })
}

/// Cadena 3: flancos CKP (S17) → sync/rpm del decoder (S18) → avance del
/// mapa. Borde saltado ⇒ confianza < 1 (se propaga, no se etiqueta).
fn chain_ckp_to_spark() -> Result<ChainReport, AuditError> {
    let bad = |r: &str| AuditError::Invalid {
        reason: r.to_owned(),
    };
    let healthy_cfg = trigger::TriggerConfig::generic_family();
    let wave = trigger::DistributorTrigger::new(healthy_cfg).map_err(|_| bad("wave"))?;
    let edges = wave
        .channel_edges(0, 750.0, 0.0, 3.0, 20_000.0, 11)
        .map_err(|_| bad("edges"))?;
    if edges.len() < 30 {
        return Err(bad("too few edges"));
    }
    // CYP sintético coherente (1 flanco / 24 CKP, convención S18).
    let per = edges.len() / 24;
    let cyp: Vec<f64> = (0..3)
        .map(|i| edges[(i * per).min(edges.len() - 1)])
        .collect();
    let now = *edges.last().ok_or_else(|| bad("empty edges"))?;
    let d = ecu::Decoder::new(edges.clone(), cyp.clone(), now).map_err(|_| bad("decoder"))?;
    let rpm_ok = d.rpm_estimate().ok_or_else(|| bad("no rpm"))?;
    let conf_ok = d.confidence();
    // Perturbación: borde saltado en ventana reciente.
    let mut bad_edges = edges.clone();
    let n = bad_edges.len();
    bad_edges.remove(n - 5);
    let now2 = *bad_edges.last().ok_or_else(|| bad("empty edges"))?;
    let d2 = ecu::Decoder::new(bad_edges, cyp, now2).map_err(|_| bad("decoder2"))?;
    let conf_bad = d2.confidence();
    if !(conf_ok >= conf_bad && conf_bad < 1.0) {
        return Err(bad("missing edge must degrade confidence"));
    }
    let map = ecu::ReferenceMaps::illustrative().map_err(|_| bad("map"))?;
    let adv = map.spark_btdc_deg.lookup(rpm_ok, 0.6);
    if !adv.is_finite() {
        return Err(bad("map lookup"));
    }
    Ok(ChainReport {
        name: "ckp->spark".to_owned(),
        links: vec![
            format!("CKP edges (S17 GENERIC, n={})", edges.len()),
            format!("decoder rpm {rpm_ok:.1}, conf {conf_ok:.3} -> {conf_bad:.3}"),
            format!("map advance {adv:.1} deg (REF-*, S18)"),
        ],
        pass: true,
    })
}

/// Cadena 4: horas de uso → gap (S26) → área de fuga (S23). El desgaste
/// abre la fuga por la ley, no por bandera.
fn chain_ring_to_blowby() -> Result<ChainReport, AuditError> {
    let bad = |r: &str| AuditError::Invalid {
        reason: r.to_owned(),
    };
    let op = wear::OperatingPoint {
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
    let fresh = wear::WearState::new_seeded(7);
    let mut worn = wear::WearState::new_seeded(7);
    for _ in 0..100 {
        worn.advance(0.5, &op, &laws.0, &laws.1, &laws.2)
            .map_err(|_| bad("wear"))?;
    }
    let (a0, a1) = (
        fresh.ring_leak_area_m2(0, 3e-3, 0.03),
        worn.ring_leak_area_m2(0, 3e-3, 0.03),
    );
    if !(a1 > a0) {
        return Err(bad("wear must open leak area"));
    }
    Ok(ChainReport {
        name: "ring->blowby".to_owned(),
        links: vec![
            "50 h at 3000 rpm (S26)".to_owned(),
            format!(
                "gap {:.1} -> {:.1} um",
                fresh.ring_gap_m[0] * 1e6,
                worn.ring_gap_m[0] * 1e6
            ),
            format!("leak area {a0:.3e} -> {a1:.3e} m2 (S23 law)"),
        ],
        pass: true,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn all_four_chains_hold() {
        let chains = audit_chains().unwrap();
        assert_eq!(chains.len(), 4);
        for c in &chains {
            assert!(c.pass, "{c:?}");
            assert!(c.links.len() >= 3, "{c:?}");
        }
        let names: Vec<&str> = chains.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "torque->sealing",
                "cam->compression",
                "ckp->spark",
                "ring->blowby"
            ]
        );
    }
}
