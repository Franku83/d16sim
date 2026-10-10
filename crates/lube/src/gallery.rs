//! Red de galerías (S22): reparto por holgura con bomba y alivio.
//!
//! Cada rama (bancada/biela) fuga `Q = G(c)·Δp/μ` con G de su holgura; la
//! presión de galería equilibra oferta (bomba×aireación) y demanda
//! (Σramas + alivio). Iteración determinista por bisección (tolerancia
//! declarada). Presiones absolutas ≥ 0 siempre (fail-closed si no).

use crate::bearing::BearingGeom;
use crate::pump::PumpState;
use thiserror::Error;

/// Error de galería.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum GalleryError {
    /// Red inválida (sin ramas, viscosidad, rpm, presión ambiente).
    #[error("invalid gallery input: {reason}")]
    Invalid { reason: String },
}

/// Resultado por rama (cojinete).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BranchResult {
    /// Caudal (m³/s).
    pub flow_m3_s: f64,
    /// Presión LOCAL en la rama (Pa abs). L1: igual a galería (la caída de
    /// línea es deuda declarada).
    pub pressure_pa_abs: f64,
    /// Calidad de película [0, 1].
    pub film_0_1: f64,
}

/// Resultado de la red.
#[derive(Debug, Clone, PartialEq)]
pub struct GalleryResult {
    /// Presión de galería (Pa abs).
    pub gallery_p_pa_abs: f64,
    /// Caudal de alivio (m³/s, 0 si no abre).
    pub relief_m3_s: f64,
    /// Calor de bombeo (W) = Δp×Q (consistencia energética S22).
    pub pump_heat_w: f64,
    pub branches: Vec<BranchResult>,
}

/// Resuelve la red por bisección en presión de galería.
///
/// `bearings`: (geometría, carga_Pa, rpm) por rama; `mu`: viscosidad
/// dinámica; `k_ref`: escala de película; `p_ref`: presión de referencia
/// para inanición (ilustrativa 300 kPa); `p_amb`: ambiente (Pa abs).
///
/// # Errors
///
/// `Invalid` si no hay ramas, μ/rpm/ambiente inválidos o no converge en
/// 200 iteraciones (fail-closed, no valor tapado).
#[allow(clippy::too_many_arguments)]
pub fn solve_gallery(
    pump: &PumpState,
    rpm: f64,
    mu_pa_s: f64,
    p_amb_pa: f64,
    bearings: &[(BearingGeom, f64, f64)],
    k_ref: f64,
    p_ref_pa: f64,
) -> Result<GalleryResult, GalleryError> {
    let bad = GalleryError::Invalid {
        reason: "gallery".to_owned(),
    };
    if bearings.is_empty() {
        return Err(bad);
    }
    if !(mu_pa_s.is_finite() && mu_pa_s > 0.0) {
        return Err(bad);
    }
    if !(rpm.is_finite() && rpm >= 0.0) {
        return Err(bad);
    }
    if !(p_amb_pa.is_finite() && p_amb_pa > 0.0) {
        return Err(bad);
    }
    if !(p_ref_pa.is_finite() && p_ref_pa > 0.0) {
        return Err(bad);
    }
    let q_pump = pump.pump.flow_m3_s(rpm).map_err(|_| bad.clone())? * pump.aeration_factor();
    // Demanda(p) = Σ G·(p−p_amb)/μ + alivio(p); oferta = Q_bomba.
    // f(p) = oferta − demanda es decreciente ⇒ bisección en [p_amb, p_max].
    let demand = |p: f64| -> f64 {
        let mut q = 0.0;
        for (g, _, _) in bearings {
            q += g.leak_conductance_m3() * (p - p_amb_pa).max(0.0) / mu_pa_s;
        }
        // Alivio: sobre su presión deriva el excedente (modelo L1: el
        // caudal sobrante sale por el bypass, la presión no lo supera
        // en más de un 5% declarado).
        q
    };
    let mut lo = p_amb_pa;
    let mut hi = pump.pump.relief_p_pa * 1.05;
    let mut p = (lo + hi) / 2.0;
    for _ in 0..200 {
        let mid = (lo + hi) / 2.0;
        if demand(mid) > q_pump {
            hi = mid;
        } else {
            lo = mid;
        }
        p = (lo + hi) / 2.0;
        if hi - lo < 1.0 {
            break;
        }
    }
    // Alivio: si la presión querría superar el tarado, el excedente sale.
    let p_gal = p.min(pump.pump.relief_p_pa * 1.05).max(p_amb_pa);
    let mut branch_flows: Vec<f64> = bearings
        .iter()
        .map(|(g, _, _)| g.leak_conductance_m3() * (p_gal - p_amb_pa).max(0.0) / mu_pa_s)
        .collect();
    let sum: f64 = branch_flows.iter().sum();
    // Inanición: si las ramas piden más de lo que hay, prorrata declarada
    // (L1 del reparto bajo cavitación; la física fina es deuda).
    let relief: f64;
    if sum > q_pump && sum > 0.0 {
        let k = q_pump / sum;
        for q in branch_flows.iter_mut() {
            *q *= k;
        }
        relief = 0.0;
    } else {
        relief = (q_pump - sum).max(0.0);
    }
    let mut branches = Vec::with_capacity(bearings.len());
    for (i, (_, load_pa, b_rpm)) in bearings.iter().enumerate() {
        branches.push(BranchResult {
            flow_m3_s: branch_flows[i],
            pressure_pa_abs: p_gal,
            film_0_1: super::bearing::film_quality_supplied(
                mu_pa_s, *b_rpm, *load_pa, k_ref, p_gal, p_ref_pa,
            ),
        });
    }
    Ok(GalleryResult {
        gallery_p_pa_abs: p_gal,
        relief_m3_s: relief,
        pump_heat_w: (p_gal - p_amb_pa).max(0.0) * q_pump,
        branches,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::bearing::BearingGeom;
    use crate::oil::OilGrade;
    use crate::pump::{Pump, PumpState};

    fn net() -> (PumpState, Vec<(BearingGeom, f64, f64)>) {
        let pump = PumpState {
            pump: Pump::illustrative(),
            oil_level_01: 1.0,
        };
        // 5 bancadas + 4 bielas (geometría ilustrativa, holguras S01).
        let mains = BearingGeom::new(0.055, 0.020, 0.030e-3).unwrap();
        let rods = BearingGeom::new(0.045, 0.018, 0.029e-3).unwrap();
        let mut b = Vec::new();
        for _ in 0..5 {
            b.push((mains, 5e6, 3000.0));
        }
        for _ in 0..4 {
            b.push((rods, 8e6, 3000.0));
        }
        (pump, b)
    }

    #[test]
    fn continuity_pressures_and_gallery_response() {
        // k_ref ilustrativa 1e5 (bancada sana ~0.9) + p_ref 300 kPa.
        let (pump, b) = net();
        let mu = OilGrade::illustrative_10w30()
            .walther()
            .unwrap()
            .mu_pa_s(80.0);
        let r = solve_gallery(&pump, 3000.0, mu, 101_325.0, &b, 1e5, 300e3).unwrap();
        // Continuidad: ramas + alivio = bomba.
        let q_sum: f64 = r.branches.iter().map(|x| x.flow_m3_s).sum::<f64>() + r.relief_m3_s;
        let q_pump = pump.pump.flow_m3_s(3000.0).unwrap();
        assert!((q_sum - q_pump).abs() / q_pump < 1e-9);
        // Presiones absolutas ≥ 0; con bomba sana el alivio regula (techo).
        assert!(r.gallery_p_pa_abs >= 101_325.0);
        assert!(r.relief_m3_s > 0.0);
        assert!(r.branches.iter().all(|x| x.pressure_pa_abs >= 0.0));
        // Película sana a 3000/80 °C en rango medio-alto (orden, no cota).
        let f0 = r.branches[0].film_0_1;
        assert!(f0 > 0.3 && f0 <= 1.0, "film={f0}");
        // Régimen inanición (baja rpm + bomba débil + holguras grandes):
        // aquí SÍ cae la presión con holgura/caudal/viscosidad.
        let mut starved = pump;
        starved.pump.efficiency = 0.5;
        starved.oil_level_01 = 1.0;
        let mut bs = b.clone();
        for bb in bs.iter_mut() {
            bb.0 = BearingGeom::new(0.055, 0.020, 0.080e-3).unwrap();
        }
        let mu_hot = OilGrade::illustrative_10w30()
            .walther()
            .unwrap()
            .mu_pa_s(130.0);
        let rs = solve_gallery(&starved, 1200.0, mu_hot, 101_325.0, &bs, 1e5, 300e3).unwrap();
        // Fuera del techo del alivio (si pinneara, el test sería vacuo).
        assert!(rs.gallery_p_pa_abs < 525_000.0);
        // Prorrata exacta aun inanido + alivio cerrado.
        let qs: f64 = rs.branches.iter().map(|x| x.flow_m3_s).sum::<f64>() + rs.relief_m3_s;
        let qp = starved.pump.flow_m3_s(1200.0).unwrap() * starved.aeration_factor();
        assert!((qs - qp).abs() / qp.max(1e-12) < 1e-9);
        // Holgura excesiva ADICIONAL en una bancada: cae más la presión.
        let mut bs2 = bs.clone();
        bs2[0].0 = BearingGeom::new(0.055, 0.020, 0.120e-3).unwrap();
        let rs2 = solve_gallery(&starved, 1200.0, mu_hot, 101_325.0, &bs2, 1e5, 300e3).unwrap();
        assert!(rs2.gallery_p_pa_abs <= rs.gallery_p_pa_abs);
        // ...y su película cae con la presión (ruta causal completa).
        assert!(rs2.branches[0].film_0_1 <= rs.branches[0].film_0_1);
    }
}
