//! Casos, pares, Monte Carlo y discrepancias S29.

use crate::faults::Fault;
use engine_types::{canonical_json, sha256_hex};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error de suite.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum SuiteError {
    /// Entradas inválidas o el motor falló (fail-closed).
    #[error("invalid suite input: {reason}")]
    Invalid { reason: String },
}

/// Resultado de un caso (física + procedencia, sin veredictos).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaseResult {
    pub faults: Vec<String>,
    pub rpm: f64,
    pub fuel_g: f64,
    pub mass_closure: f64,
    pub energy_closure: f64,
    pub hash: String,
}

/// Corre un caso a `t_end_s` fijos (determinista).
///
/// # Errors
///
/// `Invalid` si no hay al menos el caso sano o el motor falla.
pub fn run_case(faults: &[Fault], t_end_s: f64) -> Result<CaseResult, SuiteError> {
    let bad = |r: &str| SuiteError::Invalid {
        reason: r.to_owned(),
    };
    if !(t_end_s.is_finite() && t_end_s >= 1.0) {
        return Err(bad("t_end"));
    }
    let mut inp = engine::healthy_inputs();
    inp.throttle_area_m2 = 0.3e-4;
    for f in faults {
        f.apply(&mut inp);
    }
    inp.t_end_s = t_end_s;
    let trace = engine::run_engine(&inp).map_err(|e| bad(&e.to_string()))?;
    let mut names = faults.iter().map(Fault::name).collect::<Vec<_>>();
    names.sort();
    let body = serde_json::json!({
        "faults": names,
        "rpm": trace.audit.final_rpm,
        "fuel_g": trace.audit.fuel_injected_kg * 1e3,
    });
    Ok(CaseResult {
        faults: names,
        rpm: trace.audit.final_rpm,
        fuel_g: trace.audit.fuel_injected_kg * 1e3,
        mass_closure: trace.audit.mass_closure,
        energy_closure: trace.audit.energy_closure,
        hash: sha256_hex(canonical_json(&body).as_bytes()),
    })
}

/// Par combinado + interacción (combinado vs suma de simples): la física
/// no suma penalizadores y el informe lo cuantifica, no lo impone.
#[derive(Debug, Clone, PartialEq)]
pub struct PairResult {
    pub pair: (String, String),
    pub rpm_healthy: f64,
    pub rpm_a: f64,
    pub rpm_b: f64,
    pub rpm_ab: f64,
    /// Δ_ab − (Δ_a + Δ_b): ≠ 0 ⇒ no-lineal (emergente, no recetado).
    pub interaction_rpm: f64,
}

/// Corre sano + simples + combinado.
///
/// # Errors
///
/// `Invalid` si algún caso falla.
pub fn run_pair(a: &Fault, b: &Fault, t_end_s: f64) -> Result<PairResult, SuiteError> {
    let h = run_case(&[], t_end_s)?;
    let ra = run_case(std::slice::from_ref(a), t_end_s)?;
    let rb = run_case(std::slice::from_ref(b), t_end_s)?;
    let rab = run_case(&[a.clone(), b.clone()], t_end_s)?;
    Ok(PairResult {
        pair: (a.name(), b.name()),
        rpm_healthy: h.rpm,
        rpm_a: ra.rpm,
        rpm_b: rb.rpm,
        rpm_ab: rab.rpm,
        interaction_rpm: (rab.rpm - h.rpm) - ((ra.rpm - h.rpm) + (rb.rpm - h.rpm)),
    })
}

/// Discrepancia catalogada (top por |Δrpm| vs sano).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Discrepancy {
    pub rank: usize,
    pub faults: Vec<String>,
    pub rpm: f64,
    pub delta_rpm: f64,
}

/// Monte Carlo con seed fija (xorshift determinista, sin RNG externo):
/// jitter parametrizado de mariposa/carga/avance. Misma seed ⇒ mismo
/// catálogo (test); el informe lista el top por |Δrpm|.
pub struct MonteCarlo {
    pub seed: u64,
    pub n: usize,
    pub t_end_s: f64,
}

impl MonteCarlo {
    /// Ejecuta y devuelve el catálogo ordenado + hash.
    ///
    /// # Errors
    ///
    /// `Invalid` si n == 0 o un caso falla.
    pub fn run(&self) -> Result<(Vec<Discrepancy>, String), SuiteError> {
        if self.n == 0 {
            return Err(SuiteError::Invalid {
                reason: "n".to_owned(),
            });
        }
        let mut st = self.seed.max(1);
        let mut next = || {
            st ^= st << 13;
            st ^= st >> 7;
            st ^= st << 17;
            (st as f64) / (u64::MAX as f64)
        };
        let healthy = run_case(&[], self.t_end_s)?;
        let mut out = Vec::with_capacity(self.n);
        for _ in 0..self.n {
            // Distribución parametrizada y documentada (no eventos sin
            // justificar): mariposa ±20 %, carga 0–8 Nm, avance ±4°.
            let thr = 0.3e-4 * (0.8 + 0.4 * next());
            let load = 8.0 * next();
            let adv = 4.0 * (next() - 0.5) * 2.0;
            let mut inp = engine::healthy_inputs();
            inp.throttle_area_m2 = thr;
            inp.accessory_tau_n_m += load;
            inp.control.spark_adv_btdc_deg += adv;
            inp.t_end_s = self.t_end_s;
            let trace = engine::run_engine(&inp).map_err(|e| SuiteError::Invalid {
                reason: e.to_string(),
            })?;
            out.push((
                vec![format!("thr{thr:.2e}"), format!("load{load:.2}")],
                trace.audit.final_rpm,
            ));
            let _ = adv;
        }
        let mut disc: Vec<Discrepancy> = out
            .into_iter()
            .map(|(f, rpm)| Discrepancy {
                rank: 0,
                faults: f,
                rpm,
                delta_rpm: rpm - healthy.rpm,
            })
            .collect();
        disc.sort_by(|a, b| {
            b.delta_rpm
                .abs()
                .partial_cmp(&a.delta_rpm.abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for (i, d) in disc.iter_mut().enumerate() {
            d.rank = i + 1;
        }
        let hash = sha256_hex(canonical_json(&disc).as_bytes());
        Ok((disc, hash))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::faults::Fault;

    const T: f64 = 3.0;

    #[test]
    fn conservation_holds_under_combined_faults() {
        // S29-T2: invariantes bajo dos fallos simultáneos (los 4 pares).
        let pairs = [
            (
                Fault::CoolingRestricted { factor: 0.3 },
                Fault::ExtraLoad { tau_n_m: 6.0 },
            ),
            (
                Fault::RestrictIntake { factor: 0.5 },
                Fault::LeanAfr { afr: 16.5 },
            ),
            (
                Fault::RetardSpark { deg: 10.0 },
                Fault::LowVoltage { volts: 9.0 },
            ),
            (
                Fault::RichAfr { afr: 12.0 },
                Fault::RestrictIntake { factor: 0.6 },
            ),
        ];
        for (a, b) in &pairs {
            let r = run_pair(a, b, T).unwrap();
            for case in [
                run_case(&[], T).unwrap(),
                run_case(std::slice::from_ref(a), T).unwrap(),
                run_case(std::slice::from_ref(b), T).unwrap(),
                run_case(&[a.clone(), b.clone()], T).unwrap(),
            ] {
                assert!(case.mass_closure.abs() < 1e-9, "{case:?}");
                assert!(case.energy_closure.abs() < 2e-3, "{case:?}");
            }
            let _ = r;
        }
    }

    #[test]
    fn masking_pair_is_nonlinear_not_additive() {
        // NoSpark enmascara: combinado = 0 aunque el otro falle solo —
        // la interacción cuantifica lo no-aditivo (emergente).
        // (El starter S15 motora a ~350 rpm sin combustión: el combinado
        // queda en motorado, no en 0. Enmascaramiento igualmente no-aditivo.)
        let r = run_pair(&Fault::NoSpark, &Fault::ExtraLoad { tau_n_m: 6.0 }, T).unwrap();
        assert!(r.rpm_ab < r.rpm_healthy - 1000.0, "{r:?}");
        assert!((r.rpm_ab - r.rpm_a).abs() < 200.0, "{r:?}");
        assert!(r.interaction_rpm.abs() > 100.0, "{r:?}");
        // Orden conmuta (transformaciones puras): A+B ≡ B+A bit a bit.
        let ab = run_case(
            &[
                Fault::RetardSpark { deg: 8.0 },
                Fault::LeanAfr { afr: 16.0 },
            ],
            T,
        )
        .unwrap();
        let ba = run_case(
            &[
                Fault::LeanAfr { afr: 16.0 },
                Fault::RetardSpark { deg: 8.0 },
            ],
            T,
        )
        .unwrap();
        assert_eq!(ab.hash, ba.hash);
    }

    #[test]
    fn identical_partial_reading_needs_complementary_test() {
        // S29-T4 (observabilidad, vía protocolo S24): misma compresión
        // baja, distinto reparto ⇒ distinto veredicto; sin reparto ⇒
        // inconcluso aunque la lectura parcial coincida.
        use diagnostics::{diagnose_low_compression, LeakSplit, MechVerdict};
        let rings = diagnose_low_compression(
            Some(700.0),
            900.0,
            Some(35.0),
            // R01-C2/C1-03: descomposición completa normalizada (Σ = 1).
            Some(LeakSplit {
                to_crankcase_01: 0.8,
                to_exhaust_01: 0.1,
                to_coolant_01: 0.1,
            }),
        );
        let valve = diagnose_low_compression(
            Some(700.0),
            900.0,
            Some(35.0),
            Some(LeakSplit {
                to_crankcase_01: 0.1,
                to_exhaust_01: 0.8,
                to_coolant_01: 0.1,
            }),
        );
        assert_ne!(rings, valve);
        assert!(matches!(
            diagnose_low_compression(Some(700.0), 900.0, None, None),
            MechVerdict::Inconclusive { .. }
        ));
    }

    #[test]
    fn monte_carlo_seeded_is_reproducible() {
        let mc = MonteCarlo {
            seed: 1234,
            n: 16,
            t_end_s: 2.0,
        };
        let (d1, h1) = mc.run().unwrap();
        let (d2, h2) = mc.run().unwrap();
        assert_eq!(h1, h2);
        assert_eq!(d1.len(), 16);
        // Ranking ordenado por |Δ| descendente.
        for w in d1.windows(2) {
            assert!(w[0].delta_rpm.abs() >= w[1].delta_rpm.abs());
        }
        assert_eq!(d2.len(), 16);
    }
}
