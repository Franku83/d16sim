//! Temperatura termodinámica: Kelvin en el core. °C solo en `display`.

use crate::float::{require_finite, UnitsError};

/// Temperatura absoluta en K (`>= 0`, finita).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Kelvin(f64);

impl Kelvin {
    /// # Errors
    ///
    /// `NonFinite` si no es finito; `OutOfRange` si negativo.
    pub fn from_kelvin(k: f64) -> Result<Self, UnitsError> {
        let k = require_finite(k, "temperature_K")?;
        if k >= 0.0 {
            Ok(Self(k))
        } else {
            Err(UnitsError::OutOfRange {
                what: "temperature_K",
                value: k,
            })
        }
    }

    #[must_use]
    pub fn as_kelvin(self) -> f64 {
        self.0
    }
}

impl std::ops::Sub for Kelvin {
    type Output = f64;
    fn sub(self, rhs: Self) -> f64 {
        self.0 - rhs.0
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn rejects_negative_and_nonfinite() {
        assert!(Kelvin::from_kelvin(-0.1).is_err());
        assert!(Kelvin::from_kelvin(f64::NAN).is_err());
        assert_eq!(Kelvin::from_kelvin(0.0).unwrap().as_kelvin(), 0.0);
    }
}
