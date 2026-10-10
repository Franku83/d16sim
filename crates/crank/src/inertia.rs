//! Inercia efectiva (S14): esquema de J constante por equivalencia
//! energética media. `J(θ)` variable real es deuda con costura declarada.
//!
//! `J_eff = J_rot + m_recip·⟨(dx/dθ)²⟩` (media en el ciclo): la energía
//! cinética media del alternativo queda representada; la modulación por
//! ángulo se desprecia (documentado, no silencioso).

use kinematics::GeometryPack;
use thiserror::Error;

/// Error de inercia.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum InertiaError {
    /// Parámetros inválidos (J < 0, masa < 0, no finitos).
    #[error("invalid inertia parameters: {reason}")]
    InvalidParameters { reason: String },
}

/// Parámetros de inercia separados de la topología (deuda S06 saldada).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InertiaParams {
    /// Inercia rotativa (cigüeñal + volante + polea), kg·m².
    pub rotating_j_kg_m2: f64,
    /// Masa alternativa por cilindro (pistón + parte de biela), kg.
    pub recip_mass_kg: f64,
}

impl InertiaParams {
    /// # Errors
    ///
    /// `InvalidParameters` si J < 0, masa < 0 o no finitos.
    pub fn new(rotating_j_kg_m2: f64, recip_mass_kg: f64) -> Result<Self, InertiaError> {
        if !(rotating_j_kg_m2.is_finite() && rotating_j_kg_m2 >= 0.0) {
            return Err(InertiaError::InvalidParameters {
                reason: "rotating J must be finite and >= 0".to_owned(),
            });
        }
        if !(recip_mass_kg.is_finite() && recip_mass_kg >= 0.0) {
            return Err(InertiaError::InvalidParameters {
                reason: "reciprocating mass must be finite and >= 0".to_owned(),
            });
        }
        Ok(Self {
            rotating_j_kg_m2,
            recip_mass_kg,
        })
    }
}

/// Inercia efectiva media (kg·m²) por equivalencia energética.
#[must_use]
pub fn effective_inertia_mean(geom: &GeometryPack, params: InertiaParams) -> f64 {
    let n = 720usize;
    let mut sum = 0.0;
    for i in 0..n {
        let th = f64::from(i as u32) * 2.0 * std::f64::consts::PI / n as f64;
        let dx = geom.displacement_dtheta(th);
        sum += dx * dx;
    }
    params.rotating_j_kg_m2 + params.recip_mass_kg * sum / n as f64
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn separates_topology_from_mass_and_rejects_bad_inputs() {
        let geom = GeometryPack::illustrative_d16_like().unwrap();
        let bare = InertiaParams::new(0.15, 0.0).unwrap();
        assert_eq!(effective_inertia_mean(&geom, bare), 0.15);
        let loaded = InertiaParams::new(0.15, 0.5).unwrap();
        assert!(effective_inertia_mean(&geom, loaded) > 0.15);
        assert!(InertiaParams::new(-1.0, 0.0).is_err());
        assert!(InertiaParams::new(0.15, -0.1).is_err());
    }
}
