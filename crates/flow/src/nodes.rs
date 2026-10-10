//! Volúmenes 0D (S12): masa, energía interna, temperatura y composición.
//!
//! Dos especies L1: fresca y residual/quemada (fracciones que suman 1).
//! `p = m·R·T/V` por construcción; `U = m·cv·T`.

use thiserror::Error;

/// Error de volumen.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum VolumeError {
    /// Masa, temperatura o volumen no positivos/finitos.
    #[error("invalid volume state: {reason}")]
    InvalidState { reason: String },
}

/// Nodo de volumen fijo con dos especies.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Volume {
    m_kg: f64,
    t_k: f64,
    v_m3: f64,
    fresh_frac: f64,
    r_gas: f64,
    cv: f64,
}

impl Volume {
    /// Construye validando positividad y fracción en [0, 1].
    ///
    /// # Errors
    ///
    /// `InvalidState` si m/T/V no son `> 0` finitos, la fracción sale de
    /// [0, 1] o R/cv no son `> 0` finitos.
    pub fn new(
        m_kg: f64,
        t_k: f64,
        v_m3: f64,
        fresh_frac: f64,
        r_gas: f64,
        cv: f64,
    ) -> Result<Self, VolumeError> {
        let bad = |reason: &str| VolumeError::InvalidState {
            reason: reason.to_owned(),
        };
        for (v, name) in [(m_kg, "mass"), (t_k, "temperature"), (v_m3, "volume")] {
            if !(v.is_finite() && v > 0.0) {
                return Err(bad(name));
            }
        }
        if !(fresh_frac.is_finite() && (0.0..=1.0).contains(&fresh_frac)) {
            return Err(bad("fresh_frac"));
        }
        if !((r_gas.is_finite() && r_gas > 0.0) && (cv.is_finite() && cv > 0.0)) {
            return Err(bad("gas properties"));
        }
        Ok(Self {
            m_kg,
            t_k,
            v_m3,
            fresh_frac,
            r_gas,
            cv,
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
    pub fn fresh_frac(self) -> f64 {
        self.fresh_frac
    }

    /// Presión (Pa) por gas ideal.
    #[must_use]
    pub fn p_pa(self) -> f64 {
        self.m_kg * self.r_gas * self.t_k / self.v_m3
    }

    /// Entalpía específica de salida (J/kg) con la T del nodo.
    #[must_use]
    pub fn h_out(self, cp: f64) -> f64 {
        cp * self.t_k
    }

    /// Aplica `dm` (kg, con fracción fresca `f_in`) y `du` (J) en un paso.
    ///
    /// # Errors
    ///
    /// `InvalidState` si la masa resultante no es `> 0` finita o la
    /// fracción sale de [0, 1] (el llamante usa pasos que lo garantizan;
    /// el error es fail-closed, no corrección silenciosa).
    pub fn step(&mut self, dm_kg: f64, f_in: f64, du_j: f64) -> Result<(), VolumeError> {
        let bad = |reason: &str| VolumeError::InvalidState {
            reason: reason.to_owned(),
        };
        if !(dm_kg.is_finite() && du_j.is_finite() && f_in.is_finite()) {
            return Err(bad("non-finite step"));
        }
        let m1 = self.m_kg + dm_kg;
        if !(m1.is_finite() && m1 > 0.0) {
            return Err(bad("non-positive mass"));
        }
        let f1 = if dm_kg >= 0.0 {
            (self.m_kg * self.fresh_frac + dm_kg * f_in) / m1
        } else {
            self.fresh_frac
        };
        if !(0.0..=1.0).contains(&f1) {
            return Err(bad("fresh fraction out of range"));
        }
        let u1 = self.m_kg * self.cv * self.t_k + du_j;
        self.m_kg = m1;
        self.fresh_frac = f1;
        self.t_k = u1 / (m1 * self.cv);
        if !(self.t_k.is_finite() && self.t_k > 0.0) {
            return Err(bad("non-positive temperature"));
        }
        Ok(())
    }
}
