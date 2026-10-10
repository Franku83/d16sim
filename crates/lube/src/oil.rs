//! Aceite (S22): viscosidad Walther (ASTM D341) con dos puntos.
//!
//! `log10(log10(ν + 0.7)) = A − B·log10(T_K)` con ν en cSt. Grado
//! ilustrativo orden 10W-30 (NO medido en el D16Y7). Densidad constante
//! ilustrativa.

use thiserror::Error;

/// Error de aceite.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum OilError {
    /// Viscosidades o temperatura inválidas.
    #[error("invalid oil input: {reason}")]
    Invalid { reason: String },
}

/// Grado con dos puntos cinemáticos + densidad.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OilGrade {
    /// ν a 40 °C (cSt).
    pub nu_40_cst: f64,
    /// ν a 100 °C (cSt).
    pub nu_100_cst: f64,
    /// Densidad (kg/m³).
    pub density_kg_m3: f64,
}

/// Ecuación de Walther ajustada a los dos puntos.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Walther {
    a: f64,
    b: f64,
    density_kg_m3: f64,
}

impl OilGrade {
    /// 10W-30 ilustrativo (70/10.5 cSt, 870 kg/m³): orden de magnitud de
    /// ficha comercial, NO aceite medido del motor de referencia.
    #[must_use]
    pub fn illustrative_10w30() -> Self {
        Self {
            nu_40_cst: 70.0,
            nu_100_cst: 10.5,
            density_kg_m3: 870.0,
        }
    }

    /// Ajusta Walther a los dos puntos.
    ///
    /// # Errors
    ///
    /// `Invalid` si los puntos no son `> 0` finitos o ν40 ≤ ν100.
    pub fn walther(&self) -> Result<Walther, OilError> {
        let bad = OilError::Invalid {
            reason: "walther points".to_owned(),
        };
        if !(self.nu_40_cst.is_finite()
            && self.nu_100_cst.is_finite()
            && self.nu_40_cst > self.nu_100_cst
            && self.nu_100_cst > 0.0)
        {
            return Err(bad);
        }
        if !(self.density_kg_m3.is_finite() && self.density_kg_m3 > 0.0) {
            return Err(bad);
        }
        let y40 = (self.nu_40_cst + 0.7).log10().log10();
        let y100 = (self.nu_100_cst + 0.7).log10().log10();
        let x40 = 313.15f64.log10();
        let x100 = 373.15f64.log10();
        // y = A − B·x con dos puntos: B = (y40−y100)/(x100−x40).
        let b = (y40 - y100) / (x100 - x40);
        let a = y40 + b * x40;
        Ok(Walther {
            a,
            b,
            density_kg_m3: self.density_kg_m3,
        })
    }
}

impl Walther {
    /// Viscosidad cinemática (cSt) a T (°C). Decrece con T; siempre > 0.
    #[must_use]
    pub fn nu_cst(&self, temp_c: f64) -> f64 {
        let t_k = (temp_c + 273.15).max(1.0);
        let y = self.a - self.b * t_k.log10();
        10f64.powf(10f64.powf(y)) - 0.7
    }

    /// Viscosidad dinámica (Pa·s).
    #[must_use]
    pub fn mu_pa_s(&self, temp_c: f64) -> f64 {
        (self.nu_cst(temp_c) * 1e-6 * self.density_kg_m3).max(1e-9)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn walther_fits_points_and_falls_with_temperature() {
        let w = OilGrade::illustrative_10w30().walther().unwrap();
        // Ajuste exacto en los dos puntos de anclaje.
        assert!(
            (w.nu_cst(40.0) - 70.0).abs() < 0.5,
            "nu40={}",
            w.nu_cst(40.0)
        );
        assert!(
            (w.nu_cst(100.0) - 10.5).abs() < 0.2,
            "nu100={}",
            w.nu_cst(100.0)
        );
        // Monótona decreciente y positiva (límite de viscosidad).
        assert!(w.nu_cst(20.0) > w.nu_cst(40.0));
        assert!(w.nu_cst(150.0) > 0.0 && w.nu_cst(150.0) < w.nu_cst(100.0));
        assert!(w.mu_pa_s(80.0) > 0.0);
        assert!(OilGrade {
            nu_40_cst: 5.0,
            nu_100_cst: 10.0,
            density_kg_m3: 870.0
        }
        .walther()
        .is_err());
    }
}
