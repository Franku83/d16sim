//! Cojinetes y película (S22): calidad de película reducida y enlace a
//! fricción. Sin solución hidrodinámica completa (deuda): el número de
//! película ordena, no certifica.
//!
//! `film = clamp(k·μ·N/p_load)`: Stribeck reducido con k ilustrativa.
//! El multiplicador de fricción crece al perder película (ruta causal).

use thiserror::Error;

/// Error de cojinete.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum BearingError {
    /// Geometría o carga inválidas.
    #[error("invalid bearing input: {reason}")]
    Invalid { reason: String },
}

/// Geometría de muñón (entradas, p. ej. holguras OEM S01 por bancada).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BearingGeom {
    /// Diámetro (m).
    pub dia_m: f64,
    /// Ancho (m).
    pub width_m: f64,
    /// Holgura diametral (m).
    pub clearance_m: f64,
}

impl BearingGeom {
    /// # Errors
    ///
    /// `Invalid` si algo no es `> 0` finito.
    pub fn new(dia_m: f64, width_m: f64, clearance_m: f64) -> Result<Self, BearingError> {
        for (v, name) in [
            (dia_m, "dia"),
            (width_m, "width"),
            (clearance_m, "clearance"),
        ] {
            if !(v.is_finite() && v > 0.0) {
                return Err(BearingError::Invalid {
                    reason: name.to_owned(),
                });
            }
        }
        Ok(Self {
            dia_m,
            width_m,
            clearance_m,
        })
    }

    /// Conductancia de fuga lateral L1 (m³): `k_geo·c³` con
    /// `k_geo = π·D/(12·(B/2))` (forma corta documentada, no código HD).
    #[must_use]
    pub fn leak_conductance_m3(&self) -> f64 {
        std::f64::consts::PI * self.dia_m / (12.0 * (self.width_m / 2.0)) * self.clearance_m.powi(3)
    }
}

/// Calidad de película [0, 1] por número reducido (μ·N/p).
/// `k_ref` calibra la escala: ilustrativo 1e5 (orden para que una bancada
/// sana a 3000 rpm/80 °C quede ~0.9); calibración real pendiente (S26).
/// Sin k medido, el número ORDENA cojinetes/condiciones, no certifica.
#[must_use]
pub fn film_quality(mu_pa_s: f64, rpm: f64, load_pa: f64, k_ref: f64) -> f64 {
    if !(mu_pa_s.is_finite() && rpm.is_finite() && load_pa.is_finite() && k_ref.is_finite()) {
        return 0.0;
    }
    if mu_pa_s <= 0.0 || rpm <= 0.0 || load_pa <= 0.0 || k_ref <= 0.0 {
        return 0.0;
    }
    (k_ref * mu_pa_s * rpm / load_pa).clamp(0.0, 1.0)
}

/// Calidad con inanición por suministro: la presión local por debajo de
/// `p_ref` degrada la película aunque μ·N/p sea bueno (L1 declarado del
/// hambre de aceite: sin caudal no hay película que sostener).
#[must_use]
pub fn film_quality_supplied(
    mu_pa_s: f64,
    rpm: f64,
    load_pa: f64,
    k_ref: f64,
    p_local_pa_abs: f64,
    p_ref_pa_abs: f64,
) -> f64 {
    let base = film_quality(mu_pa_s, rpm, load_pa, k_ref);
    if !(p_local_pa_abs.is_finite() && p_ref_pa_abs.is_finite() && p_ref_pa_abs > 0.0) {
        return 0.0;
    }
    base * (p_local_pa_abs / p_ref_pa_abs).clamp(0.0, 1.0)
}

/// Enlace a fricción: multiplicador ≥ 1 que crece al perder película
/// (`1 + k_starve·(1 − film)`, k ilustrativa 2.0). Nunca bandera de daño.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrictionLink {
    /// Penalización por hambre de película (ilustrativa).
    pub k_starve: f64,
}

impl FrictionLink {
    /// Multiplicador de fricción desde la calidad de película.
    #[must_use]
    pub fn multiplier(&self, film_0_1: f64) -> f64 {
        1.0 + self.k_starve * (1.0 - film_0_1.clamp(0.0, 1.0))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn film_orders_and_link_is_causal() {
        // Más viscosidad o rpm ⇒ mejor película; más carga ⇒ peor.
        let base = film_quality(0.01, 3000.0, 5e6, 1e-9);
        assert!(film_quality(0.02, 3000.0, 5e6, 1e-9) > base);
        assert!(film_quality(0.01, 6000.0, 5e6, 1e-9) > base);
        assert!(film_quality(0.01, 3000.0, 10e6, 1e-9) < base);
        assert_eq!(film_quality(0.0, 3000.0, 5e6, 1e-9), 0.0);
        // Enlace: film 1 ⇒ ×1; film 0 ⇒ ×(1+k); monótono.
        let link = FrictionLink { k_starve: 2.0 };
        assert_eq!(link.multiplier(1.0), 1.0);
        assert_eq!(link.multiplier(0.0), 3.0);
        assert!(link.multiplier(0.5) == 2.0);
        assert!(BearingGeom::new(0.05, 0.02, -1e-6).is_err());
    }
}
