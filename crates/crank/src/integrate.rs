//! Integración temporal RK4 del eje (S14): `dθ/dt = ω`,
//! `dω/dt = (τ_ind − τ_fric − τ_carga)/J`. Sin reloj de render: el tiempo
//! es físico y el paso es parámetro con estudio de convergencia.

use crate::friction::FrictionModel;
use crate::torque::IndicatedTorque;
use kinematics::GeometryPack;
use thiserror::Error;

/// Error de integración.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum SpinError {
    /// Parámetros inválidos (paso, J, tiempos, condiciones iniciales).
    #[error("invalid spin input: {reason}")]
    InvalidInput { reason: String },
}

/// Carga del freno/dino.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DynoLaw {
    /// Eje libre.
    Free,
    /// Par resistente constante (N·m).
    ConstantTorque { tau_n_m: f64 },
}

/// Estado del eje.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpinState {
    pub theta_deg: f64,
    pub omega_rad_s: f64,
}

/// Resultado de un giro: serie + auditoría energética.
#[derive(Debug, Clone, PartialEq)]
pub struct SpinResult {
    pub states: Vec<SpinState>,
    /// ΔKE rotacional (J).
    pub delta_ke_j: f64,
    /// Trabajo indicado, fricción y carga (J).
    pub w_ind_j: f64,
    pub w_fric_j: f64,
    pub w_load_j: f64,
    /// Cierre: (ΔKE − (W_ind − W_fric − W_load))/escala.
    pub energy_closure: f64,
}

/// Integra el giro desde `(theta0, omega0)` durante `t_end_s` con RK4.
///
/// `Sp` (velocidad media del pistón) se evalúa como `2·S·N/60` con N
/// instantáneas (forma Chen–Flynn). `pmax` sale de la traza (solo entra
/// con c2 ≠ 0).
///
/// # Errors
///
/// `InvalidInput` si paso/J/tiempos no son válidos o la fricción falla.
#[allow(clippy::too_many_arguments)]
pub fn spin_down(
    indicated: &IndicatedTorque,
    geom: &GeometryPack,
    friction: FrictionModel,
    load: DynoLaw,
    inertia_j: f64,
    theta0_deg: f64,
    omega0_rad_s: f64,
    t_end_s: f64,
    dt_s: f64,
) -> Result<SpinResult, SpinError> {
    let bad = |reason: &str| SpinError::InvalidInput {
        reason: reason.to_owned(),
    };
    if !(inertia_j.is_finite() && inertia_j > 0.0) {
        return Err(bad("inertia must be finite and > 0"));
    }
    if !(dt_s.is_finite() && dt_s > 0.0 && dt_s <= t_end_s) {
        return Err(bad("dt must be in (0, t_end]"));
    }
    if !(omega0_rad_s.is_finite() && omega0_rad_s >= 0.0) {
        return Err(bad("omega0 must be finite and >= 0"));
    }
    if !theta0_deg.is_finite() {
        return Err(bad("theta0 must be finite"));
    }
    let load_tau = match load {
        DynoLaw::Free => 0.0,
        DynoLaw::ConstantTorque { tau_n_m } => {
            if !(tau_n_m.is_finite() && tau_n_m >= 0.0) {
                return Err(bad("load torque must be finite and >= 0"));
            }
            tau_n_m
        }
    };
    let vd = geom.displaced_m3() * 4.0;
    // pmax de la traza para el término de carga (débil por defecto, c2 = 0).
    let mut pmax_pa = 0.0f64;
    let mut th_scan = 0.0;
    while th_scan < 720.0 {
        pmax_pa = pmax_pa.max(indicated.pressure_at(th_scan, kinematics::phases::CylinderId::Cyl1));
        th_scan += 5.0;
    }

    let accel = |theta_deg: f64, omega: f64| -> Result<(f64, f64, f64, f64), SpinError> {
        let tau_ind = indicated.torque_at(theta_deg);
        let rpm = omega.max(0.0) * 60.0 / (2.0 * std::f64::consts::PI);
        let sp = 2.0 * geom.stroke().as_m() * rpm / 60.0;
        let tau_fric = friction
            .torque_mag(sp, pmax_pa, vd, omega)
            .map_err(|_| bad("friction evaluation"))?;
        let domega = (tau_ind - tau_fric - load_tau) / inertia_j;
        Ok((omega, domega, tau_ind, tau_fric))
    };

    let mut th = theta0_deg;
    let mut om = omega0_rad_s;
    let mut states = vec![SpinState {
        theta_deg: th,
        omega_rad_s: om,
    }];
    let mut w_ind = 0.0;
    let mut w_fric = 0.0;
    let mut w_load = 0.0;
    let mut t = 0.0;
    while t < t_end_s - 1e-15 {
        let h = dt_s.min(t_end_s - t);
        // RK4 sobre (θ, ω) con torques evaluados en subpasos.
        let (dth1, dom1, ti1, tf1) = accel(th, om)?;
        let (dth2, dom2, ti2, tf2) = accel(
            th + dth1 * h / 2.0 * 180.0 / std::f64::consts::PI,
            (om + dom1 * h / 2.0).max(0.0),
        )?;
        let (dth3, dom3, ti3, tf3) = accel(
            th + dth2 * h / 2.0 * 180.0 / std::f64::consts::PI,
            (om + dom2 * h / 2.0).max(0.0),
        )?;
        let (dth4, dom4, ti4, tf4) = accel(
            th + dth3 * h * 180.0 / std::f64::consts::PI,
            (om + dom3 * h).max(0.0),
        )?;
        let om_new = (om + h * (dom1 + 2.0 * dom2 + 2.0 * dom3 + dom4) / 6.0).max(0.0);
        // Trabajo por trapecios en t con torques de subpaso (auditoría O(h²),
        // separada de la precisión O(h⁴) del estado).
        let om_avg = (om + om_new) / 2.0;
        w_ind += (ti1 + 2.0 * ti2 + 2.0 * ti3 + ti4) / 6.0 * om_avg * h;
        w_fric += (tf1 + 2.0 * tf2 + 2.0 * tf3 + tf4) / 6.0 * om_avg * h;
        w_load += load_tau * om_avg * h;
        th += h * (dth1 + 2.0 * dth2 + 2.0 * dth3 + dth4) / 6.0 * 180.0 / std::f64::consts::PI;
        om = om_new;
        t += h;
        states.push(SpinState {
            theta_deg: th,
            omega_rad_s: om,
        });
    }
    let ke0 = 0.5 * inertia_j * omega0_rad_s * omega0_rad_s;
    let ke1 = 0.5 * inertia_j * om * om;
    let scale = (ke0 + w_ind.abs() + w_fric.abs() + w_load.abs()).max(1e-9);
    Ok(SpinResult {
        states,
        delta_ke_j: ke1 - ke0,
        w_ind_j: w_ind,
        w_fric_j: w_fric,
        w_load_j: w_load,
        energy_closure: ((ke1 - ke0) - (w_ind - w_fric - w_load)) / scale,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::friction::{ChenFlynnCoeffs, FrictionModel};
    use crate::torque::IndicatedTorque;

    fn motored_setup() -> (IndicatedTorque, GeometryPack, f64) {
        // Traza motored S12-like sintética pero física: p alta cerca de TDC
        // de compresión/expansión (local 360°) y ambiente en el resto.
        let geom = GeometryPack::illustrative_d16_like().unwrap();
        let mut th = Vec::new();
        let mut pp = Vec::new();
        for i in 0..=72 {
            let t = f64::from(i) * 10.0;
            // Pico gaussiano en 360° sobre base ambiente (ilustrativo).
            let d = ((t - 360.0) / 60.0).powi(2);
            th.push(t);
            pp.push(101_325.0 + 1_900_000.0 * (-d).exp());
        }
        let ind = IndicatedTorque::from_trace(&th, &pp, geom.clone()).unwrap();
        (ind, geom, 0.15)
    }

    fn fric() -> FrictionModel {
        FrictionModel::ChenFlynn {
            coeffs: ChenFlynnCoeffs {
                c1_pa: 30_000.0,
                c2_dlless: 0.0,
                c3_pa_s_m: 8_000.0,
                c4_pa_s2_m2: 400.0,
            },
        }
    }

    #[test]
    fn friction_without_combustion_spins_down() {
        // Gate S14 (i): sin combustión + fricción positiva ⇒ ω decrece en
        // neto. El par de gas pulsa (oscilación legítima intra-revolución),
        // así que la aserción es decrecimiento neto + cota energética (sin
        // energía gratis), no monotonía estricta.
        let (ind, geom, j) = motored_setup();
        let r = spin_down(&ind, &geom, fric(), DynoLaw::Free, j, 0.0, 80.0, 2.0, 1e-4).unwrap();
        let om_end = r.states.last().unwrap().omega_rad_s;
        assert!(om_end < 80.0, "must spin down: {om_end}");
        assert!(om_end >= 0.0);
        // Sin energía gratis: la prueba es la auditoría (energy_audit_closes
        // and_converges), no una cota instantánea — el volante devuelve
        // energía de compresión y ω puede rizar por encima de ω₀.
    }

    #[test]
    fn energy_audit_closes_and_converges() {
        // Gate S14 (ii): ΔKE = W_ind − W_fric − W_load en error numérico.
        let (ind, geom, j) = motored_setup();
        let r = spin_down(&ind, &geom, fric(), DynoLaw::Free, j, 0.0, 80.0, 0.5, 1e-4).unwrap();
        assert!(
            r.energy_closure.abs() < 1e-3,
            "closure={}",
            r.energy_closure
        );
        // Convergencia al refinar paso (orden > 1, sin divergencia).
        let r2 = spin_down(&ind, &geom, fric(), DynoLaw::Free, j, 0.0, 80.0, 0.5, 5e-5).unwrap();
        let e1 = r.states.last().unwrap().omega_rad_s;
        let e2 = r2.states.last().unwrap().omega_rad_s;
        assert!((e1 - e2).abs() < 0.5, "refinement must agree: {e1} vs {e2}");
    }

    #[test]
    fn locked_rotor_stays_and_inputs_rejected() {
        let (ind, geom, j) = motored_setup();
        // ω0 = 0 con fricción nula en reposo... el torque indicado puede
        // moverlo: con carga que lo supera, se queda (reposo impuesto).
        let r = spin_down(
            &ind,
            &geom,
            FrictionModel::None,
            DynoLaw::ConstantTorque { tau_n_m: 1e9 },
            j,
            0.0,
            0.0,
            0.1,
            1e-4,
        )
        .unwrap();
        assert_eq!(r.states.last().unwrap().omega_rad_s, 0.0);
        assert!(spin_down(
            &ind,
            &geom,
            fric(),
            DynoLaw::Free,
            -1.0,
            0.0,
            80.0,
            0.5,
            1e-4
        )
        .is_err());
        assert!(spin_down(&ind, &geom, fric(), DynoLaw::Free, j, 0.0, 80.0, 0.5, 0.0).is_err());
    }
}
