//! Bomba y alivio (S22): desplazamiento positivo con eficiencia y
//! bypass por presión. Sin mapa medido Q–ΔP (curva pendiente S01/S25).

use thiserror::Error;

/// Error de bomba.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum PumpError {
    /// Parámetros inválidos (desplazamiento, eficiencia, alivio, rpm).
    #[error("invalid pump input: {reason}")]
    Invalid { reason: String },
}

/// Bomba de desplazamiento positivo (ilustrativa, sin mapa medido).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pump {
    /// Desplazamiento (m³/rev).
    pub displacement_m3_rev: f64,
    /// Eficiencia volumétrica [0, 1] (desgaste ⇒ menor).
    pub efficiency: f64,
    /// Presión de alivio (Pa abs): sobre ella deriva caudal.
    pub relief_p_pa: f64,
}

impl Pump {
    /// Bomba ilustrativa (8 cc/rev, eff 0.9, alivio 500 kPa abs).
    #[must_use]
    pub fn illustrative() -> Self {
        Self {
            displacement_m3_rev: 8e-6,
            efficiency: 0.9,
            relief_p_pa: 500_000.0,
        }
    }

    /// Caudal teórico (m³/s) a rpm dadas.
    ///
    /// # Errors
    ///
    /// `Invalid` si desplazamiento/eficiencia/rpm violan dominio.
    pub fn flow_m3_s(&self, rpm: f64) -> Result<f64, PumpError> {
        let bad = PumpError::Invalid {
            reason: "pump flow".to_owned(),
        };
        if !(self.displacement_m3_rev.is_finite() && self.displacement_m3_rev > 0.0) {
            return Err(bad);
        }
        if !(self.efficiency.is_finite() && (0.0..=1.0).contains(&self.efficiency)) {
            return Err(bad);
        }
        if !(rpm.is_finite() && rpm >= 0.0) {
            return Err(bad);
        }
        Ok(self.displacement_m3_rev * rpm / 60.0 * self.efficiency)
    }
}

/// Estado de bomba para escenarios (sana/degradada).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PumpState {
    pub pump: Pump,
    /// Nivel de aceite [0, 1] (bajo el pickup ⇒ aireación).
    pub oil_level_01: f64,
}

impl PumpState {
    /// Aireación por bajo nivel: pierde eficiencia efectiva linealmente
    /// bajo el 25% (modelo L1 declarado, no medición).
    #[must_use]
    pub fn aeration_factor(&self) -> f64 {
        if self.oil_level_01 >= 0.25 {
            1.0
        } else {
            (self.oil_level_01 / 0.25).max(0.0)
        }
    }
}
