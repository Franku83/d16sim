//! Combustible/mezcla: AFR vs lambda codificados como tipos distintos.
//!
//! `lambda = AFR / AFR_estequiométrico`. El estequiométrico es parámetro
//! explícito (depende del combustible); no hay constante oculta.

use crate::float::{require_finite, UnitsError};

/// Relación aire-combustible másica (`> 0`).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Afr(f64);

/// Lambda (`> 0`): 1 = estequiométrico del combustible dado.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Lambda(f64);

impl Afr {
    /// # Errors
    ///
    /// `NonFinite` / `OutOfRange` si no es `> 0` finito.
    pub fn from_ratio(afr: f64) -> Result<Self, UnitsError> {
        let afr = require_finite(afr, "afr")?;
        if afr > 0.0 {
            Ok(Self(afr))
        } else {
            Err(UnitsError::OutOfRange {
                what: "afr",
                value: afr,
            })
        }
    }

    #[must_use]
    pub fn as_ratio(self) -> f64 {
        self.0
    }

    /// # Errors
    ///
    /// Si `stoich` no es `> 0` finita.
    pub fn to_lambda(self, stoich: Self) -> Result<Lambda, UnitsError> {
        Lambda::from_value(self.0 / stoich.0)
    }
}

impl Lambda {
    /// # Errors
    ///
    /// `NonFinite` / `OutOfRange` si no es `> 0` finito.
    pub fn from_value(l: f64) -> Result<Self, UnitsError> {
        let l = require_finite(l, "lambda")?;
        if l > 0.0 {
            Ok(Self(l))
        } else {
            Err(UnitsError::OutOfRange {
                what: "lambda",
                value: l,
            })
        }
    }

    #[must_use]
    pub fn as_value(self) -> f64 {
        self.0
    }

    /// # Errors
    ///
    /// Si `stoich` no es `> 0` finita.
    pub fn to_afr(self, stoich: Afr) -> Result<Afr, UnitsError> {
        Afr::from_ratio(self.0 * stoich.as_ratio())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn afr_lambda_roundtrip_with_explicit_stoich() {
        let stoich = Afr::from_ratio(14.7).unwrap();
        let afr = Afr::from_ratio(12.5).unwrap();
        let lambda = afr.to_lambda(stoich).unwrap();
        assert!((lambda.as_value() - 0.850_340_136_054_421_8).abs() < 1e-12);
        let back = lambda.to_afr(stoich).unwrap();
        assert!((back.as_ratio() - 12.5).abs() < 1e-12);
    }

    #[test]
    fn rejects_nonpositive() {
        assert!(Afr::from_ratio(0.0).is_err());
        assert!(Lambda::from_value(-1.0).is_err());
    }
}
