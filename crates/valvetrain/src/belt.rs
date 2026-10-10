//! Correa de distribución (S11): relación 2:1 y offset configurable.
//!
//! Un diente desplazado en la polea del árbol (`teeth_total` dientes)
//! desfasa la leva `offset_teeth/teeth_total × 360°` de árbol, es decir el
//! doble en grados de cigüeñal. El cigüeñal y la secuencia mecánica no
//! cambian: solo la fase de eventos de válvula.

use thiserror::Error;

/// Error de correa.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum BeltError {
    /// Dientes totales ≤ 0 o no finitos los parámetros.
    #[error("invalid belt drive: {reason}")]
    InvalidDrive { reason: String },
}

/// Transmisión por correa con offset de montaje.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BeltDrive {
    teeth_total: f64,
    offset_teeth: f64,
}

impl BeltDrive {
    /// Construye con dientes totales y offset (puede ser fraccionario para
    /// estudios; el caso físico es entero).
    ///
    /// # Errors
    ///
    /// `InvalidDrive` si `teeth_total ≤ 0` o hay no finitos.
    pub fn new(teeth_total: f64, offset_teeth: f64) -> Result<Self, BeltError> {
        if !(teeth_total.is_finite() && teeth_total > 0.0) {
            return Err(BeltError::InvalidDrive {
                reason: "teeth_total must be finite and > 0".to_owned(),
            });
        }
        if !offset_teeth.is_finite() {
            return Err(BeltError::InvalidDrive {
                reason: "offset must be finite".to_owned(),
            });
        }
        Ok(Self {
            teeth_total,
            offset_teeth,
        })
    }

    /// Desfase de leva en grados de CIGÜEÑAL (2:1): + = avance de eventos.
    #[must_use]
    pub fn cam_shift_crank_deg(&self) -> f64 {
        self.offset_teeth / self.teeth_total * 720.0
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn one_tooth_shifts_events_without_touching_crank() {
        // 1 diente en 100 ⇒ 7.2° de cigüeñal (ilustrativo; el nº de dientes
        // OEM es PENDING y va como entrada).
        let b = BeltDrive::new(100.0, 1.0).unwrap();
        assert!((b.cam_shift_crank_deg() - 7.2).abs() < 1e-12);
        assert_eq!(
            BeltDrive::new(100.0, 0.0).unwrap().cam_shift_crank_deg(),
            0.0
        );
        assert!(BeltDrive::new(0.0, 1.0).is_err());
    }
}
