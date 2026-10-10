//! Medición con corrección elemental (modelo L1 declarado).
//!
//! `corrected = reading − zero_error`; incertidumbre = resolución/2 +
//! |coef_temp·(T−20 °C)|. Sin curvas de instrumento inventadas.

use units::float::{require_finite, UnitsError};

/// Lectura de un instrumento dimensional.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Measurement {
    /// Lectura bruta (m).
    pub reading_m: f64,
    /// Error de cero (m).
    pub zero_error_m: f64,
    /// Resolución (m).
    pub resolution_m: f64,
    /// Temperatura de la pieza (°C).
    pub temperature_c: f64,
    /// Coeficiente térmico de corrección (m/°C, 0 si se ignora).
    pub temp_coeff_m_per_c: f64,
}

/// Resultado corregido con incertidumbre expandida simple (k=1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CorrectedReading {
    pub value_m: f64,
    pub uncertainty_m: f64,
}

/// Corrige una lectura según el modelo L1.
///
/// # Errors
///
/// `NonFinite` si alguna entrada no es finita; `OutOfRange` si la
/// resolución es negativa.
pub fn correct_reading(m: Measurement) -> Result<CorrectedReading, UnitsError> {
    let reading = require_finite(m.reading_m, "reading")?;
    let zero = require_finite(m.zero_error_m, "zero_error")?;
    let res = require_finite(m.resolution_m, "resolution")?;
    let temp = require_finite(m.temperature_c, "temperature")?;
    let coeff = require_finite(m.temp_coeff_m_per_c, "temp_coeff")?;
    if res < 0.0 {
        return Err(UnitsError::OutOfRange {
            what: "resolution",
            value: res,
        });
    }
    Ok(CorrectedReading {
        value_m: reading - zero,
        uncertainty_m: res / 2.0 + coeff.abs() * (temp - 20.0).abs(),
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn correction_and_uncertainty_are_additive() {
        let r = correct_reading(Measurement {
            reading_m: 0.075_02,
            zero_error_m: 0.000_01,
            resolution_m: 0.000_01,
            temperature_c: 20.0,
            temp_coeff_m_per_c: 0.0,
        })
        .unwrap();
        assert!((r.value_m - 0.075_01).abs() < 1e-12);
        assert!((r.uncertainty_m - 0.000_005).abs() < 1e-12);
    }
}
