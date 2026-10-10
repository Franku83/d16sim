//! Estado termodinámico homogéneo (0D): `p·V = m·R·T` por construcción.

use crate::mixture::{Mixture, MixtureError};
use thiserror::Error;

/// Error de estado.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum StateError {
    /// Masa, temperatura o volumen no positivos/finitos.
    #[error("invalid thermo state: {reason}")]
    InvalidState { reason: String },
}

/// Estado 0D consistente: la presión deriva de gas ideal, nunca se impone
/// por separado (evita `p ≠ mRT/V` silencioso).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThermoState {
    m_kg: f64,
    t_k: f64,
    v_m3: f64,
    p_pa: f64,
}

impl ThermoState {
    /// Construye desde masa, temperatura y volumen (`p = m·R·T/V`).
    ///
    /// # Errors
    ///
    /// `InvalidState` si m/T/V no son `> 0` finitos.
    pub fn from_m_t_v(m_kg: f64, t_k: f64, v_m3: f64, mix: Mixture) -> Result<Self, StateError> {
        let bad = |reason: &str| StateError::InvalidState {
            reason: reason.to_owned(),
        };
        for (v, name) in [(m_kg, "mass"), (t_k, "temperature"), (v_m3, "volume")] {
            if !(v.is_finite() && v > 0.0) {
                return Err(bad(name));
            }
        }
        Ok(Self {
            m_kg,
            t_k,
            v_m3,
            p_pa: m_kg * mix.r() * t_k / v_m3,
        })
    }

    #[must_use]
    pub fn m_kg(self) -> f64 {
        self.m_kg
    }

    #[must_use]
    pub fn t_k(self) -> f64 {
        self.t_k
    }

    #[must_use]
    pub fn v_m3(self) -> f64 {
        self.v_m3
    }

    #[must_use]
    pub fn p_pa(self) -> f64 {
        self.p_pa
    }
}

impl From<MixtureError> for StateError {
    fn from(_: MixtureError) -> Self {
        StateError::InvalidState {
            reason: "mixture".to_owned(),
        }
    }
}
