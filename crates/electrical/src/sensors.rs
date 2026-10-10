//! Canal de sensor (S19): transductor → cable → ADC → filtro.
//!
//! La lectura NUNCA es el ground truth: cada etapa aporta transferencia,
//! retardo, ruido, offset, deriva y saturación. Un ECT falso cambia la
//! decisión de la ECU sin tocar la temperatura termodinámica (gate).

use thiserror::Error;

/// Error de sensor.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum SensorError {
    /// Parámetros inválidos (transferencia, referencia, tasa, bits).
    #[error("invalid sensor setup: {reason}")]
    InvalidSetup { reason: String },
}

/// Función de transferencia del transductor (entradas ilustrativas, no
/// curvas OEM calibradas: la tabulación real es deuda S01/S25).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Transfer {
    /// V = a·x + b (MAP, TPS).
    Linear { a_v_per_unit: f64, b_v: f64 },
    /// NTC: R = R25·exp(B·(1/T − 1/T25)) con divisor y pull-up.
    Ntc {
        r25_ohm: f64,
        beta_k: f64,
        pullup_ohm: f64,
    },
    /// Conmutado rico/pobre alrededor de lambda = 1 (O2 narrowband L1).
    NarrowbandO2,
}

/// Ruido determinista por canal (LCG con seed: reproducible).
#[derive(Debug, Clone)]
pub struct ChannelNoise {
    state: u64,
    spare: Option<f64>,
}

impl ChannelNoise {
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

    /// Uniforme [0,1) y gaussiana N(0,std) deterministas.
    pub fn uniform01(&mut self) -> f64 {
        self.uniform()
    }

    /// Muestra gaussiana N(0, std).
    pub fn gaussian(&mut self, std: f64) -> f64 {
        if let Some(s) = self.spare.take() {
            return s * std;
        }
        let u1 = self.uniform().max(1e-300);
        let u2 = self.uniform();
        let r = (-2.0 * u1.ln()).sqrt();
        let th = 2.0 * std::f64::consts::PI * u2;
        self.spare = Some(r * th.sin());
        r * th.cos() * std
    }
}

/// Canal completo con estado de filtrado.
#[derive(Debug, Clone)]
pub struct SensorChannel {
    /// Nombre (p. ej. "ECT").
    pub name: &'static str,
    pub transfer: Transfer,
    /// Tensión de referencia (V).
    pub vref_v: f64,
    /// Constante del filtro de primer orden (s, 0 = sin filtro).
    pub filter_tau_s: f64,
    /// Ruido aditivo σ (V).
    pub noise_std_v: f64,
    /// Offset/sesgo aditivo (V, deriva de calibración/envejecimiento).
    pub offset_v: f64,
    /// Deriva lineal (V/s, 0 = sin deriva).
    pub drift_v_per_s: f64,
    /// Saturación [lo, hi] (V).
    pub sat_lo_v: f64,
    pub sat_hi_v: f64,
    /// Bits del ADC.
    pub adc_bits: u32,
    filtered_v: f64,
    time_s: f64,
    noise: ChannelNoise,
}

impl SensorChannel {
    /// Construye validando referencia, tasa implícita, bits y saturación.
    ///
    /// # Errors
    ///
    /// `InvalidSetup` si vref ≤ 0, bits fuera de [1, 24], saturación
    /// invertida o ruidos/constantes inválidos.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: &'static str,
        transfer: Transfer,
        vref_v: f64,
        filter_tau_s: f64,
        noise_std_v: f64,
        offset_v: f64,
        drift_v_per_s: f64,
        sat_lo_v: f64,
        sat_hi_v: f64,
        adc_bits: u32,
        seed: u64,
    ) -> Result<Self, SensorError> {
        let bad = |reason: &str| SensorError::InvalidSetup {
            reason: reason.to_owned(),
        };
        if !(vref_v.is_finite() && vref_v > 0.0) {
            return Err(bad("vref"));
        }
        if !(filter_tau_s.is_finite() && filter_tau_s >= 0.0) {
            return Err(bad("filter tau"));
        }
        if !(noise_std_v.is_finite() && noise_std_v >= 0.0) {
            return Err(bad("noise"));
        }
        if !(offset_v.is_finite() && drift_v_per_s.is_finite()) {
            return Err(bad("offset/drift"));
        }
        if !(sat_lo_v.is_finite() && sat_hi_v.is_finite() && sat_lo_v < sat_hi_v) {
            return Err(bad("saturation band"));
        }
        if !(1..=24).contains(&adc_bits) {
            return Err(bad("adc bits"));
        }
        match transfer {
            Transfer::Linear { a_v_per_unit, b_v } => {
                if !(a_v_per_unit.is_finite() && b_v.is_finite()) {
                    return Err(bad("linear transfer"));
                }
            }
            Transfer::Ntc {
                r25_ohm,
                beta_k,
                pullup_ohm,
            } => {
                if !((r25_ohm.is_finite() && r25_ohm > 0.0)
                    && (beta_k.is_finite() && beta_k > 0.0)
                    && (pullup_ohm.is_finite() && pullup_ohm > 0.0))
                {
                    return Err(bad("ntc transfer"));
                }
            }
            Transfer::NarrowbandO2 => {}
        }
        Ok(Self {
            name,
            transfer,
            vref_v,
            filter_tau_s,
            noise_std_v,
            offset_v,
            drift_v_per_s,
            sat_lo_v,
            sat_hi_v,
            adc_bits,
            filtered_v: 0.0,
            time_s: 0.0,
            noise: ChannelNoise::new(seed),
        })
    }

    /// Tensión del transductor ante magnitud física `x` (unidad según
    /// transferencia: kPa, °C→K interno, grados, lambda).
    fn transducer_v(&self, x: f64) -> f64 {
        match self.transfer {
            Transfer::Linear { a_v_per_unit, b_v } => a_v_per_unit * x + b_v,
            Transfer::Ntc {
                r25_ohm,
                beta_k,
                pullup_ohm,
            } => {
                // x en °C; NTC + divisor con pull-up a vref.
                let t_k = x + 273.15;
                if !(t_k.is_finite() && t_k > 0.0) {
                    return 0.0;
                }
                let r = r25_ohm * (beta_k * (1.0 / t_k - 1.0 / 298.15)).exp();
                self.vref_v * r / (pullup_ohm + r)
            }
            Transfer::NarrowbandO2 => {
                // x = lambda: rico 0.9 V / pobre 0.1 V (L1, sin curva).
                if x < 1.0 {
                    0.9
                } else {
                    0.1
                }
            }
        }
    }

    /// Avanza `dt` con magnitud física `x` y tensión YA degradada por el
    /// cable (`v_wire` sustituye al transductor cuando hay fallo; `None` =
    /// usar transductor). Retorna la lectura filtrada y cuantizada (V).
    pub fn step(&mut self, dt_s: f64, x: f64, v_wire: Option<f64>) -> f64 {
        if !(dt_s.is_finite() && dt_s > 0.0) {
            return self.filtered_v;
        }
        self.time_s += dt_s;
        let mut v = v_wire.unwrap_or_else(|| self.transducer_v(x));
        v += self.offset_v + self.drift_v_per_s * self.time_s;
        v += self.noise.gaussian(self.noise_std_v);
        v = v.clamp(self.sat_lo_v, self.sat_hi_v);
        // ADC: cuantiza a bits en [0, vref].
        let levels = (1u64 << self.adc_bits) as f64;
        let q = ((v / self.vref_v).clamp(0.0, 1.0) * (levels - 1.0)).round() / (levels - 1.0)
            * self.vref_v;
        // Filtro de primer orden (Euler explícito; tau = 0 ⇒ sin filtro).
        if self.filter_tau_s > 0.0 {
            let a = dt_s / (self.filter_tau_s + dt_s);
            self.filtered_v += a * (q - self.filtered_v);
        } else {
            self.filtered_v = q;
        }
        self.filtered_v
    }

    /// Lectura actual (V).
    #[must_use]
    pub fn reading_v(&self) -> f64 {
        self.filtered_v
    }

    /// Tiempo interno (s).
    #[must_use]
    pub fn time_s(&self) -> f64 {
        self.time_s
    }
}

/// Acción ECU ilustrativa desde lectura ECT (enriquecimiento de arranque
/// en frío por TENSION leída). No toca termodinámica: la T física vive en
/// otro tipo (`thermo`/`engine`) que este crate ni nombra.
#[must_use]
pub fn warmup_enrich_from_ect(ect_v: f64) -> f64 {
    if ect_v > 3.0 {
        1.25
    } else if ect_v > 1.5 {
        1.05
    } else {
        1.0
    }
}

/// Carga estimada desde lectura MAP (0–1). No modifica la presión física.
#[must_use]
pub fn load_from_map(map_v: f64) -> f64 {
    ((map_v - 0.2) / (4.8 - 0.2)).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn map_chan() -> SensorChannel {
        // MAP ilustrativo 20–250 kPa → 0.2–4.8 V (NO calibrado OEM).
        SensorChannel::new(
            "MAP",
            Transfer::Linear {
                a_v_per_unit: (4.8 - 0.2) / (250.0 - 20.0),
                b_v: 0.2 - 20.0 * (4.8 - 0.2) / (250.0 - 20.0),
            },
            5.0,
            0.005,
            0.0,
            0.0,
            0.0,
            0.0,
            5.0,
            10,
            1,
        )
        .unwrap()
    }

    #[test]
    fn step_response_reaches_63_percent_at_tau() {
        // Escalón 0→5 V (vía transferencia lineal directa): a t = τ debe
        // estar al 63.2% (filtro de primer orden honesto).
        let mut ch = SensorChannel::new(
            "STEP",
            Transfer::Linear {
                a_v_per_unit: 1.0,
                b_v: 0.0,
            },
            5.0,
            0.1,
            0.0,
            0.0,
            0.0,
            -1.0,
            6.0,
            16,
            9,
        )
        .unwrap();
        let dt = 0.001;
        let mut v = 0.0;
        for _ in 0..100 {
            v = ch.step(dt, 5.0, None);
        }
        // Tras 0.1 s = 1τ: 1 − e^−1 = 63.2%.
        assert!((v - 5.0 * 0.632_120_558_828_557_7).abs() < 0.02, "v={v}");
    }

    #[test]
    fn deterministic_noise_with_seed() {
        let mut a = map_chan();
        a.noise_std_v = 0.01;
        let mut b = map_chan();
        b.noise_std_v = 0.01;
        // Misma seed (1) ⇒ misma serie; distinta seed ⇒ distinta.
        let sa: Vec<f64> = (0..50).map(|_| a.step(0.01, 100.0, None)).collect();
        let sb: Vec<f64> = (0..50).map(|_| b.step(0.01, 100.0, None)).collect();
        assert_eq!(sa, sb);
    }

    #[test]
    fn ntc_and_o2_transfer_shapes() {
        // NTC: a más T, menos R, menos V (divisor con pull-up).
        let mut ect = SensorChannel::new(
            "ECT",
            Transfer::Ntc {
                r25_ohm: 2000.0,
                beta_k: 3435.0,
                pullup_ohm: 2000.0,
            },
            5.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            5.0,
            10,
            3,
        )
        .unwrap();
        let cold = ect.step(0.01, 20.0, None);
        let hot = ect.step(0.01, 90.0, None);
        assert!(cold > hot && cold <= 5.0 && hot >= 0.0);
        // A 25 °C con pull-up igual: ~2.5 V.
        let mut e25 = ect.clone();
        let v25 = e25.step(0.01, 25.0, None);
        assert!((v25 - 2.5).abs() < 0.05, "v25={v25}");
        // O2: conmuta en lambda = 1.
        let mut o2 = SensorChannel::new(
            "O2",
            Transfer::NarrowbandO2,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            10,
            4,
        )
        .unwrap();
        assert!(o2.step(0.01, 0.9, None) > 0.5);
        assert!(o2.step(0.01, 1.1, None) < 0.5);
    }
}

#[cfg(test)]
mod gate_tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::wiring::{Wire, WireFault};

    /// Gate S19: un ECT falso cambia la ACCIÓN de la ECU sin que exista
    /// ningún camino hacia la temperatura termodinámica (este crate ni
    /// nombra tipos termodinámicos: la separación es por construcción y la
    /// acción difiere ante el mismo físico).
    #[test]
    fn false_ect_changes_ecu_action_only() {
        let mk = || {
            SensorChannel::new(
                "ECT",
                Transfer::Ntc {
                    r25_ohm: 2000.0,
                    beta_k: 3435.0,
                    pullup_ohm: 2000.0,
                },
                5.0,
                0.05,
                0.0,
                0.0,
                0.0,
                0.0,
                5.0,
                10,
                21,
            )
            .unwrap()
        };
        // Mismo físico (90 °C), dos transductores: sano vs sesgado −130 °C.
        let physical = 90.0;
        let mut ok_ch = mk();
        let mut bad_ch = mk();
        for _ in 0..200 {
            ok_ch.step(0.01, physical, None);
            bad_ch.step(0.01, physical - 130.0, None);
        }
        let a_ok = warmup_enrich_from_ect(ok_ch.reading_v());
        let a_bad = warmup_enrich_from_ect(bad_ch.reading_v());
        assert_eq!(a_ok, 1.0);
        assert_eq!(a_bad, 1.25);
        assert!(a_bad != a_ok);
    }

    /// Gate S19: un MAP falso no cambia la presión física (el físico es una
    /// constante del test que el canal no puede mutar) pero sí la carga
    /// estimada: 0 V ⇒ 0.0, 5 V ⇒ 1.0 frente a 0.065 sano.
    #[test]
    fn false_map_changes_load_estimate_only() {
        let physical_kpa = 35.0;
        let true_v = 0.2 + (physical_kpa - 20.0) * (4.8 - 0.2) / (250.0 - 20.0);
        let mut open = Wire::new(100.0, 1e6, 5.0, WireFault::Open).unwrap();
        let mut short = Wire::new(100.0, 1e6, 5.0, WireFault::ShortToSupply).unwrap();
        assert_eq!(load_from_map(open.voltage_at_adc(true_v)), 0.0);
        assert_eq!(load_from_map(short.voltage_at_adc(true_v)), 1.0);
        assert!((load_from_map(true_v) - 0.065).abs() < 0.01);
    }
}
