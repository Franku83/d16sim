//! Mezcla simple: `R` y `γ` explícitos (S09). El `cp(T, mezcla)` real,
//! residuales y especies llegan en S10+.

use thiserror::Error;

/// Error de mezcla/estado.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum MixtureError {
    /// Parámetro no físico (R ≤ 0, γ ≤ 1 o no finitos).
    #[error("invalid mixture parameter: {reason}")]
    InvalidParameter { reason: String },
}

/// Gas ideal con calores específicos constantes (documentado como
/// simplificación S09; la dependencia T/mezcla es S10+).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mixture {
    r_j_per_kg_k: f64,
    gamma: f64,
}

impl Mixture {
    /// Construye con `R` y `γ` explícitos del llamante.
    ///
    /// # Errors
    ///
    /// `InvalidParameter` si `R ≤ 0`, `γ ≤ 1` o no finitos.
    pub fn new(r_j_per_kg_k: f64, gamma: f64) -> Result<Self, MixtureError> {
        let bad = |reason: &str| MixtureError::InvalidParameter {
            reason: reason.to_owned(),
        };
        if !(r_j_per_kg_k.is_finite() && r_j_per_kg_k > 0.0) {
            return Err(bad("R must be finite and > 0"));
        }
        if !(gamma.is_finite() && gamma > 1.0) {
            return Err(bad("gamma must be finite and > 1"));
        }
        Ok(Self {
            r_j_per_kg_k,
            gamma,
        })
    }

    #[must_use]
    pub fn r(&self) -> f64 {
        self.r_j_per_kg_k
    }

    #[must_use]
    pub fn gamma(&self) -> f64 {
        self.gamma
    }

    /// cv = R/(γ−1) (J/kg·K).
    #[must_use]
    pub fn cv(&self) -> f64 {
        self.r_j_per_kg_k / (self.gamma - 1.0)
    }

    /// cp = γ·R/(γ−1) (J/kg·K).
    #[must_use]
    pub fn cp(&self) -> f64 {
        self.gamma * self.r_j_per_kg_k / (self.gamma - 1.0)
    }
}
