//! Casos ambiguos (S20): dos causas, una lectura. El veredicto exige la
//! medición complementaria: sin ella es `Inconclusive` por diseño.
//!
//! El harness de tests conoce la verdad SOLO para montar los escenarios
//! (gate S20: "únicamente para evaluación"); los instrumentos nunca la ven.

use thiserror::Error;

/// Error de caso.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum AmbiguityError {
    /// Configuración inválida (tolerancias, datos).
    #[error("invalid ambiguity case: {reason}")]
    InvalidCase { reason: String },
}

/// Veredicto: o inconcluso, o discriminado POR una medición nombrada.
#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    Inconclusive { shared_reading: String },
    Discriminated { by_measurement: String },
}

/// Caso ambiguo ejecutable.
#[derive(Debug, Clone, PartialEq)]
pub enum AmbiguityCase {
    /// MAP con sesgo +0.5 V vs subida real de presión: mismo voltaje.
    MapBiasVsLeak {
        v_biased: f64,
        v_real_rise: f64,
        tolerance_v: f64,
        /// Referencia independiente (segundo manómetro ideal del harness).
        ref_biased_kpa: f64,
        ref_real_kpa: f64,
        ref_tolerance_kpa: f64,
    },
    /// CKP intermitente vs distribuidor desfasado: ambos perturban el
    /// timing; la FORMA de onda (varianza de intervalos) discrimina.
    CkpJitterVsOffset {
        interval_variance_a: f64,
        interval_variance_b: f64,
        separation_ratio: f64,
    },
}

impl AmbiguityCase {
    /// Evalúa: sin medición complementaria ⇒ `Inconclusive`.
    pub fn without_complementary_test(&self) -> Verdict {
        match self {
            AmbiguityCase::MapBiasVsLeak {
                v_biased,
                v_real_rise,
                tolerance_v,
                ..
            } => {
                if (v_biased - v_real_rise).abs() <= *tolerance_v {
                    Verdict::Inconclusive {
                        shared_reading: format!("MAP {v_biased:.2} V ≈ {v_real_rise:.2} V"),
                    }
                } else {
                    Verdict::Discriminated {
                        by_measurement: "direct MAP voltage difference".to_owned(),
                    }
                }
            }
            AmbiguityCase::CkpJitterVsOffset { .. } => Verdict::Inconclusive {
                shared_reading: "disturbed ignition timing in both".to_owned(),
            },
        }
    }

    /// Evalúa CON la medición complementaria (referencia / forma de onda).
    pub fn with_complementary_test(&self) -> Verdict {
        match self {
            AmbiguityCase::MapBiasVsLeak {
                ref_biased_kpa,
                ref_real_kpa,
                ref_tolerance_kpa,
                ..
            } => {
                if (ref_biased_kpa - ref_real_kpa).abs() > *ref_tolerance_kpa {
                    Verdict::Discriminated {
                        by_measurement: "independent manifold pressure reference".to_owned(),
                    }
                } else {
                    Verdict::Inconclusive {
                        shared_reading: "reference agrees".to_owned(),
                    }
                }
            }
            AmbiguityCase::CkpJitterVsOffset {
                interval_variance_a,
                interval_variance_b,
                separation_ratio,
            } => {
                let (hi, lo) = if interval_variance_a > interval_variance_b {
                    (interval_variance_a, interval_variance_b)
                } else {
                    (interval_variance_b, interval_variance_a)
                };
                if *lo > 0.0 && *hi / *lo > *separation_ratio {
                    Verdict::Discriminated {
                        by_measurement: "scope edge-interval variance shape".to_owned(),
                    }
                } else {
                    Verdict::Inconclusive {
                        shared_reading: "similar jitter".to_owned(),
                    }
                }
            }
        }
    }
}

/// Varianza de intervalos entre flancos (métrica de forma para el caso CKP).
#[must_use]
pub fn edge_interval_variance(edges: &[f64]) -> f64 {
    if edges.len() < 3 {
        return 0.0;
    }
    let dts: Vec<f64> = edges.windows(2).map(|w| w[1] - w[0]).collect();
    let mean = dts.iter().sum::<f64>() / dts.len() as f64;
    dts.iter().map(|d| (d - mean).powi(2)).sum::<f64>() / dts.len() as f64
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use trigger::{DistributorTrigger, TriggerConfig, TriggerFault};

    #[test]
    fn map_bias_vs_leak_needs_reference() {
        // Sesgo +0.5 V sobre 0.5 V sanos = 1.0 V; subida real a 60 kPa =
        // 1.0 V: la misma lectura, causas distintas (verdad solo del harness).
        let healthy_v = 0.2 + (35.0 - 20.0) * (4.8 - 0.2) / (250.0 - 20.0);
        let case = AmbiguityCase::MapBiasVsLeak {
            v_biased: healthy_v + 0.5,
            v_real_rise: 0.2 + (60.0 - 20.0) * (4.8 - 0.2) / (250.0 - 20.0),
            tolerance_v: 0.05,
            ref_biased_kpa: 35.0,
            ref_real_kpa: 60.0,
            ref_tolerance_kpa: 5.0,
        };
        assert!(matches!(
            case.without_complementary_test(),
            Verdict::Inconclusive { .. }
        ));
        assert!(matches!(
            case.with_complementary_test(),
            Verdict::Discriminated { .. }
        ));
    }

    #[test]
    fn jitter_vs_offset_separated_by_waveform_shape() {
        // Intermitente real (seed) vs offset 10°: la varianza de intervalos
        // los separa por >10× (métrica sobre flancos S17 medidos).
        let mut cfg_j = TriggerConfig::generic_family();
        cfg_j.fault = TriggerFault::Intermittent {
            channel: 0,
            seed: 41,
            drop_probability: 0.15,
        };
        let tj = DistributorTrigger::new(cfg_j).unwrap();
        let mut cfg_o = TriggerConfig::generic_family();
        cfg_o.fault = TriggerFault::DistributorOffset {
            offset_crank_deg: 10.0,
        };
        let to = DistributorTrigger::new(cfg_o).unwrap();
        let ej = tj.channel_edges(0, 750.0, 0.0, 2.0, 200_000.0, 5).unwrap();
        let eo = to.channel_edges(0, 750.0, 0.0, 2.0, 200_000.0, 5).unwrap();
        let vj = edge_interval_variance(&ej);
        let vo = edge_interval_variance(&eo);
        assert!(vj > vo * 10.0, "vj={vj} vo={vo}");
        let case = AmbiguityCase::CkpJitterVsOffset {
            interval_variance_a: vj,
            interval_variance_b: vo,
            separation_ratio: 10.0,
        };
        assert!(matches!(
            case.without_complementary_test(),
            Verdict::Inconclusive { .. }
        ));
        assert!(matches!(
            case.with_complementary_test(),
            Verdict::Discriminated { .. }
        ));
    }
}
