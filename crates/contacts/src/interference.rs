//! Detección geométrica de interferencias (prototipo S07).
//!
//! - Pistón–válvula coaxial: `clearance = gap0 − lift·cos(cant) −
//!   (Dv/2)·sin(cant) + x(θ)`; negativo ⇒ contacto con penetración.
//!   El offset lateral no cambia el huelgo axial con corona plana
//!   (documentado, testeado); sí importará en asientos/ángulo (S11).
//! - Biela–bloque: hulgura mínima = pared − (r + semiancho de biela).
//! - Rosca: ajuste nominal por diámetros + paso.
//!
//! Todo contacto devuelve `needs_material_model = true` y
//! `damage = NotAssessed`: **ningún daño mecánico sin material/criterio**.

use thiserror::Error;
use units::float::require_finite;

/// Error de verificación geométrica (entrada inválida, no daño).
#[derive(Debug, Clone, PartialEq, Error)]
pub enum InterferenceError {
    /// Dimensiones de entrada inválidas.
    #[error("invalid interference input: {reason}")]
    InvalidInput { reason: String },

    /// Unión geométricamente imposible (penetración sólida sin
    /// deformación/corte contemplados en el modelo actual).
    #[error("geometrically impossible: {reason}")]
    Impossible { reason: String },
}

/// Veredicto de daño: en S07 siempre `NotAssessed` (gate).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageVerdict {
    NotAssessed,
}

/// Reporte de contacto/interferencia.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContactReport {
    /// Huelgo firmado (m): negativo = penetración.
    pub clearance_m: f64,
    /// ¿Hay solape geométrico?
    pub overlap: bool,
    /// Penetración (m), 0 si no hay solape.
    pub penetration_m: f64,
    /// El daño requiere modelo material (S27): siempre true en S07.
    pub needs_material_model: bool,
    /// Veredicto: siempre `NotAssessed` en S07.
    pub damage: DamageVerdict,
}

/// Entradas del chequeo pistón–válvula (todo SI, todo del llamante).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PistonValveInput {
    /// Huelgo con válvula cerrada sobre el plano de PMS (m).
    pub closed_gap_m: f64,
    /// Alzada impuesta (m). El perfil real es S11.
    pub lift_m: f64,
    /// Ángulo de la válvula respecto al eje (rad, 0 = coaxial).
    pub cant_rad: f64,
    /// Diámetro de cabeza de válvula (m).
    pub valve_dia_m: f64,
    /// Desplazamiento del pistón desde PMS `x(θ)` (m).
    pub piston_x_m: f64,
}

/// Chequeo axial pistón–válvula.
///
/// # Errors
///
/// `InvalidInput` si alguna entrada no es finita, es negativa donde no
/// corresponde, o el huelgo cerrado es negativo.
pub fn check_piston_valve(input: PistonValveInput) -> Result<ContactReport, InterferenceError> {
    let bad = |reason: &str| InterferenceError::InvalidInput {
        reason: reason.to_owned(),
    };
    for (v, name) in [
        (input.closed_gap_m, "closed_gap"),
        (input.lift_m, "lift"),
        (input.valve_dia_m, "valve_dia"),
        (input.piston_x_m, "piston_x"),
    ] {
        require_finite(v, name).map_err(|_| bad("non-finite input"))?;
        if v < 0.0 {
            return Err(bad("negative length input"));
        }
    }
    require_finite(input.cant_rad, "cant").map_err(|_| bad("non-finite cant"))?;
    // Borde más bajo del plato inclinado + avance axial por alzada.
    let lowest_edge_drop = input.lift_m * input.cant_rad.cos()
        + (input.valve_dia_m / 2.0) * input.cant_rad.sin().abs();
    let clearance = input.closed_gap_m - lowest_edge_drop + input.piston_x_m;
    Ok(ContactReport {
        clearance_m: clearance,
        overlap: clearance < 0.0,
        penetration_m: (-clearance).max(0.0),
        needs_material_model: true,
        damage: DamageVerdict::NotAssessed,
    })
}

/// Huelgo mínimo biela–pared del cárter (modelo L0 simétrico).
///
/// # Errors
///
/// `InvalidInput` si alguna entrada no es finita o es negativa.
pub fn check_rod_block(
    crank_radius_m: f64,
    rod_beam_halfwidth_m: f64,
    wall_halfwidth_m: f64,
) -> Result<ContactReport, InterferenceError> {
    for (v, name) in [
        (crank_radius_m, "crank_radius"),
        (rod_beam_halfwidth_m, "beam_halfwidth"),
        (wall_halfwidth_m, "wall_halfwidth"),
    ] {
        require_finite(v, name).map_err(|_| InterferenceError::InvalidInput {
            reason: "non-finite input".to_owned(),
        })?;
        if v < 0.0 {
            return Err(InterferenceError::InvalidInput {
                reason: "negative length input".to_owned(),
            });
        }
    }
    let clearance = wall_halfwidth_m - (crank_radius_m + rod_beam_halfwidth_m);
    Ok(ContactReport {
        clearance_m: clearance,
        overlap: clearance < 0.0,
        penetration_m: (-clearance).max(0.0),
        needs_material_model: true,
        damage: DamageVerdict::NotAssessed,
    })
}

/// Ajuste de rosca nominal: diámetros + coincidencia de paso.
///
/// # Errors
///
/// `InvalidInput` ante no finitos; `Impossible` si el paso no coincide o
/// el macho supera a la hembra (interferencia sólida).
pub fn thread_fit(
    male_major_dia_m: f64,
    female_minor_dia_m: f64,
    pitch_matches: bool,
) -> Result<f64, InterferenceError> {
    for (v, name) in [
        (male_major_dia_m, "male_dia"),
        (female_minor_dia_m, "female_dia"),
    ] {
        require_finite(v, name).map_err(|_| InterferenceError::InvalidInput {
            reason: "non-finite input".to_owned(),
        })?;
        if v <= 0.0 {
            return Err(InterferenceError::InvalidInput {
                reason: "non-positive diameter".to_owned(),
            });
        }
    }
    if !pitch_matches {
        return Err(InterferenceError::Impossible {
            reason: "thread pitch mismatch".to_owned(),
        });
    }
    if male_major_dia_m > female_minor_dia_m {
        return Err(InterferenceError::Impossible {
            reason: "solid interference: male exceeds female".to_owned(),
        });
    }
    Ok(female_minor_dia_m - male_major_dia_m)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn pv(gap: f64, lift: f64, x: f64) -> ContactReport {
        check_piston_valve(PistonValveInput {
            closed_gap_m: gap,
            lift_m: lift,
            cant_rad: 0.0,
            valve_dia_m: 0.030,
            piston_x_m: x,
        })
        .unwrap()
    }

    #[test]
    fn coaxial_clearance_is_exact() {
        // clearance = gap − lift + x (cant 0).
        let r = pv(0.006, 0.008, 0.0);
        assert!(r.overlap);
        assert!((r.penetration_m - 0.002).abs() < 1e-15);
        assert_eq!(r.damage, DamageVerdict::NotAssessed);
        assert!(r.needs_material_model);
        let ok = pv(0.006, 0.004, 0.0);
        assert!(!ok.overlap);
        assert!((ok.clearance_m - 0.002).abs() < 1e-15);
        // El pistón al bajar (x > 0) abre huelgo.
        let down = pv(0.006, 0.008, 0.010);
        assert!(!down.overlap);
    }

    #[test]
    fn cant_angle_lowers_the_disc_edge() {
        let flat = pv(0.006, 0.004, 0.0);
        let canted = check_piston_valve(PistonValveInput {
            closed_gap_m: 0.006,
            lift_m: 0.004,
            cant_rad: 0.1,
            valve_dia_m: 0.030,
            piston_x_m: 0.0,
        })
        .unwrap();
        // Borde: 0.004·cos(.1) + 0.015·sin(.1) ≈ 0.00548 > 0.004.
        assert!(canted.clearance_m < flat.clearance_m);
        assert!(
            (canted.clearance_m - (0.006 - (0.004 * 0.1f64.cos() + 0.015 * 0.1f64.sin()))).abs()
                < 1e-15
        );
        assert_eq!(canted.damage, DamageVerdict::NotAssessed);
    }

    #[test]
    fn rod_block_and_thread_follow_inputs() {
        let ok = check_rod_block(0.045, 0.010, 0.080).unwrap();
        assert!((ok.clearance_m - 0.025).abs() < 1e-15);
        let hit = check_rod_block(0.045, 0.010, 0.050).unwrap();
        assert!(hit.overlap);
        assert_eq!(hit.damage, DamageVerdict::NotAssessed);
        assert!((thread_fit(0.010, 0.0105, true).unwrap() - 0.0005).abs() < 1e-15);
        assert!(matches!(
            thread_fit(0.010, 0.0105, false),
            Err(InterferenceError::Impossible { .. })
        ));
        assert!(matches!(
            thread_fit(0.011, 0.0105, true),
            Err(InterferenceError::Impossible { .. })
        ));
    }

    #[test]
    fn damage_is_never_assessed_and_inputs_validated() {
        assert!(matches!(
            check_piston_valve(PistonValveInput {
                closed_gap_m: f64::NAN,
                lift_m: 0.0,
                cant_rad: 0.0,
                valve_dia_m: 0.030,
                piston_x_m: 0.0,
            }),
            Err(InterferenceError::InvalidInput { .. })
        ));
        // Ni siquiera con 10 mm de penetración se decreta daño.
        let deep = pv(0.001, 0.011, 0.0);
        assert!(deep.overlap);
        assert_eq!(deep.damage, DamageVerdict::NotAssessed);
    }
}
