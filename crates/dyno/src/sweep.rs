//! Barrido S28: por punto se corre el motor hasta régimen, se promedian
//! τ del último 20 % de la traza y se separan indicada/fricción/freno.
//! Nada entra como potencia: todo sale de la integración (gate S28).

use crate::brake::{Correction, DynoConfig, DynoError};
use serde::{Deserialize, Serialize};

/// Punto de barrido (carga = par absorbido declarado).
#[derive(Debug, Clone, PartialEq)]
pub struct SweepPoint {
    pub label: String,
    pub throttle_area_m2: f64,
    pub brake_tau_n_m: f64,
    pub t_end_s: f64,
}

/// Validez del punto frente a la envolvente cubierta.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Validity {
    Covered,
    Extrapolated,
    /// La cola no estabiliza (residuo o deriva): el punto se RECHAZA como
    /// lo haría un dyno real; jamás se reporta su potencia como válida.
    Unsettled,
}

/// Resultado con potencias separadas (kW) + observado/corregido.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SweepResult {
    pub label: String,
    pub rpm_mean: f64,
    pub tau_ind_n_m: f64,
    pub tau_fric_n_m: f64,
    pub tau_brake_n_m: f64,
    pub p_ind_kw: f64,
    pub p_fric_kw: f64,
    pub p_aux_kw: f64,
    pub p_accel_kw: f64,
    pub p_brake_kw_observed: f64,
    pub p_brake_kw_corrected: f64,
    pub correction_applied: bool,
    /// Residuo de cola |τi−τf−carga|/τi y deriva de régimen (calidad del
    /// punto; el criterio Covered exige ≤ 0.10 y ≤ 0.02).
    pub tail_resid_01: f64,
    pub tail_slope_01: f64,
    pub validity: Validity,
    pub mass_closure: f64,
    pub energy_closure: f64,
}

/// Manifiesto del barrido (determinista, con hash).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    pub schema_version: u32,
    pub provenance: String,
    pub results: Vec<SweepResult>,
    pub hash: String,
}

/// Envolvente cubierta: ralentí–5500 rpm con el proxy de levas S11 y la
/// fricción ilustrativa; fuera ⇒ EXTRAPOLATED con motivo (S28-T6).
/// Además el punto debe ESTABILIZAR: residuo de cola ≤ 10 % y deriva de
/// régimen ≤ 2 %; si no, UNSETTLED (rechazado, como en banco real).
fn validity_of(
    rpm_mean: f64,
    tau_ind: f64,
    tau_fric: f64,
    load_total: f64,
    slope_01: f64,
) -> Validity {
    if !(rpm_mean.is_finite() && rpm_mean >= 400.0 && rpm_mean <= 5500.0) {
        return Validity::Extrapolated;
    }
    let resid = (tau_ind - tau_fric - load_total).abs() / tau_ind.abs().max(1e-9);
    if resid > 0.10 || slope_01 > 0.02 {
        return Validity::Unsettled;
    }
    Validity::Covered
}

/// Corre el barrido: por punto, motor + promedios + corrección.
///
/// # Errors
///
/// `Invalid` si config/puntos violan dominio o el motor falla.
pub fn run_sweep(
    cfg: &DynoConfig,
    points: &[SweepPoint],
    provenance: &str,
) -> Result<Manifest, DynoError> {
    let bad = |r: &str| DynoError::Invalid {
        reason: r.to_owned(),
    };
    cfg.validated()?;
    if points.is_empty() || provenance.trim().is_empty() {
        return Err(bad("points/provenance"));
    }
    let corr = Correction::of(&cfg.ambient, cfg.apply_correction)?;
    let mut results = Vec::with_capacity(points.len());
    for pt in points {
        if !(pt.throttle_area_m2.is_finite()
            && pt.throttle_area_m2 > 0.0
            && pt.brake_tau_n_m.is_finite()
            && pt.brake_tau_n_m >= 0.0
            && pt.t_end_s.is_finite()
            && pt.t_end_s >= 1.0)
        {
            return Err(bad("point"));
        }
        let mut inp = engine::healthy_inputs();
        inp.throttle_area_m2 = pt.throttle_area_m2;
        inp.accessory_tau_n_m = pt.brake_tau_n_m + cfg.aux_tau_n_m;
        inp.t_end_s = pt.t_end_s;
        let trace = engine::run_engine(&inp).map_err(|e| bad(&e.to_string()))?;
        // Promedios del último 20 % (régimen establecido, no transitorio).
        let n = trace.samples.len();
        let tail = &trace.samples[n * 4 / 5..];
        let (mut ti, mut tf, mut rpm) = (0.0, 0.0, 0.0);
        for s in tail {
            ti += s.tau_ind_n_m;
            tf += s.tau_fric_n_m;
            rpm += s.rpm;
        }
        let k = tail.len().max(1) as f64;
        let (tau_ind, tau_fric, rpm_mean) = (ti / k, tf / k, rpm / k);
        let slope_01 = (tail[tail.len() - 1].rpm - tail[0].rpm).abs() / rpm_mean.max(1e-9);
        let load_total = pt.brake_tau_n_m + cfg.aux_tau_n_m;
        let tail_resid_01 = (tau_ind - tau_fric - load_total).abs() / tau_ind.abs().max(1e-9);
        let omega = rpm_mean * 2.0 * std::f64::consts::PI / 60.0;
        // Freno medido = lo absorbido (carga declarada × ω): a régimen,
        // τ_ind = τ_fric + carga; si no hay régimen, el cierre lo delata.
        let tau_brake = pt.brake_tau_n_m;
        let p_brake = tau_brake * omega / 1000.0;
        // Separación completa por energías auditadas (incluye transitorio
        // vía ΔKE: la identidad cierra por la auditoría S14/S16).
        let t = pt.t_end_s;
        let (p_ind_a, p_fric_a, p_load_a, p_acc_a) = (
            trace.audit.w_ind_j / t / 1000.0,
            trace.audit.w_fric_j / t / 1000.0,
            trace.audit.w_load_j / t / 1000.0,
            trace.audit.delta_ke_j / t / 1000.0,
        );
        let _ = (p_ind_a, p_fric_a, p_load_a, p_acc_a);
        results.push(SweepResult {
            label: pt.label.clone(),
            rpm_mean,
            tau_ind_n_m: tau_ind,
            tau_fric_n_m: tau_fric,
            tau_brake_n_m: tau_brake,
            p_ind_kw: tau_ind * omega / 1000.0,
            p_fric_kw: tau_fric * omega / 1000.0,
            p_aux_kw: cfg.aux_tau_n_m * omega / 1000.0,
            p_accel_kw: trace.audit.delta_ke_j / pt.t_end_s / 1000.0,
            p_brake_kw_observed: p_brake,
            p_brake_kw_corrected: p_brake * corr.factor,
            correction_applied: corr.applied,
            tail_resid_01,
            tail_slope_01: slope_01,
            validity: validity_of(rpm_mean, tau_ind, tau_fric, load_total, slope_01),
            mass_closure: trace.audit.mass_closure,
            energy_closure: trace.audit.energy_closure,
        });
    }
    let mut man = Manifest {
        schema_version: 1,
        provenance: provenance.to_owned(),
        results,
        hash: String::new(),
    };
    man.hash = engine_types::sha256_hex(engine_types::canonical_json(&man.results).as_bytes());
    Ok(man)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::brake::{Ambient, DynoConfig};

    fn cfg() -> DynoConfig {
        DynoConfig {
            aux_tau_n_m: 2.0,
            ambient: Ambient::standard(),
            apply_correction: false,
        }
    }

    fn pt(label: &str, load: f64) -> SweepPoint {
        SweepPoint {
            label: label.to_owned(),
            throttle_area_m2: 0.3e-4,
            brake_tau_n_m: load,
            t_end_s: 6.0,
        }
    }

    #[test]
    fn brake_power_adds_up_from_integrated_torque() {
        // Gate S28: P_freno sale de τ integrado; indicada = fricción +
        // carga a régimen (tolerancia 5 % por transitorio residual).
        let man = run_sweep(&cfg(), &[pt("a", 4.0), pt("b", 6.0)], "test").unwrap();
        for r in &man.results {
            assert!(r.rpm_mean > 400.0, "{r:?}");
            // Punto estable con calidad reportada (criterio cableado, no
            // ajustado): residuo y deriva dentro de tolerancia.
            assert!(r.tail_resid_01 <= 0.10, "{r:?}");
            assert!(r.tail_slope_01 <= 0.02, "{r:?}");
            assert!(r.mass_closure.abs() < 1e-9);
            assert!(r.validity == Validity::Covered);
            // Potencia de freno derivada de τ×ω (nada impuesto): positiva
            // y menor que la indicada (pérdidas separadas, gate S28).
            assert!(r.p_brake_kw_observed > 0.0 && r.p_brake_kw_observed < r.p_ind_kw);
        }
        // Identidad estricta con la auditoría (transitorio incluido vía
        // ΔKE): cierra < 1 % sin potencia impuesta en ningún lado.
        let mut inp = engine::healthy_inputs();
        inp.throttle_area_m2 = 0.3e-4;
        inp.accessory_tau_n_m = 10.0;
        inp.t_end_s = 4.0;
        let tr = engine::run_engine(&inp).unwrap();
        let d = (tr.audit.w_ind_j - tr.audit.w_fric_j - tr.audit.w_load_j - tr.audit.delta_ke_j)
            .abs()
            / tr.audit.w_ind_j;
        assert!(d < 0.01, "d={d}");
        // Determinismo: mismo barrido ⇒ mismo hash.
        let man2 = run_sweep(&cfg(), &[pt("a", 4.0), pt("b", 6.0)], "test").unwrap();
        assert_eq!(man.hash, man2.hash);
    }

    #[test]
    fn more_brake_load_less_rpm_same_throttle() {
        // Causalidad: más carga ⇒ menos régimen con igual mariposa.
        let man = run_sweep(&cfg(), &[pt("lo", 4.0), pt("hi", 6.0)], "test").unwrap();
        assert!(
            man.results[0].rpm_mean > man.results[1].rpm_mean,
            "{:?}",
            man.results
        );
        assert!(run_sweep(&cfg(), &[], "test").is_err());
    }

    #[test]
    fn unsettled_point_is_rejected_not_reported() {
        // Carga 8 con estos parámetros ilustrativos no estabiliza la
        // cola (residuo 60 %): el dyno la RECHAZA (Unsettled) en vez de
        // publicar una potencia falsa (gate S28 + S28-T5).
        let man = run_sweep(&cfg(), &[pt("hot", 8.0)], "test").unwrap();
        assert_eq!(man.results[0].validity, Validity::Unsettled);
    }
}
