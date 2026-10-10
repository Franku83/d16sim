//! Leyes de desgaste S26: formulación explícita + soporte declarado.
//! Todas las constantes son ILUSTRATIVAS (sin tribología medida del
//! D16Y7): ordenan escenarios, no predicen kilómetros.

use thiserror::Error;

/// Error de ley.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum LawError {
    /// Parámetros o punto operativo inválidos (fail-closed).
    #[error("invalid wear input: {reason}")]
    Invalid { reason: String },
}

/// Estado de soporte de un mecanismo (terminología oficial S26).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Support {
    /// Ley con parámetros defendibles (aunque ilustrativos): da tendencia.
    TrendConditioned,
    /// Sin ley constitutiva / parámetro / evidencia: sin números.
    Unsupported,
}

/// Ley de desgaste con dominio y provenance.
#[derive(Debug, Clone, PartialEq)]
pub struct WearLaw {
    pub name: String,
    pub support: Support,
    pub provenance: String,
    /// Coeficiente ilustrativo con rango [lo, hi] (incertidumbre).
    pub coeff: f64,
    pub coeff_range: (f64, f64),
}

impl WearLaw {
    /// Archard anillo/camisa en contacto oiled: `dh = K·(1−film)·p·ds/H`.
    /// K EFECTIVO ilustrativo 1e-8 (orden µm/50h al punto nominal, P1):
    /// el K seco ingenuo (1e-4) con MPa daba METROS de desgaste en 50 h
    /// (fallo intermedio real de S26, documentado); la película soporta
    /// casi toda la carga y solo (1−film) va a asperezas. Tendencia, no
    /// vida útil.
    #[must_use]
    pub fn archard_ring() -> Self {
        Self {
            name: "archard-ring-liner-oiled".to_owned(),
            support: Support::TrendConditioned,
            provenance: "Archard 1953, forma; K efectivo oiled ilustrativo (P1)".to_owned(),
            coeff: 1e-8,
            coeff_range: (1e-9, 1e-7),
        }
    }

    /// Cojinete: `dc = Kb·(1−film)·p·dt` acoplado a película S22
    /// (Kb ilustrativo). Sin película no hay número HD: tendencia.
    #[must_use]
    pub fn bearing_starvation() -> Self {
        Self {
            name: "bearing-starvation".to_owned(),
            support: Support::TrendConditioned,
            provenance: "acoplado a film_quality S22; Kb ilustrativo (P1)".to_owned(),
            coeff: 5e-18,
            coeff_range: (5e-19, 5e-17),
        }
    }

    /// Envejecimiento de aceite: Arrhenius `dox = A·exp(−Ea/RT)·dt`
    /// (Ea 80 kJ/mol ilustrativa). Tiñe viscosidad vía Walther.
    #[must_use]
    pub fn oil_aging() -> Self {
        Self {
            name: "oil-aging-arrhenius".to_owned(),
            support: Support::TrendConditioned,
            provenance: "Arrhenius, forma; Ea,A ilustrativos (P1)".to_owned(),
            coeff: 80_000.0,
            coeff_range: (60_000.0, 100_000.0),
        }
    }

    /// Fatiga de contacto / scuffing / fretting térmico: SIN ley.
    #[must_use]
    pub fn unsupported(name: &str, why: &str) -> Self {
        Self {
            name: name.to_owned(),
            support: Support::Unsupported,
            provenance: why.to_owned(),
            coeff: 0.0,
            coeff_range: (0.0, 0.0),
        }
    }

    /// Paso de Archard: `dh (m)` desde presión (Pa) y deslizamiento (m).
    ///
    /// # Errors
    ///
    /// `Invalid` si entradas no finitas/negativas o ley no soportada.
    pub fn archard_step(
        &self,
        pressure_pa: f64,
        sliding_m: f64,
        hardness_pa: f64,
    ) -> Result<f64, LawError> {
        if self.support != Support::TrendConditioned {
            return Err(LawError::Invalid {
                reason: "unsupported law has no step".to_owned(),
            });
        }
        for v in [pressure_pa, sliding_m, hardness_pa, self.coeff] {
            if !v.is_finite() {
                return Err(LawError::Invalid {
                    reason: "non-finite".to_owned(),
                });
            }
        }
        if pressure_pa < 0.0 || sliding_m < 0.0 || hardness_pa <= 0.0 || self.coeff < 0.0 {
            return Err(LawError::Invalid {
                reason: "domain".to_owned(),
            });
        }
        Ok(self.coeff * pressure_pa * sliding_m / hardness_pa)
    }

    /// Paso oiled: Archard × (1−film) (acopla S22: sin película todo va a
    /// asperezas; con película ideal el desgaste tiende a cero).
    ///
    /// # Errors
    ///
    /// `Invalid` si film fuera de [0, 1] o lo heredado de `archard_step`.
    pub fn archard_step_oiled(
        &self,
        pressure_pa: f64,
        sliding_m: f64,
        hardness_pa: f64,
        film_0_1: f64,
    ) -> Result<f64, LawError> {
        if !(film_0_1.is_finite() && (0.0..=1.0).contains(&film_0_1)) {
            return Err(LawError::Invalid {
                reason: "film".to_owned(),
            });
        }
        Ok(self.archard_step(pressure_pa, sliding_m, hardness_pa)? * (1.0 - film_0_1))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn archard_orders_and_rejects() {
        let law = WearLaw::archard_ring();
        // Más presión o deslizamiento ⇒ más desgaste (dirección).
        let a = law.archard_step(5e6, 1000.0, 2e9).unwrap();
        let b = law.archard_step(10e6, 1000.0, 2e9).unwrap();
        assert!(b > a && a > 0.0);
        // Oiled: mejor película ⇒ menos desgaste; film 1 ⇒ cero.
        let o_good = law.archard_step_oiled(5e6, 1000.0, 2e9, 0.9).unwrap();
        let o_bad = law.archard_step_oiled(5e6, 1000.0, 2e9, 0.2).unwrap();
        assert!(o_bad > o_good && o_good > 0.0);
        assert_eq!(law.archard_step_oiled(5e6, 1000.0, 2e9, 1.0).unwrap(), 0.0);
        assert!(law.archard_step_oiled(5e6, 1000.0, 2e9, 1.5).is_err());
        assert!(law.archard_step(-1.0, 1.0, 2e9).is_err());
        assert!(law.archard_step(1.0, 1.0, 0.0).is_err());
        // Sin soporte no hay paso (nunca número silencioso).
        let u = WearLaw::unsupported("scuffing", "sin ley constitutiva");
        assert!(u.archard_step(1.0, 1.0, 1.0).is_err());
    }
}
