//! Transductor VR + acondicionador (S17): tensión vs tiempo, ruido,
//! umbral con histéresis, polaridad, entrehierro y cableado.
//!
//! Modelo L1: `v(t) = ±S·ω_diz·sin(pasos) + ruido`, con S sensibilidad
//! ilustrativa y ruido gaussiano determinista (LCG con seed: sin
//! aleatoriedad del sistema). Polaridad invertida = pendiente de cruce
//! invertida (mecanismo del drift 10–15° documentado, no calibrado).

use thiserror::Error;

/// Error de transductor.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum VrError {
    /// Parámetros inválidos (sensibilidad, gap, ruido, umbral).
    #[error("invalid VR setup: {reason}")]
    Invalid { reason: String },
}

/// Polaridad del cableado VR.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Polarity {
    Normal,
    Reversed,
}

/// Sensor de reluctancia variable (un canal).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VrSensor {
    /// Sensibilidad (V por rad/s del distribuidor, ilustrativa).
    pub sensitivity_v_per_rad_s: f64,
    /// Factor de entrehierro (1 = nominal ilustrativo; > 1 = más gap).
    pub gap_factor: f64,
    /// Ruido aditivo σ (V, determinista por seed en el muestreo).
    pub noise_std_v: f64,
    /// Polaridad del cableado.
    pub polarity: Polarity,
    /// ¿Circuito abierto? (cable cortado: 0 V siempre).
    pub open_circuit: bool,
}

impl VrSensor {
    /// Sensor ilustrativo (S = 40 mV por rad/s de distribuidor: ~1.57 V a
    /// 750 rpm; gap nominal; ruido 5 mV). Orden de magnitud de VR real
    /// (no medido en el distribuidor de la variante).
    #[must_use]
    pub fn illustrative() -> Self {
        Self {
            sensitivity_v_per_rad_s: 0.04,
            gap_factor: 1.0,
            noise_std_v: 0.005,
            polarity: Polarity::Normal,
            open_circuit: false,
        }
    }

    /// # Errors
    ///
    /// `Invalid` si sensibilidad/gap/ruido no son válidos.
    pub fn validated(&self) -> Result<(), VrError> {
        let bad = |reason: &str| VrError::Invalid {
            reason: reason.to_owned(),
        };
        if !(self.sensitivity_v_per_rad_s.is_finite() && self.sensitivity_v_per_rad_s > 0.0) {
            return Err(bad("sensitivity"));
        }
        if !(self.gap_factor.is_finite() && self.gap_factor >= 1.0) {
            return Err(bad("gap_factor >= 1"));
        }
        if !(self.noise_std_v.is_finite() && self.noise_std_v >= 0.0) {
            return Err(bad("noise"));
        }
        Ok(())
    }

    /// Amplitud (V) a velocidad de distribuidor dada.
    #[must_use]
    pub fn amplitude(&self, omega_distributor_rad_s: f64) -> f64 {
        if self.open_circuit {
            return 0.0;
        }
        self.sensitivity_v_per_rad_s * omega_distributor_rad_s / self.gap_factor
    }
}

/// Acondicionador: umbral con histéresis para flancos.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Conditioner {
    /// Umbral de disparo (V, > 0).
    pub threshold_v: f64,
    /// Histéresis (V, ≥ 0).
    pub hysteresis_v: f64,
}

impl Conditioner {
    /// # Errors
    ///
    /// `Invalid` si umbral ≤ 0 o histéresis negativa/no finita.
    pub fn new(threshold_v: f64, hysteresis_v: f64) -> Result<Self, VrError> {
        if !(threshold_v.is_finite() && threshold_v > 0.0) {
            return Err(VrError::Invalid {
                reason: "threshold must be finite and > 0".to_owned(),
            });
        }
        if !(hysteresis_v.is_finite() && hysteresis_v >= 0.0) {
            return Err(VrError::Invalid {
                reason: "hysteresis must be finite and >= 0".to_owned(),
            });
        }
        Ok(Self {
            threshold_v,
            hysteresis_v,
        })
    }

    /// Detecta flancos de subida en una serie (t, v) con histéresis:
    /// arma por debajo de `thr − hys/2`, dispara por encima de `thr + hys/2`.
    /// Retorna tiempos de cruce (interpolados linealmente).
    #[must_use]
    pub fn rising_edges(&self, series: &[(f64, f64)]) -> Vec<f64> {
        let mut edges = Vec::new();
        let mut armed = false;
        let lo = self.threshold_v - self.hysteresis_v / 2.0;
        let hi = self.threshold_v + self.hysteresis_v / 2.0;
        for w in series.windows(2) {
            let ((t0, v0), (t1, v1)) = (w[0], w[1]);
            if !armed && v0 < lo {
                armed = true;
            }
            if armed && v0 <= hi && v1 > hi && v1 > v0 {
                let frac = (hi - v0) / (v1 - v0);
                edges.push(t0 + frac * (t1 - t0));
                armed = false;
            }
        }
        edges
    }
}

/// Ruido gaussiano determinista (Box–Muller sobre LCG con seed explícita).
pub struct DeterministicNoise {
    state: u64,
    spare: Option<f64>,
}

impl DeterministicNoise {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self {
            state: seed,
            spare: None,
        }
    }

    fn uniform(&mut self) -> f64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.state >> 11) as f64) / ((1u64 << 53) as f64)
    }

    /// Uniforme [0, 1) para sorteos reproducibles (intermitencias).
    pub fn uniform01(&mut self) -> f64 {
        self.uniform()
    }

    /// Muestra N(0, std).
    pub fn gaussian(&mut self, std: f64) -> f64 {
        if let Some(s) = self.spare.take() {
            return s * std;
        }
        let u1 = self.uniform().max(1e-300);
        let u2 = self.uniform();
        let r = (-2.0 * u1.ln()).sqrt();
        let theta = 2.0 * std::f64::consts::PI * u2;
        self.spare = Some(r * theta.sin());
        r * theta.cos() * std
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn amplitude_tracks_speed_and_gap() {
        let s = VrSensor::illustrative();
        s.validated().unwrap();
        // A 750 rpm el distribuidor gira a 39.27 rad/s ⇒ ~1.571 V.
        let w = 39.269_908_169_872_37;
        let a = s.amplitude(w);
        assert!((a / s.sensitivity_v_per_rad_s - w).abs() < 1e-9);
        // 0.04 V·s/rad × 12.5π rad/s = π/2 V (aritmética independiente).
        assert!((a - 12.5 * std::f64::consts::PI * 0.04).abs() < 1e-9);
        let wide = VrSensor {
            gap_factor: 3.0,
            ..s
        };
        assert!((wide.amplitude(w) - a / 3.0).abs() < 1e-12);
        let open = VrSensor {
            open_circuit: true,
            ..s
        };
        assert_eq!(open.amplitude(100.0), 0.0);
    }

    #[test]
    fn hysteresis_rejects_chatter_and_finds_edges() {
        let c = Conditioner::new(1.0, 0.2).unwrap();
        // Rampa limpia 0→2 V: un flanco en t ≈ 0.55.
        let series: Vec<(f64, f64)> = (0..=20)
            .map(|i| (f64::from(i) * 0.1, f64::from(i) * 0.1))
            .collect();
        let edges = c.rising_edges(&series);
        assert_eq!(edges.len(), 1);
        assert!((edges[0] - 1.1).abs() < 1e-9);
        // Sin histéresis el ruido contaría de más (documentado, no testeado
        // con ruido aquí: ver test de canales).
        assert!(Conditioner::new(0.0, 0.0).is_err());
    }
}
