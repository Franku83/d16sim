//! Fracción de masa quemada tipo Wiebe:
//! `x(θ) = 1 − exp[−a·((θ−θs)/Δθ)^(m+1)]` en `[θs, θs+Δθ]`.
//!
//! Completitud al final de ventana: `1 − e^(−a)` (a=5 ⇒ 99.33%).

use thiserror::Error;

/// Error de Wiebe.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum WiebeError {
    /// Parámetros inválidos (a ≤ 0, m < 0, Δθ ≤ 0 o no finitos).
    #[error("invalid Wiebe parameters: {reason}")]
    InvalidParameters { reason: String },
}

/// Función de Wiebe validada.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Wiebe {
    a: f64,
    m_exp: f64,
    theta_start_rad: f64,
    delta_theta_rad: f64,
}

impl Wiebe {
    /// Construye validando `a > 0`, `m ≥ 0`, `Δθ > 0` finitos.
    ///
    /// # Errors
    ///
    /// `InvalidParameters` si algún parámetro viola su dominio.
    pub fn new(
        a: f64,
        m_exp: f64,
        theta_start_rad: f64,
        delta_theta_rad: f64,
    ) -> Result<Self, WiebeError> {
        let bad = |reason: &str| WiebeError::InvalidParameters {
            reason: reason.to_owned(),
        };
        if !(a.is_finite() && a > 0.0) {
            return Err(bad("a must be finite and > 0"));
        }
        if !(m_exp.is_finite() && m_exp >= 0.0) {
            return Err(bad("m must be finite and >= 0"));
        }
        if !theta_start_rad.is_finite() {
            return Err(bad("theta_start must be finite"));
        }
        if !(delta_theta_rad.is_finite() && delta_theta_rad > 0.0) {
            return Err(bad("delta_theta must be finite and > 0"));
        }
        Ok(Self {
            a,
            m_exp,
            theta_start_rad,
            delta_theta_rad,
        })
    }

    /// Fracción quemada a `theta_rad` (0 antes, completitud al final).
    #[must_use]
    pub fn burned_fraction(&self, theta_rad: f64) -> f64 {
        let tau = (theta_rad - self.theta_start_rad) / self.delta_theta_rad;
        if tau <= 0.0 {
            0.0
        } else if tau >= 1.0 {
            1.0 - (-self.a).exp()
        } else {
            1.0 - (-self.a * tau.powf(self.m_exp + 1.0)).exp()
        }
    }

    /// Derivada analítica `dx/dθ` (1/rad); 0 fuera de la ventana.
    #[must_use]
    pub fn burned_rate(&self, theta_rad: f64) -> f64 {
        let tau = (theta_rad - self.theta_start_rad) / self.delta_theta_rad;
        if tau <= 0.0 || tau >= 1.0 {
            0.0
        } else {
            let e = (-self.a * tau.powf(self.m_exp + 1.0)).exp();
            self.a * (self.m_exp + 1.0) * tau.powf(self.m_exp) * e / self.delta_theta_rad
        }
    }

    #[must_use]
    pub fn theta_start_rad(&self) -> f64 {
        self.theta_start_rad
    }

    #[must_use]
    pub fn end_rad(&self) -> f64 {
        self.theta_start_rad + self.delta_theta_rad
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn shape_and_completeness() {
        let w = Wiebe::new(5.0, 2.0, 0.0, 1.0).unwrap();
        assert_eq!(w.burned_fraction(-0.5), 0.0);
        assert_eq!(w.burned_rate(-0.5), 0.0);
        // Completitud 1 − e^−5.
        assert!((w.burned_fraction(1.0) - 0.993_262_053_000_914_5).abs() < 1e-12);
        assert_eq!(w.burned_rate(1.0), 0.0);
        // Monótona creciente dentro de la ventana.
        let mut prev = 0.0;
        for i in 1..=20 {
            let x = w.burned_fraction(f64::from(i) / 20.0);
            assert!(x >= prev);
            prev = x;
        }
        // Tasa diferenciable: FD centrada coincide.
        let t = 0.4;
        let h = 1e-6;
        let num = (w.burned_fraction(t + h) - w.burned_fraction(t - h)) / (2.0 * h);
        assert!((w.burned_rate(t) - num).abs() < 1e-7);
    }

    #[test]
    fn invalid_rejected() {
        assert!(Wiebe::new(0.0, 2.0, 0.0, 1.0).is_err());
        assert!(Wiebe::new(5.0, -1.0, 0.0, 1.0).is_err());
        assert!(Wiebe::new(5.0, 2.0, 0.0, 0.0).is_err());
    }
}
