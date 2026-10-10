//! Presión: absoluta vs manométrica (gauge) vs diferencial.
//!
//! - [`PressureAbs`]: Pa absolutos, `>= 0`. Lo que la termodinámica usa.
//! - [`PressureDelta`]: diferencia de presión (Pa). Lo que un regulador o
//!   un sensor diferencial realmente imponen.
//! - El "gauge" solo existe como `PressureDelta` junto a un ambiente
//!   explícito: `abs.to_gauge(ambient)` / `delta.to_abs(ambient)`.
//!
//! No existe conversión silenciosa bar-gauge → Pa absoluto.

use crate::float::{require_finite, require_non_negative, UnitsError};

/// Presión absoluta en Pa (`>= 0`, finita).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PressureAbs(f64);

/// Diferencia de presión en Pa (finita, con signo).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PressureDelta(f64);

impl PressureAbs {
    /// # Errors
    ///
    /// `NonFinite` / `OutOfRange` si `pa` no es `>= 0` finito.
    pub fn from_pa(pa: f64) -> Result<Self, UnitsError> {
        Ok(Self(require_non_negative(pa, "pressure_abs_pa")?))
    }

    /// # Errors
    ///
    /// Igual que [`Self::from_pa`].
    pub fn from_kpa(kpa: f64) -> Result<Self, UnitsError> {
        Self::from_pa(kpa * 1_000.0)
    }

    /// # Errors
    ///
    /// Igual que [`Self::from_pa`].
    pub fn from_bar_abs(bar: f64) -> Result<Self, UnitsError> {
        Self::from_pa(bar * 100_000.0)
    }

    /// # Errors
    ///
    /// Igual que [`Self::from_pa`].
    pub fn from_psi_abs(psi: f64) -> Result<Self, UnitsError> {
        Self::from_pa(psi * 6_894.757_293_168)
    }

    #[must_use]
    pub fn as_pa(self) -> f64 {
        self.0
    }

    #[must_use]
    pub fn as_kpa(self) -> f64 {
        self.0 / 1_000.0
    }

    #[must_use]
    pub fn as_bar_abs(self) -> f64 {
        self.0 / 100_000.0
    }

    /// Lectura gauge frente a un ambiente **explícito**.
    #[must_use]
    pub fn to_gauge(self, ambient: Self) -> PressureDelta {
        PressureDelta(self.0 - ambient.0)
    }
}

impl PressureDelta {
    /// # Errors
    ///
    /// `NonFinite` si `pa` no es finito.
    pub fn from_pa(pa: f64) -> Result<Self, UnitsError> {
        Ok(Self(require_finite(pa, "pressure_delta_pa")?))
    }

    /// # Errors
    ///
    /// Igual que [`Self::from_pa`].
    pub fn from_kpa(kpa: f64) -> Result<Self, UnitsError> {
        Self::from_pa(kpa * 1_000.0)
    }

    /// # Errors
    ///
    /// Igual que [`Self::from_pa`].
    pub fn from_psi_diff(psi: f64) -> Result<Self, UnitsError> {
        Self::from_pa(psi * 6_894.757_293_168)
    }

    #[must_use]
    pub fn as_pa(self) -> f64 {
        self.0
    }

    /// Absoluta = ambiente + delta. Sin ambiente no hay absoluta.
    #[must_use]
    pub fn to_abs(self, ambient: PressureAbs) -> PressureAbs {
        // `to_abs` solo se llama con deltas provenientes de `to_gauge` o
        // validadas; el resultado físico es `>= 0`. Si un sensor roto da
        // delta < -ambiente, se satura a 0 en vez de presión negativa:
        // un instrumento no crea vacío absoluto imposible.
        PressureAbs((ambient.as_pa() + self.0).max(0.0))
    }
}

impl std::ops::Add<PressureDelta> for PressureAbs {
    type Output = Self;
    fn add(self, rhs: PressureDelta) -> Self {
        Self((self.0 + rhs.0).max(0.0))
    }
}

impl std::ops::Sub for PressureAbs {
    type Output = PressureDelta;
    fn sub(self, rhs: Self) -> PressureDelta {
        PressureDelta(self.0 - rhs.0)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    /// Generador determinista (LCG) para barridos sin dependencias.
    fn lcg(state: &mut u64) -> f64 {
        *state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        // [0, 1)
        ((*state >> 11) as f64) / ((1u64 << 53) as f64)
    }

    #[test]
    fn abs_gauge_roundtrip_over_range() {
        let mut s = 0x1234_5678_9abc_def1;
        for _ in 0..2000 {
            let ambient = 80_000.0 + lcg(&mut s) * 40_000.0;
            let abs = 20_000.0 + lcg(&mut s) * 400_000.0;
            let a = PressureAbs::from_pa(abs).unwrap();
            let amb = PressureAbs::from_pa(ambient).unwrap();
            let back = a.to_gauge(amb).to_abs(amb);
            assert!((back.as_pa() - abs).abs() <= 1e-9 * abs.max(1.0));
        }
    }

    #[test]
    fn unit_factors_are_consistent() {
        let p = PressureAbs::from_bar_abs(1.0).unwrap();
        assert!((p.as_kpa() - 100.0).abs() < 1e-9);
        let q = PressureAbs::from_psi_abs(14.503_773_773).unwrap();
        assert!((q.as_bar_abs() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn negative_abs_rejected() {
        assert!(PressureAbs::from_pa(-1.0).is_err());
    }
}
