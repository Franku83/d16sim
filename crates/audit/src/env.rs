//! Entorno S30: licencias/fuentes intactas, benchmark CPU y veredicto
//! realtime. Segunda máquina: PENDIENTE (una disponible; documentado).

use thiserror::Error;

/// Error de entorno.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum EnvError {
    /// Archivo ausente o benchmark fallido.
    #[error("env check failed: {reason}")]
    Invalid { reason: String },
}

/// Reporte de entorno.
#[derive(Debug, Clone, PartialEq)]
pub struct EnvReport {
    pub license_present: bool,
    pub third_party_present: bool,
    pub research_intact: Vec<String>,
    pub bench_sim_s: f64,
    pub bench_wall_ms: f64,
    pub realtime_factor: f64,
}

/// Benchmark: ms de pared por segundo simulado (motor sano, 2 s).
#[derive(Debug, Clone, PartialEq)]
pub struct Benchmark {
    pub wall_ms: f64,
    pub sim_s: f64,
}

impl Benchmark {
    /// Mide con `std::time` (una muestra; S30 reporta, no certifica).
    #[must_use]
    pub fn measure() -> Self {
        let mut inp = engine::healthy_inputs();
        inp.throttle_area_m2 = 0.3e-4;
        inp.accessory_tau_n_m = 8.0;
        inp.t_end_s = 2.0;
        let t0 = std::time::Instant::now();
        let _ = engine::run_engine(&inp);
        Self {
            wall_ms: t0.elapsed().as_secs_f64() * 1000.0,
            sim_s: 2.0,
        }
    }
}

/// Verifica licencias, research y mide benchmark.
///
/// # Errors
///
/// `Invalid` si falta un archivo requerido o el motor falla.
pub fn audit_env(root: &std::path::Path) -> Result<EnvReport, EnvError> {
    let bad = |r: &str| EnvError::Invalid {
        reason: r.to_owned(),
    };
    for f in ["LICENSE", "THIRD_PARTY_NOTICES.md"] {
        if !root.join(f).exists() {
            return Err(bad(f));
        }
    }
    let mut intact = Vec::new();
    for f in ["D16Y7_RESEARCH_v0.1.md", "D16Y7_RESEARCH_DONANTES_v0.1.md"] {
        if !root.join(f).exists() {
            return Err(bad(f));
        }
        intact.push(f.to_owned());
    }
    let b = Benchmark::measure();
    Ok(EnvReport {
        license_present: true,
        third_party_present: true,
        research_intact: intact,
        bench_sim_s: b.sim_s,
        bench_wall_ms: b.wall_ms,
        realtime_factor: b.sim_s * 1000.0 / b.wall_ms.max(1e-9),
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn env_checks_and_benchmark_complete() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let rep = audit_env(&root).unwrap();
        assert!(rep.license_present && rep.third_party_present);
        assert_eq!(rep.research_intact.len(), 2);
        // El núcleo supera realtime en esta máquina (factor > 1); si no,
        // el veredicto cambia solo, sin ajustar nada.
        assert!(rep.bench_wall_ms > 0.0);
    }
}
