//! Batería L1 (S15): Voc(SOC, T), resistencia interna, tensión en bornes.
//!
//! Parámetros ilustrativos de batería de arranque 12 V (no medidos en un
//! D16Y7 concreto). Sin envejecimiento ni estratificación (deuda).

use thiserror::Error;

/// Error de batería.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum BatteryError {
    /// SOC, temperatura o corriente fuera de dominio.
    #[error("invalid battery input: {reason}")]
    InvalidInput { reason: String },
}

/// Batería de arranque.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Battery {
    /// Estado de carga [0, 1].
    pub soc: f64,
    /// Temperatura (°C).
    pub temp_c: f64,
    /// Tensión a plena carga y 20 °C (V).
    pub v_full: f64,
    /// Caída lineal con descarga (V).
    pub v_slope: f64,
    /// Coeficiente térmico (V/°C).
    pub v_temp_coeff: f64,
    /// Resistencia interna base (Ω).
    pub r0_ohm: f64,
    /// Resistencia extra de masa/cableado (Ω, ∞ ≈ masa abierta).
    pub r_ground_ohm: f64,
}

impl Battery {
    /// Batería ilustrativa 12 V (12.8 V llena, R0 8 mΩ, masa 2 mΩ).
    #[must_use]
    pub fn illustrative_12v() -> Self {
        Self {
            soc: 1.0,
            temp_c: 20.0,
            v_full: 12.8,
            v_slope: 1.8,
            v_temp_coeff: -0.01,
            r0_ohm: 0.008,
            r_ground_ohm: 0.002,
        }
    }

    /// Valida el estado/parámetros de la batería (R02-C2/C1-01).
    ///
    /// # Errors
    ///
    /// `InvalidInput` si `soc`/`temp` o los parámetros eléctricos no son
    /// finitos o violan dominio.
    pub fn validated(&self) -> Result<(), BatteryError> {
        let bad = |reason: &str| BatteryError::InvalidInput {
            reason: reason.to_owned(),
        };
        if !(self.soc.is_finite() && (0.0..=1.0).contains(&self.soc)) {
            return Err(bad("soc must be in [0, 1]"));
        }
        for (v, name) in [
            (self.temp_c, "temp"),
            (self.v_full, "v_full"),
            (self.v_slope, "v_slope"),
            (self.v_temp_coeff, "v_temp_coeff"),
            (self.r0_ohm, "r0"),
            (self.r_ground_ohm, "r_ground"),
        ] {
            if !v.is_finite() {
                return Err(bad(name));
            }
        }
        if self.r0_ohm < 0.0 || self.r_ground_ohm < 0.0 {
            return Err(bad("resistance must be >= 0"));
        }
        Ok(())
    }

    /// Tensión de circuito abierto (V).
    ///
    /// # Errors
    ///
    /// `InvalidInput` si el estado es inconsistente (no finito).
    pub fn voc(&self) -> Result<f64, BatteryError> {
        if !(self.soc.is_finite() && (0.0..=1.0).contains(&self.soc)) {
            return Err(BatteryError::InvalidInput {
                reason: "soc must be in [0, 1]".to_owned(),
            });
        }
        Ok(
            self.v_full - self.v_slope * (1.0 - self.soc)
                + self.v_temp_coeff * (self.temp_c - 20.0),
        )
    }

    /// Resistencia interna total (Ω): crece al descargar y en frío.
    ///
    /// # Errors
    ///
    /// `InvalidInput` si algún parámetro no es finito/negativo.
    pub fn resistance(&self) -> Result<f64, BatteryError> {
        for (v, name) in [
            (self.r0_ohm, "r0"),
            (self.r_ground_ohm, "r_ground"),
            (self.temp_c, "temp"),
        ] {
            if !(v.is_finite() && v >= 0.0) {
                return Err(BatteryError::InvalidInput {
                    reason: name.to_owned(),
                });
            }
        }
        let cold = 1.0 + 0.02 * (20.0 - self.temp_c).max(0.0);
        Ok((self.r0_ohm * (1.0 + 1.5 * (1.0 - self.soc)) * cold) + self.r_ground_ohm)
    }

    /// Tensión en bornes bajo corriente I (V). Puede colapsar con SOC bajo
    /// o masa abierta: es el mecanismo de "batería descargada", no un flag.
    ///
    /// # Errors
    ///
    /// Propaga `voc`/`resistance`.
    pub fn terminal_voltage(&self, current_a: f64) -> Result<f64, BatteryError> {
        if !(current_a.is_finite() && current_a >= 0.0) {
            return Err(BatteryError::InvalidInput {
                reason: "current must be finite and >= 0".to_owned(),
            });
        }
        Ok((self.voc()? - current_a * self.resistance()?).max(0.0))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn voltage_collapses_under_load_when_weak() {
        let good = Battery::illustrative_12v();
        assert!((good.voc().unwrap() - 12.8).abs() < 1e-12);
        assert!(good.terminal_voltage(300.0).unwrap() > 9.0);
        // Descargada: Voc cae y R sube ⇒ colapso en bornes.
        let dead = Battery {
            soc: 0.0,
            ..Battery::illustrative_12v()
        };
        assert!(dead.voc().unwrap() < good.voc().unwrap());
        assert!(dead.terminal_voltage(300.0).unwrap() < good.terminal_voltage(300.0).unwrap());
        // Masa abierta (1 Ω): el terminal colapsa con carga.
        let open = Battery {
            r_ground_ohm: 1.0,
            ..Battery::illustrative_12v()
        };
        assert!(open.terminal_voltage(300.0).unwrap() < 1.0);
    }
}
