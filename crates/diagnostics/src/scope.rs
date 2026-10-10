//! Osciloscopio (S20): timebase, muestreo, trigger, grabación y
//! calibración (ganancia/offset). Sin filtro antialias modelado: el alias
//! es DEMOSTRABLE como limitación del instrumento (test dedicado).
//!
//! La tasa la fija el instrumento, no la GUI: documenta Nyquist por canal.

use thiserror::Error;

/// Error de osciloscopio.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ScopeError {
    /// Parámetros inválidos (tasa, base, ganancia, nivel).
    #[error("invalid scope setup: {reason}")]
    InvalidSetup { reason: String },
}

/// Captura grabada (t, v) con procedencia del instrumento.
#[derive(Debug, Clone, PartialEq)]
pub struct ScopeCapture {
    /// Etiqueta del instrumento + tasa (p. ej. "scope@200kHz").
    pub instrument: String,
    pub sample_rate_hz: f64,
    pub points: Vec<(f64, f64)>,
}

/// Osciloscopio con errores de calibración declarados.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Oscilloscope {
    /// Tasa de muestreo (Hz).
    pub sample_rate_hz: f64,
    /// Ancho de banda declarado (Hz, informativo: sin filtro modelado).
    pub bandwidth_hz: f64,
    /// Error de ganancia (fracción) y offset (V) de calibración.
    pub gain_error: f64,
    pub offset_error_v: f64,
    /// Nivel de trigger flanco subida (V).
    pub trigger_v: f64,
}

impl Oscilloscope {
    /// # Errors
    ///
    /// `InvalidSetup` si tasa/ancho de banda no son válidos.
    pub fn new(
        sample_rate_hz: f64,
        bandwidth_hz: f64,
        gain_error: f64,
        offset_error_v: f64,
        trigger_v: f64,
    ) -> Result<Self, ScopeError> {
        let bad = |reason: &str| ScopeError::InvalidSetup {
            reason: reason.to_owned(),
        };
        if !(sample_rate_hz.is_finite() && sample_rate_hz > 0.0) {
            return Err(bad("sample rate"));
        }
        if !(bandwidth_hz.is_finite() && bandwidth_hz > 0.0) {
            return Err(bad("bandwidth"));
        }
        if !(gain_error.is_finite() && offset_error_v.is_finite() && trigger_v.is_finite()) {
            return Err(bad("calibration/trigger"));
        }
        Ok(Self {
            sample_rate_hz,
            bandwidth_hz,
            gain_error,
            offset_error_v,
            trigger_v,
        })
    }

    /// Adquiere `signal(t)` en `[t0, t0 + duration_s]` con calibración.
    /// Retorna la captura (grabable).
    pub fn acquire(&self, signal: &dyn Fn(f64) -> f64, t0: f64, duration_s: f64) -> ScopeCapture {
        let n = (duration_s * self.sample_rate_hz).ceil() as usize + 1;
        let mut points = Vec::with_capacity(n);
        for i in 0..n {
            let t = t0 + f64::from(i as u32) / self.sample_rate_hz;
            if t > t0 + duration_s {
                break;
            }
            let v = signal(t) * (1.0 + self.gain_error) + self.offset_error_v;
            points.push((t, v));
        }
        ScopeCapture {
            instrument: format!("scope@{}Hz", self.sample_rate_hz),
            sample_rate_hz: self.sample_rate_hz,
            points,
        }
    }

    /// Primer cruce ascendente del nivel de trigger (interpolado).
    #[must_use]
    pub fn trigger_time(&self, cap: &ScopeCapture) -> Option<f64> {
        for w in cap.points.windows(2) {
            let ((t0, v0), (t1, v1)) = (w[0], w[1]);
            if v0 <= self.trigger_v && v1 > self.trigger_v && v1 > v0 {
                let f = (self.trigger_v - v0) / (v1 - v0);
                return Some(t0 + f * (t1 - t0));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn trigger_and_calibration_visible() {
        let s = Oscilloscope::new(10_000.0, 5_000.0, 0.0, 0.0, 1.0).unwrap();
        let cap = s.acquire(
            &|t| 2.0 * (2.0 * std::f64::consts::PI * 50.0 * t).sin(),
            0.0,
            0.05,
        );
        let tt = s.trigger_time(&cap).unwrap();
        // Primer cruce de 1 V en seno 2 V/50 Hz: arcsin(0.5)/2π/50.
        let expect = (0.5f64).asin() / (2.0 * std::f64::consts::PI * 50.0);
        assert!((tt - expect).abs() < 2e-4, "tt={tt}");
        // Ganancia +2%: visible en la muestra.
        let g = Oscilloscope::new(10_000.0, 5_000.0, 0.02, 0.0, 1.0).unwrap();
        let cap2 = g.acquire(&|_| 2.0, 0.0, 0.001);
        assert!((cap2.points[0].1 - 2.04).abs() < 1e-12);
    }

    #[test]
    fn aliasing_is_demonstrable_not_hidden() {
        // Seno 5 kHz muestreado a 6 kHz (< 2×Nyquist para 5 kHz): el
        // instrumento MIENTE mostrando ~1 kHz. Limitación declarada.
        let s = Oscilloscope::new(6_000.0, 3_000.0, 0.0, 0.0, 0.0).unwrap();
        let cap = s.acquire(
            &|t| (2.0 * std::f64::consts::PI * 5_000.0 * t).sin(),
            0.0,
            0.005,
        );
        // Cuenta cruces ascendentes de 0: aparecen ~5 en 5 ms (1 kHz
        // alias) en vez de 25 (5 kHz reales).
        let crossings = cap
            .points
            .windows(2)
            .filter(|w| w[0].1 <= 0.0 && w[1].1 > 0.0)
            .count();
        assert!((crossings as i64 - 5).abs() <= 1, "crossings={crossings}");
    }
}
