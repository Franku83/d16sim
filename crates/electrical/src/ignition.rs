//! Bobina/igniter/dwell simplificada (S15): rampa de corriente y evento
//! de chispa DISPONIBLE (no programado: el scheduler es S18).
//!
//! `I = V·t/L`, `E = ½·L·I²`; chispa disponible si `E ≥ E_min`. Sin tiempo
//! de quemado, sin avance, sin energía de arco (deuda).

use thiserror::Error;

/// Error de encendido.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum IgnitionError {
    /// Parámetros inválidos (L, dwell, tensión, umbral).
    #[error("invalid ignition input: {reason}")]
    InvalidInput { reason: String },
}

/// Bobina + igniter con dwell comandado.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coil {
    /// Inductancia primaria (H, ilustrativa).
    pub inductance_h: f64,
    /// Energía mínima de chispa (J, umbral ilustrativo de disponibilidad).
    pub min_spark_energy_j: f64,
    /// Dwell comandado (s, 0 = sin carga).
    pub dwell_s: f64,
}

impl Coil {
    /// Bobina ilustrativa (L = 5 mH, umbral 30 mJ, dwell a comandar).
    #[must_use]
    pub fn illustrative() -> Self {
        Self {
            inductance_h: 5e-3,
            min_spark_energy_j: 30e-3,
            dwell_s: 0.0,
        }
    }

    /// Evento de chispa para tensión de carga dada.
    ///
    /// # Errors
    ///
    /// `InvalidInput` si L/umbral/tensión no son válidos.
    pub fn spark_at(&self, charge_volts: f64) -> Result<SparkEvent, IgnitionError> {
        let bad = |reason: &str| IgnitionError::InvalidInput {
            reason: reason.to_owned(),
        };
        if !(self.inductance_h.is_finite() && self.inductance_h > 0.0) {
            return Err(bad("inductance"));
        }
        if !(self.min_spark_energy_j.is_finite() && self.min_spark_energy_j >= 0.0) {
            return Err(bad("threshold"));
        }
        if !(charge_volts.is_finite() && charge_volts >= 0.0) {
            return Err(bad("voltage"));
        }
        if !(self.dwell_s.is_finite() && self.dwell_s >= 0.0) {
            return Err(bad("dwell"));
        }
        let current = charge_volts * self.dwell_s / self.inductance_h;
        let energy = 0.5 * self.inductance_h * current * current;
        Ok(SparkEvent {
            available: energy >= self.min_spark_energy_j && self.dwell_s > 0.0,
            energy_j: energy,
            dwell_s: self.dwell_s,
        })
    }
}

/// Evento de chispa disponible (el CUÁNDO lo programa S18).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SparkEvent {
    pub available: bool,
    pub energy_j: f64,
    pub dwell_s: f64,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn spark_needs_dwell_and_voltage() {
        let mut coil = Coil::illustrative();
        // Sin dwell no hay chispa aunque haya tensión (chispa nula).
        assert!(!coil.spark_at(12.8).unwrap().available);
        // 3 ms a 12.8 V: I = 7.68 A, E = 147 mJ ≥ 30 mJ.
        coil.dwell_s = 3e-3;
        let s = coil.spark_at(12.8).unwrap();
        assert!(s.available);
        assert!((s.energy_j - 0.147_456).abs() < 1e-9);
        // Tensión baja colapsa la energía (~V²).
        let weak = coil.spark_at(6.0).unwrap();
        assert!(weak.energy_j < s.energy_j);
        assert!((weak.energy_j / s.energy_j - (6.0 / 12.8f64).powi(2)).abs() < 1e-12);
    }
}
