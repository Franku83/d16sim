//! Leyes de conductancia (S23): formulación explícita + evidencia.
//!
//! NINGUNA ley afirma predecir fugas reales: la de anillo es geométrica L1
//! (sin dinámica/asiento); la de junta es paramétrica NO calibrada con
//! sensibilidad declarada. juntos: dirección correcta garantizada por
//! construcción (más clamp ⇒ menos fuga), magnitud pendiente de S25.

use thiserror::Error;

/// Error de conductancia.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ConductanceError {
    /// Geometría o presiones inválidas.
    #[error("invalid conductance input: {reason}")]
    Invalid { reason: String },
}

/// Estado de contacto de sellado por segmento (viene de `fastener`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SealContactState {
    /// Presión de contacto (Pa).
    pub contact_pressure_pa: f64,
    /// Área del segmento (m²).
    pub area_m2: f64,
}

/// Área de fuga de junta por segmento (m²): forma explícita L1 con umbral
/// de sellado `p_seal_ref_pa` NO calibrado (S25: leak-down vs torque).
/// Monótona decreciente en contacto por construcción.
#[must_use]
pub fn gasket_leak_area(state: SealContactState, p_seal_ref_pa: f64) -> f64 {
    if !(p_seal_ref_pa.is_finite() && p_seal_ref_pa > 0.0) {
        return state.area_m2;
    }
    if !(state.contact_pressure_pa.is_finite() && state.area_m2.is_finite()) {
        return 0.0;
    }
    let ratio = (state.contact_pressure_pa / p_seal_ref_pa).clamp(0.0, 1.0);
    state.area_m2.max(0.0) * (1.0 - ratio).powi(2)
}

/// Área de fuga de anillo (m²): gap × profundidad radial (geométrica).
///
/// # Errors
///
/// `Invalid` si gap/profundidad no son `≥ 0` finitos.
pub fn annulus_leak_area(end_gap_m: f64, radial_depth_m: f64) -> Result<f64, ConductanceError> {
    if !(end_gap_m.is_finite() && radial_depth_m.is_finite()) {
        return Err(ConductanceError::Invalid {
            reason: "non-finite".to_owned(),
        });
    }
    if end_gap_m < 0.0 || radial_depth_m < 0.0 {
        return Err(ConductanceError::Invalid {
            reason: "negative".to_owned(),
        });
    }
    Ok(end_gap_m * radial_depth_m)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn gasket_law_direction_and_ring_geometry() {
        let seg = SealContactState {
            contact_pressure_pa: 100e6,
            area_m2: 400e-6,
        };
        // Más clamp ⇒ menos fuga (dirección garantizada por la forma).
        let a_lo = gasket_leak_area(
            SealContactState {
                contact_pressure_pa: 50e6,
                ..seg
            },
            200e6,
        );
        let a_hi = gasket_leak_area(seg, 200e6);
        assert!(a_lo > a_hi && a_hi > 0.0);
        // Contacto total ⇒ sellado; nulo ⇒ área completa.
        assert_eq!(
            gasket_leak_area(
                SealContactState {
                    contact_pressure_pa: 300e6,
                    ..seg
                },
                200e6,
            ),
            0.0
        );
        assert_eq!(
            gasket_leak_area(
                SealContactState {
                    contact_pressure_pa: 0.0,
                    ..seg
                },
                200e6,
            ),
            400e-6
        );
        // Anillo: gap × profundidad, exacto y monótono.
        assert_eq!(annulus_leak_area(0.3e-3, 3e-3).unwrap(), 0.9e-6);
        assert!(
            annulus_leak_area(1.5e-3, 3e-3).unwrap() > annulus_leak_area(0.3e-3, 3e-3).unwrap()
        );
        assert!(annulus_leak_area(-1.0, 3e-3).is_err());
    }
}
