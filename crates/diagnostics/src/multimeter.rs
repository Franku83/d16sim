//! Multímetro (S20): rango, impedancia de entrada, resolución y medida
//! de resistencia con corriente de prueba (circuito sin tensión).
//!
//! La impedancia CARGA al circuito: `V_leída = V_src·Zin/(Zsrc+Zin)`.
//! Rango saturado ⇒ lectura al fondo de escala (documentado, no error).

use thiserror::Error;

/// Error de multímetro.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum MultimeterError {
    /// Parámetros inválidos (rango, impedancia, resolución).
    #[error("invalid multimeter setup: {reason}")]
    InvalidSetup { reason: String },
}

/// Error de medida de resistencia.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ResistanceError {
    /// El circuito tiene tensión: no se puede medir resistencia.
    #[error("circuit powered: cannot measure resistance")]
    CircuitPowered,
}

/// Multímetro con carga de entrada modelada.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Multimeter {
    /// Fondo de escala (V).
    pub range_v: f64,
    /// Impedancia de entrada (Ω, ilustrativa 10 MΩ).
    pub impedance_ohm: f64,
    /// Resolución (V).
    pub resolution_v: f64,
    /// Corriente de prueba en Ω (A, ilustrativa 1 mA).
    pub ohms_test_current_a: f64,
}

impl Multimeter {
    /// Típico 10 MΩ, 20 V, 1 mV (ilustrativo, no certificado).
    #[must_use]
    pub fn typical_10m() -> Self {
        Self {
            range_v: 20.0,
            impedance_ohm: 10e6,
            resolution_v: 1e-3,
            ohms_test_current_a: 1e-3,
        }
    }

    /// Mide tensión con carga del circuito (Zsrc explícita).
    #[must_use]
    pub fn measure_voltage(&self, v_src: f64, z_src_ohm: f64) -> f64 {
        let loaded = v_src * self.impedance_ohm / (z_src_ohm.max(0.0) + self.impedance_ohm);
        let clipped = loaded.clamp(-self.range_v, self.range_v);
        (clipped / self.resolution_v).round() * self.resolution_v
    }

    /// Mide resistencia (Ω) ante caída `v_drop` con su corriente; falla si
    /// el circuito tiene tensión propia (`v_live ≠ 0`).
    ///
    /// # Errors
    ///
    /// `CircuitPowered` si `v_live` no es ~0.
    pub fn measure_resistance(&self, v_drop: f64, v_live: f64) -> Result<f64, ResistanceError> {
        if v_live.abs() > 0.05 {
            return Err(ResistanceError::CircuitPowered);
        }
        Ok(v_drop / self.ohms_test_current_a)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn loading_range_resolution_and_powered_guard() {
        let m = Multimeter::typical_10m();
        // Fuente ideal: sin carga apreciable + cuantización a 1 mV.
        assert!((m.measure_voltage(2.34567, 0.0) - 2.346).abs() < 1e-12);
        // Fuente débil (100 kΩ): cae ~1% por división (+cuantización 1 mV).
        assert!((m.measure_voltage(5.0, 100e3) - 5.0 * 10e6 / 10.1e6).abs() < 1e-3);
        // Fuera de rango: satura al fondo de escala.
        assert_eq!(m.measure_voltage(30.0, 0.0), 20.0);
        // Resistencia: 0.33 V con 1 mA ⇒ 330 Ω (caso E01: banda 350–700).
        assert!((m.measure_resistance(0.33, 0.0).unwrap() - 330.0).abs() < 1e-9);
        assert!(m.measure_resistance(0.33, 12.0).is_err());
    }
}
