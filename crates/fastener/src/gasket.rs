//! Campo de presión culata–junta–bloque por segmentos (rigidez
//! equivalente L1, no FEM) y costura de conductancia para S23.
//!
//! El reparto es proporcional a pesos de rigidez dados por el llamante
//! (uniformes por defecto, declarados ilustrativos). La secuencia de
//! apriete queda registrada pero **no** distorsiona el campo: sin modelo
//! de interacción elástica, fingirla sería un multiplicador oculto (S08).

use thiserror::Error;

/// Error del modelo de junta.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum GasketError {
    /// Entradas inválidas (áreas/pesos no positivos o desajuste).
    #[error("invalid gasket input: {reason}")]
    InvalidInput { reason: String },
}

/// Estado de sellado por segmento: en S08 siempre indeterminado (el sellado
/// real exige ley de fuga + presión/temperatura locales en S23).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SealStatus {
    SealingIndeterminate,
}

/// Segmento de sellado (anillo de fuego, aceite, refrigerante...).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SealSegment {
    /// Área de contacto del segmento (m², entrada del llamante).
    pub area_m2: f64,
    /// Peso de rigidez relativo (adimensional, > 0).
    pub stiffness_weight: f64,
}

/// Conductancia de fuga entre un par de volúmenes: costura futura S23.
/// El valor real lo calculará el solver de fugas desde `SealContactState`;
/// aquí solo existe el tipo para no hardcodear fugas después.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SealConductance {
    /// Aún sin modelo de fuga: S23 la implementará.
    Unsupported { reason: &'static str },
}

/// Campo de presión de contacto resuelto (L1).
#[derive(Debug, Clone, PartialEq)]
pub struct GasketField {
    /// Fuerza total de clamp (N, suma de precargas nominales).
    pub total_clamp_n: f64,
    /// Presión de contacto por segmento (Pa).
    pub contact_pressure_pa: Vec<f64>,
    /// No-uniformidad max/min del campo (≥ 1).
    pub non_uniformity: f64,
    /// Estado por segmento (siempre indeterminado en S08).
    pub status: Vec<SealStatus>,
}

/// Reparte la carga total sobre los segmentos según sus pesos.
///
/// # Errors
///
/// `InvalidInput` si no hay segmentos, áreas/pesos no finitos o no
/// positivos, o la carga total no es finita/negativa.
pub fn distribute_clamp(
    total_clamp_n: f64,
    segments: &[SealSegment],
) -> Result<GasketField, GasketError> {
    let bad = |reason: &str| GasketError::InvalidInput {
        reason: reason.to_owned(),
    };
    if segments.is_empty() {
        return Err(bad("need >= 1 segment"));
    }
    if !total_clamp_n.is_finite() || total_clamp_n < 0.0 {
        return Err(bad("total clamp must be finite and >= 0"));
    }
    let mut weight_sum = 0.0;
    for s in segments {
        if !(s.area_m2.is_finite() && s.area_m2 > 0.0) {
            return Err(bad("segment area must be finite and > 0"));
        }
        if !(s.stiffness_weight.is_finite() && s.stiffness_weight > 0.0) {
            return Err(bad("stiffness weight must be finite and > 0"));
        }
        weight_sum += s.stiffness_weight;
    }
    let pressures: Vec<f64> = segments
        .iter()
        .map(|s| total_clamp_n * (s.stiffness_weight / weight_sum) / s.area_m2)
        .collect();
    let max = pressures.iter().fold(0.0f64, |a, b| a.max(*b));
    let min = pressures.iter().fold(f64::INFINITY, |a, b| a.min(*b));
    Ok(GasketField {
        total_clamp_n,
        contact_pressure_pa: pressures,
        non_uniformity: if min > 0.0 { max / min } else { f64::INFINITY },
        status: vec![SealStatus::SealingIndeterminate; segments.len()],
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn fire_rings() -> Vec<SealSegment> {
        vec![
            SealSegment {
                area_m2: 400e-6,
                stiffness_weight: 1.0,
            },
            SealSegment {
                area_m2: 400e-6,
                stiffness_weight: 1.0,
            },
            SealSegment {
                area_m2: 400e-6,
                stiffness_weight: 1.0,
            },
            SealSegment {
                area_m2: 400e-6,
                stiffness_weight: 1.0,
            },
        ]
    }

    #[test]
    fn uniform_field_and_undertorque_non_uniformity() {
        // 4 anillos iguales, 100 kN ⇒ 62.5 MPa por anillo.
        let even = distribute_clamp(100_000.0, &fire_rings()).unwrap();
        for p in &even.contact_pressure_pa {
            assert!((*p - 62.5e6).abs() < 1e-3);
        }
        assert!((even.non_uniformity - 1.0).abs() < 1e-12);
        // Subapriete modelado como menor carga en un cuadrante: el campo lo
        // muestra como no-uniformidad observada, no como distorsión impuesta.
        let uneven = distribute_clamp(
            100_000.0,
            &[
                SealSegment {
                    area_m2: 400e-6,
                    stiffness_weight: 0.5,
                },
                SealSegment {
                    area_m2: 400e-6,
                    stiffness_weight: 1.0,
                },
                SealSegment {
                    area_m2: 400e-6,
                    stiffness_weight: 1.0,
                },
                SealSegment {
                    area_m2: 400e-6,
                    stiffness_weight: 1.0,
                },
            ],
        )
        .unwrap();
        assert!(uneven.non_uniformity > even.non_uniformity);
        assert!(uneven
            .status
            .iter()
            .all(|s| *s == SealStatus::SealingIndeterminate));
    }

    #[test]
    fn conductance_seam_is_unsupported_until_s23() {
        let c = SealConductance::Unsupported {
            reason: "leakage solver lands in S23",
        };
        assert!(matches!(c, SealConductance::Unsupported { .. }));
        assert!(distribute_clamp(1.0, &[]).is_err());
    }
}
