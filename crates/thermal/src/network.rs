//! Nodos térmicos (S21): bilancio explícito por paso, Euler explícito.
//!
//! Fuentes: quemado→pared (fracción), fricción→aceite, ambiente como BC.
//! Termostato: factor 0→1 en banda [tstat_lo, tstat_hi] sobre el camino
//! refrigerante→ambiente (L1 de la válvula real).

use thiserror::Error;

/// Error térmico.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ThermalError {
    /// Parámetros o estado inválidos (capacidades, T, potencias).
    #[error("invalid thermal input/state: {reason}")]
    Invalid { reason: String },
}

/// Parámetros ilustrativos de la red (ver valores en `illustrative()`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThermalParams {
    /// Fracción del calor liberado a paredes [0, 1].
    pub wall_frac: f64,
    /// Capacidades (J/K): metal, pistón, aceite, refrigerante.
    pub c_wall: f64,
    pub c_piston: f64,
    pub c_oil: f64,
    pub c_cool: f64,
    /// Conductancias (W/K): pistón→metal, metal→refrigerante,
    /// metal→aceite, metal→ambiente, aceite→ambiente.
    pub ha_piston_wall: f64,
    pub ha_wall_cool: f64,
    pub ha_wall_oil: f64,
    pub ha_wall_amb: f64,
    pub ha_oil_amb: f64,
    /// Radiador a plena apertura (W/K) + banda del termostato (°C).
    pub ha_rad_full: f64,
    pub tstat_lo_c: f64,
    pub tstat_hi_c: f64,
    /// Lag del ECT (s) y enriquecimiento por ECT (on/off).
    pub ect_tau_s: f64,
    pub ect_enrich: bool,
}

impl ThermalParams {
    /// Red ilustrativa (órdenes de magnitud, NO D16Y7 medido):
    /// metal ~60 kg Al, aceite 3.5 L, refrigerante 5 L; hA de W/K de un
    /// SOHC pequeño a ralentí/carga parcial.
    #[must_use]
    pub fn illustrative() -> Self {
        Self {
            wall_frac: 0.30,
            c_wall: 55_000.0,
            c_piston: 4_000.0,
            c_oil: 6_700.0,
            c_cool: 21_000.0,
            ha_piston_wall: 60.0,
            ha_wall_cool: 900.0,
            ha_wall_oil: 120.0,
            ha_wall_amb: 25.0,
            ha_oil_amb: 8.0,
            ha_rad_full: 1_600.0,
            tstat_lo_c: 80.0,
            tstat_hi_c: 88.0,
            ect_tau_s: 5.0,
            ect_enrich: true,
        }
    }

    /// Valida rangos físicos (capacidades/conductancias ≥ 0, fracción y
    /// banda coherentes).
    ///
    /// # Errors
    ///
    /// `Invalid` si algo viola dominio.
    pub fn validated(&self) -> Result<(), ThermalError> {
        let bad = |reason: &str| ThermalError::Invalid {
            reason: reason.to_owned(),
        };
        if !(self.wall_frac.is_finite() && (0.0..=1.0).contains(&self.wall_frac)) {
            return Err(bad("wall_frac"));
        }
        for (v, name) in [
            (self.c_wall, "c_wall"),
            (self.c_piston, "c_piston"),
            (self.c_oil, "c_oil"),
            (self.c_cool, "c_cool"),
            (self.ha_piston_wall, "ha_pw"),
            (self.ha_wall_cool, "ha_wc"),
            (self.ha_wall_oil, "ha_wo"),
            (self.ha_wall_amb, "ha_wa"),
            (self.ha_oil_amb, "ha_oa"),
            (self.ha_rad_full, "ha_rad"),
            (self.ect_tau_s, "ect_tau"),
        ] {
            if !(v.is_finite() && v >= 0.0) {
                return Err(bad(name));
            }
        }
        if !(self.tstat_lo_c.is_finite()
            && self.tstat_hi_c.is_finite()
            && self.tstat_lo_c < self.tstat_hi_c)
        {
            return Err(bad("thermostat band"));
        }
        Ok(())
    }
}

/// Estado térmico (temperaturas en °C, ECT filtrada).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThermalState {
    pub t_wall_c: f64,
    pub t_piston_c: f64,
    pub t_oil_c: f64,
    pub t_cool_c: f64,
    pub ect_filt_c: f64,
}

impl ThermalState {
    /// Todo a ambiente (arranque en frío declarado).
    #[must_use]
    pub fn cold(ambient_c: f64) -> Self {
        Self {
            t_wall_c: ambient_c,
            t_piston_c: ambient_c,
            t_oil_c: ambient_c,
            t_cool_c: ambient_c,
            ect_filt_c: ambient_c,
        }
    }

    /// Energía almacenada (J) sobre 0 °C (auditoría).
    #[must_use]
    pub fn stored_j(&self, p: &ThermalParams) -> f64 {
        self.t_wall_c * p.c_wall
            + self.t_piston_c * p.c_piston
            + self.t_oil_c * p.c_oil
            + self.t_cool_c * p.c_cool
    }

    /// Factor del termostato [0, 1] (0 cerrado bajo la banda).
    #[must_use]
    pub fn thermostat_factor(&self, p: &ThermalParams) -> f64 {
        ((self.t_cool_c - p.tstat_lo_c) / (p.tstat_hi_c - p.tstat_lo_c)).clamp(0.0, 1.0)
    }

    /// Enriquecimiento por ECT (L1 de la cadena S19: frío ⇒ rico).
    #[must_use]
    pub fn enrich_factor(&self, p: &ThermalParams) -> f64 {
        if !p.ect_enrich {
            return 1.0;
        }
        if self.ect_filt_c < 40.0 {
            1.25
        } else if self.ect_filt_c < 70.0 {
            1.05
        } else {
            1.0
        }
    }

    /// Avanza `dt` con calor de quemado a paredes, potencia de fricción al
    /// aceite y ambiente dados. Euler explícito; fail-closed.
    ///
    /// # Errors
    ///
    /// `Invalid` si entradas no finitas/negativas donde no corresponde.
    pub fn step(
        &mut self,
        dt_s: f64,
        q_burn_to_wall_j: f64,
        p_fric_w: f64,
        ambient_c: f64,
        p: &ThermalParams,
    ) -> Result<(), ThermalError> {
        p.validated()?;
        let bad = ThermalError::Invalid {
            reason: "thermal step".to_owned(),
        };
        if !(dt_s.is_finite() && dt_s > 0.0) {
            return Err(bad);
        }
        if !(q_burn_to_wall_j.is_finite()
            && q_burn_to_wall_j >= 0.0
            && p_fric_w.is_finite()
            && p_fric_w >= 0.0
            && ambient_c.is_finite())
        {
            return Err(bad);
        }
        let tstat = self.thermostat_factor(p);
        let d_wall = q_burn_to_wall_j + p.ha_piston_wall * (self.t_piston_c - self.t_wall_c) * dt_s
            - p.ha_wall_cool * (self.t_wall_c - self.t_cool_c) * dt_s
            - p.ha_wall_oil * (self.t_wall_c - self.t_oil_c) * dt_s
            - p.ha_wall_amb * (self.t_wall_c - ambient_c) * dt_s;
        let d_piston = -p.ha_piston_wall * (self.t_piston_c - self.t_wall_c) * dt_s;
        let d_oil = p_fric_w * dt_s + p.ha_wall_oil * (self.t_wall_c - self.t_oil_c) * dt_s
            - p.ha_oil_amb * (self.t_oil_c - ambient_c) * dt_s;
        let d_cool = p.ha_wall_cool * (self.t_wall_c - self.t_cool_c) * dt_s
            - (p.ha_rad_full * tstat) * (self.t_cool_c - ambient_c) * dt_s;
        self.t_wall_c += d_wall / p.c_wall;
        self.t_piston_c += d_piston / p.c_piston;
        self.t_oil_c += d_oil / p.c_oil;
        self.t_cool_c += d_cool / p.c_cool;
        // ECT con lag de primer orden hacia el refrigerante.
        let a = dt_s / (p.ect_tau_s + dt_s);
        self.ect_filt_c += a * (self.t_cool_c - self.ect_filt_c);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn heats_cools_and_thermostat_gates() {
        let p = ThermalParams::illustrative();
        p.validated().unwrap();
        let mut s = ThermalState::cold(20.0);
        // Sin calor ni fricción: todo queda a ambiente (sin fuentes).
        s.step(1.0, 0.0, 0.0, 20.0, &p).unwrap();
        assert!((s.t_cool_c - 20.0).abs() < 1e-12);
        assert_eq!(s.thermostat_factor(&p), 0.0);
        // 10 kW a paredes 60 s calientan el metal y el refrigerante por
        // acoplo (pasos de 1 s: el calor difunde pared→refrigerante).
        for _ in 0..60 {
            s.step(1.0, 10_000.0, 0.0, 20.0, &p).unwrap();
        }
        assert!(s.t_wall_c > 25.0 && s.t_cool_c > 20.0);
        // Fricción calienta el aceite en exclusiva (aislado: hA a cero).
        let mut pi = p;
        pi.ha_wall_oil = 0.0;
        pi.ha_oil_amb = 0.0;
        let mut s2 = ThermalState::cold(20.0);
        s2.step(10.0, 0.0, 1_000.0, 20.0, &pi).unwrap();
        assert!((s2.t_oil_c - (20.0 + 10_000.0 / pi.c_oil)).abs() < 1e-9);
        // Enriquecimiento frío ⇒ 1.25; caliente ⇒ 1.0.
        assert_eq!(s2.enrich_factor(&pi), 1.25);
        let mut hot = s2;
        hot.ect_filt_c = 90.0;
        assert_eq!(hot.enrich_factor(&pi), 1.0);
        // La ECT sigue al refrigerante con lag (tau 5 s: ~63% en 5 s;
        // se fija t_cool para aislar el lag del resto de la red).
        let mut lag = ThermalState::cold(20.0);
        for _ in 0..50 {
            lag.t_cool_c = 90.0;
            lag.step(0.1, 0.0, 0.0, 20.0, &pi).unwrap();
        }
        let expect = 20.0 + (90.0 - 20.0) * 0.632_120_558_828_557_7;
        assert!((lag.ect_filt_c - expect).abs() < 2.0);
        // Termostato: 0 bajo 80, 1 sobre 88, rampa entre ellos.
        let mut tstat = ThermalState::cold(20.0);
        tstat.t_cool_c = 84.0;
        assert!((tstat.thermostat_factor(&pi) - 0.5).abs() < 1e-12);
    }
}
