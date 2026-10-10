//! Mecanismos S27: `piston_valve_collision_if_contact`, `bolt_yield`,
//! `bearing_wear_under_oil_starvation` (riesgo, sin gripado), rosca
//! (UNSUPPORTED). La trayectoria nunca cambia por el diagnóstico: los
//! chequeos son funciones puras de entradas declaradas.

use thiserror::Error;

/// Error de mecanismo.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum DamageError {
    /// Entradas inválidas o mecanismo sin soporte para el paso pedido.
    #[error("invalid damage input: {reason}")]
    Invalid { reason: String },
}

/// Soporte (misma terminología que S26).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Support {
    GeometricOnly,
    CriterionGeneric,
    Unsupported,
}

/// Mecanismo con dominio de validez declarado.
#[derive(Debug, Clone, PartialEq)]
pub struct DamageMechanism {
    pub name: String,
    pub domain: String,
    pub support: Support,
    pub provenance: String,
}

/// Estado de un mecanismo: riesgo con margen, o ruptura con criterio.
#[derive(Debug, Clone, PartialEq)]
pub enum MechanismState {
    /// Sin contacto / dentro de criterio (con margen declarado).
    Clear { margin: String },
    /// Condición presente pero criterio de daño no evaluable: se conserva
    /// RIESGO, jamás rotura teatral (gate S27).
    Risk { margin: String, needs: String },
    /// Criterio físico cumplido con material+geometría declarados.
    Ruptured { criterion: String },
}

/// Resultado de contacto (hecho geométrico, no veredicto de daño).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContactOutcome {
    pub clearance_m: f64,
    pub overlap: bool,
}

/// Estado de perno (el único con criterio de ruptura genérico).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoltState {
    Elastic,
    UncertainRange,
    PlasticOnsetGeneric,
}

/// Pistón–válvula SI contacto: el solape es geometría (verificada S07);
/// doblar la válvula exigiría material de válvula no medido ⇒ RIESGO.
/// La trayectoria del motor no se toca (función pura).
///
/// # Errors
///
/// `Invalid` si las entradas violan el dominio de S07.
pub fn piston_valve_state(
    input: contacts::interference::PistonValveInput,
) -> Result<(ContactOutcome, MechanismState), DamageError> {
    let rep =
        contacts::interference::check_piston_valve(input).map_err(|e| DamageError::Invalid {
            reason: format!("piston-valve: {e}"),
        })?;
    let contact = ContactOutcome {
        clearance_m: rep.clearance_m,
        overlap: rep.overlap,
    };
    let state = if !rep.overlap {
        MechanismState::Clear {
            margin: format!("clearance {:.3} mm", rep.clearance_m * 1e3),
        }
    } else {
        MechanismState::Risk {
            margin: format!("penetration {:.3} mm", rep.penetration_m * 1e3),
            needs: "valve material + seat geometry (PENDING); bending NOT inferred".to_owned(),
        }
    };
    Ok((contact, state))
}

/// Resistencia genérica ISO 898-1 clase 10.9 (Rp0.2 mín 940 MPa):
/// material GENÉRICO de la clase, nunca medido del perno D16Y7.
pub const RP02_CLASS109_PA: f64 = 940e6;

/// Fluencia de perno desde torque (cadena S08): σ = F/A con F de
/// `T/(K·d)` en rango de lubricante. Ruptura (fluencia) solo si TODO el
/// rango supera Rp0.2 genérico; si el rango lo cruza ⇒ riesgo/estado.
///
/// # Errors
///
/// `Invalid` si torque/diámetro/área violan dominio.
pub fn bolt_yield_state(
    torque_nm: f64,
    nominal_dia_m: f64,
    tensile_area_m2: f64,
    lubricant: fastener::preload::LubricantCondition,
) -> Result<(BoltState, DamageMechanism), DamageError> {
    let bad = DamageError::Invalid {
        reason: "bolt yield".to_owned(),
    };
    if !(torque_nm.is_finite() && torque_nm > 0.0) {
        return Err(bad);
    }
    if !(nominal_dia_m.is_finite() && nominal_dia_m > 0.0) {
        return Err(bad);
    }
    if !(tensile_area_m2.is_finite() && tensile_area_m2 > 0.0) {
        return Err(bad);
    }
    let (k_lo, k_hi) = fastener::preload::illustrative_k_range(lubricant).map_err(|_| bad)?;
    // K alto ⇒ F baja: utilización mínima con k_hi.
    let u_lo = torque_nm / (k_hi * nominal_dia_m) / tensile_area_m2 / RP02_CLASS109_PA;
    let u_hi = torque_nm / (k_lo * nominal_dia_m) / tensile_area_m2 / RP02_CLASS109_PA;
    let mech = DamageMechanism {
        name: "bolt_yield".to_owned(),
        domain: "axial preload vs Rp0.2 ISO 898-1 10.9 (GENERIC, not D16Y7 measured)".to_owned(),
        support: Support::CriterionGeneric,
        provenance: "S08 preload chain + ISO 898-1 class proof (generic)".to_owned(),
    };
    let state = if u_lo > 1.0 {
        BoltState::PlasticOnsetGeneric
    } else if u_hi < 1.0 {
        BoltState::Elastic
    } else {
        BoltState::UncertainRange
    };
    Ok((state, mech))
}

/// Cojinete bajo inanición: tendencia S26 (holgura que crece) + riesgo
/// por película; el GRIPADO no tiene criterio ⇒ nunca ruptura aquí.
///
/// # Errors
///
/// `Invalid` si film/carga violan dominio.
pub fn bearing_starvation_risk(film_0_1: f64, load_pa: f64) -> Result<MechanismState, DamageError> {
    if !(film_0_1.is_finite() && (0.0..=1.0).contains(&film_0_1)) {
        return Err(DamageError::Invalid {
            reason: "film".to_owned(),
        });
    }
    if !(load_pa.is_finite() && load_pa >= 0.0) {
        return Err(DamageError::Invalid {
            reason: "load".to_owned(),
        });
    }
    if film_0_1 >= 0.5 {
        Ok(MechanismState::Clear {
            margin: format!("film {film_0_1:.2} sustains load"),
        })
    } else {
        Ok(MechanismState::Risk {
            margin: format!("film {film_0_1:.2} under load {:.1} MPa", load_pa / 1e6),
            needs: "seizure criterion (UNSUPPORTED); wear trend via S26".to_owned(),
        })
    }
}

/// Rosca arrancada: sin longitud de compromiso medida ⇒ UNSUPPORTED, sin
/// números (gate S27: conservar estado, no teatralizar).
#[must_use]
pub fn thread_state() -> (DamageMechanism, MechanismState) {
    (
        DamageMechanism {
            name: "thread_strip".to_owned(),
            domain: "engagement length + material (PENDING)".to_owned(),
            support: Support::Unsupported,
            provenance: "no engagement data; S27 keeps risk state".to_owned(),
        },
        MechanismState::Risk {
            margin: "unknown engagement".to_owned(),
            needs: "thread engagement + nut material (PENDING)".to_owned(),
        },
    )
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use contacts::interference::PistonValveInput;

    fn pv(clear_gap_mm: f64, lift_mm: f64, piston_mm: f64) -> PistonValveInput {
        PistonValveInput {
            closed_gap_m: clear_gap_mm * 1e-3,
            lift_m: lift_mm * 1e-3,
            cant_rad: 0.0,
            valve_dia_m: 30e-3,
            piston_x_m: piston_mm * 1e-3,
        }
    }

    #[test]
    fn timing_offset_without_collision_never_bends_valve() {
        // Prueba negativa S27-T5: desfase con huelgo ⇒ Clear, sin daño.
        let (c, s) = piston_valve_state(pv(3.0, 8.0, 6.0)).unwrap();
        assert!(!c.overlap);
        assert!(matches!(s, MechanismState::Clear { .. }));
        // Contacto real ⇒ RIESGO con penetración, jamás válvula doblada.
        let (c2, s2) = piston_valve_state(pv(3.0, 8.0, 0.5)).unwrap();
        assert!(c2.overlap);
        match &s2 {
            MechanismState::Risk { needs, .. } => assert!(needs.contains("NOT inferred")),
            _ => panic!("contact must stay Risk, got {s2:?}"),
        }
        // Función pura: mismas entradas ⇒ mismo reporte (trayectoria
        // intacta por construcción).
        assert_eq!(piston_valve_state(pv(3.0, 8.0, 0.5)).unwrap(), (c2, s2));
    }

    #[test]
    fn bolt_tolerable_stays_elastic_plastic_needs_full_range() {
        use fastener::preload::LubricantCondition;
        // M10 10.9 (At 58 mm²) a 60 Nm aceitado: elástico.
        let (st, _) =
            bolt_yield_state(60.0, 10e-3, 58e-6, LubricantCondition::OiledGeneric).unwrap();
        assert_eq!(st, BoltState::Elastic);
        // 80 Nm: el rango de K cruza la fluencia (u 0.86–1.13) ⇒ el
        // estado honesto es INCERTIDUMBRE, no rotura (sesgo de
        // parametrización S27-T6: con K estrecho medido se decidiría).
        // (100 Nm ya fluye todo el rango: corrección de una expectativa
        // errónea del test inicial — la física mandó.)
        let (st2, _) =
            bolt_yield_state(80.0, 10e-3, 58e-6, LubricantCondition::OiledGeneric).unwrap();
        assert_eq!(st2, BoltState::UncertainRange);
        // Sobre-par extremo: TODO el rango sobre Rp0.2 ⇒ fluencia
        // genérica (criterio cumplido, material declarado).
        let (st3, mech) =
            bolt_yield_state(400.0, 10e-3, 58e-6, LubricantCondition::OiledGeneric).unwrap();
        assert_eq!(st3, BoltState::PlasticOnsetGeneric);
        assert!(mech.domain.contains("GENERIC"));
        assert!(bolt_yield_state(-5.0, 10e-3, 58e-6, LubricantCondition::OiledGeneric).is_err());
    }

    #[test]
    fn starvation_is_risk_never_seizure_and_thread_unsupported() {
        assert!(matches!(
            bearing_starvation_risk(0.8, 5e6).unwrap(),
            MechanismState::Clear { .. }
        ));
        match bearing_starvation_risk(0.2, 8e6).unwrap() {
            MechanismState::Risk { needs, .. } => assert!(needs.contains("UNSUPPORTED")),
            _ => panic!("starvation must stay Risk"),
        }
        let (mech, state) = thread_state();
        assert_eq!(mech.support, Support::Unsupported);
        assert!(matches!(state, MechanismState::Risk { .. }));
        assert!(bearing_starvation_risk(1.5, 5e6).is_err());
    }
}
