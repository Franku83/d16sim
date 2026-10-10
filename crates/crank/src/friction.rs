//! Fricción reducida (S14): FMEP paramétrico con coeficientes del
//! llamante. Sin coeficientes medidos del D16Y7: el modelo es la FORMA,
//! los números son entradas (demo ilustrativa etiquetada).
//!
//! Forma Chen–Flynn sin término de carga por defecto (`c2 = 0`): Pipitone
//! (SAE 2009-01-1984) muestra que el término de carga es el eslabón débil
//! en SI pequeños (error medio 13%, máx 35%). Forzar `c2 ≠ 0` exige
//! evidencia (documentado).

use thiserror::Error;

/// Error de fricción.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum FrictionError {
    /// Coeficientes o punto operativo inválidos.
    #[error("invalid friction input: {reason}")]
    InvalidInput { reason: String },
}

/// Coeficientes Chen–Flynn: `FMEP = c1 + c2·pmax + c3·Sp + c4·Sp²`
/// (Pa, Pa/Pa, Pa·s/m, Pa·s²/m²). Llamante los aporta; S14 no calibra.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChenFlynnCoeffs {
    pub c1_pa: f64,
    pub c2_dlless: f64,
    pub c3_pa_s_m: f64,
    pub c4_pa_s2_m2: f64,
}

/// Modelo de fricción.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrictionModel {
    /// Sin fricción (referencia de conservación).
    None,
    /// Viscoso puro: τ = c·ω.
    Viscous { c_n_m_s: f64 },
    /// FMEP Chen–Flynn → τ = FMEP·Vd/(4π) (4T), opuesto al giro.
    ChenFlynn { coeffs: ChenFlynnCoeffs },
}

impl FrictionModel {
    /// Torque de fricción (N·m, ≥ 0 como magnitud resistente).
    ///
    /// # Errors
    ///
    /// `InvalidInput` si coeficientes, velocidad media del pistón o pmax
    /// violan dominio.
    pub fn torque_mag(
        &self,
        mean_piston_speed_m_s: f64,
        pmax_pa: f64,
        displaced_m3: f64,
        omega_rad_s: f64,
    ) -> Result<f64, FrictionError> {
        let bad = |reason: &str| FrictionError::InvalidInput {
            reason: reason.to_owned(),
        };
        match *self {
            FrictionModel::None => Ok(0.0),
            FrictionModel::Viscous { c_n_m_s } => {
                if !(c_n_m_s.is_finite() && c_n_m_s >= 0.0) {
                    return Err(bad("viscous c must be finite and >= 0"));
                }
                if !omega_rad_s.is_finite() {
                    return Err(bad("omega must be finite"));
                }
                Ok(c_n_m_s * omega_rad_s.abs())
            }
            FrictionModel::ChenFlynn { coeffs } => {
                let c = coeffs;
                for (v, name) in [
                    (c.c1_pa, "c1"),
                    (c.c2_dlless, "c2"),
                    (c.c3_pa_s_m, "c3"),
                    (c.c4_pa_s2_m2, "c4"),
                    (mean_piston_speed_m_s, "Sp"),
                    (pmax_pa, "pmax"),
                    (displaced_m3, "Vd"),
                ] {
                    if !(v.is_finite() && v >= 0.0) {
                        return Err(bad(name));
                    }
                }
                let fmep = c.c1_pa
                    + c.c2_dlless * pmax_pa
                    + c.c3_pa_s_m * mean_piston_speed_m_s
                    + c.c4_pa_s2_m2 * mean_piston_speed_m_s * mean_piston_speed_m_s;
                Ok(fmep * displaced_m3 / (4.0 * std::f64::consts::PI))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn chen_flynn_shape_without_load_term() {
        // Forma ilustrativa (NO coeficientes D16Y7): c1 = 0.3 bar,
        // c3/c4 para ~1 bar a Sp ≈ 4.5 m/s (750 rpm, S = 90 mm).
        let m = FrictionModel::ChenFlynn {
            coeffs: ChenFlynnCoeffs {
                c1_pa: 30_000.0,
                c2_dlless: 0.0,
                c3_pa_s_m: 8_000.0,
                c4_pa_s2_m2: 400.0,
            },
        };
        let vd = 4.0 * 397.6e-6;
        let t = m.torque_mag(4.5, 2e6, vd, 78.5).unwrap();
        // FMEP = 30000 + 36000 + 8100 = 74100 Pa ⇒ τ ≈ 9.4 N·m.
        assert!((t - 9.38).abs() < 0.05, "t={t}");
        // pmax no entra con c2 = 0 (Pipitone: término débil).
        let t2 = m.torque_mag(4.5, 9e6, vd, 78.5).unwrap();
        assert_eq!(t, t2);
        assert!(m.torque_mag(-1.0, 0.0, vd, 78.5).is_err());
    }
}
