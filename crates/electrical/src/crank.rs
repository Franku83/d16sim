//! Arranque arrastrado (S15): batería + starter + compresión motored.
//!
//! Sin combustible ni chispa integradas: `started` es siempre falso por
//! diseño (la integración con combustión es S16). El gate S15 es justo
//! eso: ni con combustible y chispa presentes arranca aquí.

use crate::battery::Battery;
use crate::ignition::{Coil, SparkEvent};
use crate::starter::{StarterDrive, StarterMotor};
use crank::{spin_with_assist, DynoLaw, FrictionModel, InertiaParams};
use kinematics::GeometryPack;
use thiserror::Error;

/// Error de arranque.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum CrankError {
    /// Entradas inválidas (tiempos, dwell, inercia, traza).
    #[error("invalid crank input: {reason}")]
    InvalidInput { reason: String },
}

/// Entradas del arranque arrastrado.
#[derive(Debug, Clone)]
pub struct ColdCrankInputs {
    pub battery: Battery,
    pub starter: StarterMotor,
    pub drive: StarterDrive,
    pub coil: Coil,
    /// Masa de combustible presente (kg, informativa: S15 no la quema).
    pub fuel_present_kg: f64,
    /// Inercia y fricción (misma parametrización que S14).
    pub inertia: InertiaParams,
    pub friction: FrictionModel,
    /// Traza motored p(θ) como carga resistiva (S12).
    pub trace_theta_deg: Vec<f64>,
    pub trace_p_pa: Vec<f64>,
    pub t_end_s: f64,
    pub dt_s: f64,
}

/// Resultado del arranque.
#[derive(Debug, Clone)]
pub struct ColdCrankResult {
    /// Siempre falso en S15 (diseño): sin bootstrap.
    pub started: bool,
    pub no_start_reason: &'static str,
    /// RPM media del último ciclo de cigüeñal (0 si no gira).
    pub mean_rpm_last_rev: f64,
    /// Rizo pico-pico del último ciclo (rpm): fluctuación por compresión.
    pub ripple_rpm: f64,
    /// Energía eléctrica suministrada (J) y en cobre (J).
    pub e_supplied_j: f64,
    pub e_copper_j: f64,
    /// Chispa disponible al final (dwell × tensión).
    pub spark: SparkEvent,
    /// Cierre mecánico heredado de `crank`.
    pub mech_closure: f64,
}

/// Gira arrastrado durante `t_end_s` sin arrancar jamás.
///
/// # Errors
///
/// `InvalidInput` si tiempos, traza o inercia no son válidos.
pub fn cold_crank(
    geom: &GeometryPack,
    inputs: &ColdCrankInputs,
) -> Result<ColdCrankResult, CrankError> {
    let bad = |reason: &str| CrankError::InvalidInput {
        reason: reason.to_owned(),
    };
    if !(inputs.t_end_s.is_finite() && inputs.t_end_s > 0.0) {
        return Err(bad("t_end"));
    }
    if !(inputs.fuel_present_kg.is_finite() && inputs.fuel_present_kg >= 0.0) {
        return Err(bad("fuel_present"));
    }
    let ind = crank::IndicatedTorque::from_trace(
        &inputs.trace_theta_deg,
        &inputs.trace_p_pa,
        geom.clone(),
    )
    .map_err(|_| bad("trace"))?;
    let j = crank::effective_inertia_mean(geom, inputs.inertia);
    let battery = inputs.battery;
    let starter = inputs.starter;
    let drive = inputs.drive;
    // Acoplo electromecánico L1: el par nominal escala con V/V0 (motor DC:
    // τ ∝ V a primer orden) y V cae con la carga por R interna. Sin este
    // acoplo, una batería muerta giraría igual (bug detectado por test).
    // V0 = tensión de referencia de los parámetros nominales del starter.
    let assist = |_theta: f64, omega: f64| -> f64 {
        const V0: f64 = 12.8;
        let cur = drive.current(&starter, omega).unwrap_or(0.0);
        let v = battery.terminal_voltage(cur).unwrap_or(0.0);
        let scale = (v / V0).clamp(0.0, 1.0);
        drive.crank_torque(&starter, omega).unwrap_or(0.0) * scale
    };
    let spin = spin_with_assist(
        &ind,
        geom,
        inputs.friction,
        DynoLaw::Free,
        j,
        0.0,
        0.0,
        inputs.t_end_s,
        inputs.dt_s,
        &assist,
    )
    .map_err(|_| bad("spin"))?;

    // Auditoría eléctrica posteada con la serie (V e I por estado).
    let mut e_sup = 0.0;
    let mut e_cu = 0.0;
    let mut prev_t = 0.0;
    let dt_nom = inputs.dt_s;
    for (k, s) in spin.states.iter().enumerate() {
        if k == 0 {
            continue;
        }
        let cur = drive.current(&starter, s.omega_rad_s).unwrap_or(0.0);
        let v = battery.terminal_voltage(cur).unwrap_or(0.0);
        let r = battery.resistance().unwrap_or(0.0);
        e_sup += v * cur * dt_nom;
        e_cu += cur * cur * r * dt_nom;
        prev_t = s.theta_deg;
    }
    let _ = prev_t;

    // Última revolución: media y rizo (fluctuación por compresión).
    let rev: Vec<f64> = {
        let th_end = spin.states.last().map(|s| s.theta_deg).unwrap_or(0.0);
        spin.states
            .iter()
            .filter(|s| s.theta_deg > th_end - 360.0)
            .map(|s| s.omega_rad_s * 60.0 / (2.0 * std::f64::consts::PI))
            .collect()
    };
    let (mean_rpm, ripple) = if rev.len() >= 2 {
        let mean = rev.iter().sum::<f64>() / rev.len() as f64;
        let max = rev.iter().fold(0.0f64, |a, b| a.max(*b));
        let min = rev.iter().fold(f64::INFINITY, |a, b| a.min(*b));
        (mean, max - min)
    } else {
        (0.0, 0.0)
    };

    // Chispa disponible con la tensión final en bornes.
    let cur_end = drive
        .current(
            &starter,
            spin.states.last().map(|s| s.omega_rad_s).unwrap_or(0.0),
        )
        .unwrap_or(0.0);
    let v_end = battery.terminal_voltage(cur_end).unwrap_or(0.0);
    let spark = inputs
        .coil
        .spark_at(v_end)
        .unwrap_or(crate::ignition::SparkEvent {
            available: false,
            energy_j: 0.0,
            dwell_s: 0.0,
        });

    Ok(ColdCrankResult {
        started: false,
        no_start_reason:
            "ignition-fuel integration lands in S16: S15 cranks without starting by design",
        mean_rpm_last_rev: mean_rpm,
        ripple_rpm: ripple,
        e_supplied_j: e_sup,
        e_copper_j: e_cu,
        spark,
        mech_closure: spin.energy_closure,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::battery::Battery;
    use crate::ignition::Coil;
    use crate::starter::{StarterDrive, StarterMotor};
    use crank::{ChenFlynnCoeffs, FrictionModel};

    fn trace() -> (Vec<f64>, Vec<f64>) {
        let mut th = Vec::new();
        let mut pp = Vec::new();
        for i in 0..=72 {
            let t = f64::from(i) * 10.0;
            let d = ((t - 360.0) / 60.0).powi(2);
            th.push(t);
            pp.push(101_325.0 + 1_900_000.0 * (-d).exp());
        }
        (th, pp)
    }

    fn inputs() -> ColdCrankInputs {
        let (th, pp) = trace();
        ColdCrankInputs {
            battery: Battery::illustrative_12v(),
            starter: StarterMotor::illustrative(),
            drive: StarterDrive::new(12.0).unwrap(),
            coil: {
                let mut c = Coil::illustrative();
                c.dwell_s = 3e-3;
                c
            },
            fuel_present_kg: 25e-6,
            inertia: InertiaParams::new(0.15, 0.5).unwrap(),
            friction: FrictionModel::ChenFlynn {
                coeffs: ChenFlynnCoeffs {
                    c1_pa: 30_000.0,
                    c2_dlless: 0.0,
                    c3_pa_s_m: 8_000.0,
                    c4_pa_s2_m2: 400.0,
                },
            },
            trace_theta_deg: th,
            trace_p_pa: pp,
            t_end_s: 2.0,
            dt_s: 1e-4,
        }
    }

    fn geom() -> GeometryPack {
        GeometryPack::illustrative_d16_like().unwrap()
    }

    #[test]
    fn healthy_crank_turns_with_compression_ripple() {
        let r = cold_crank(&geom(), &inputs()).unwrap();
        // Gira a régimen de arranque con fluctuación por compresión.
        assert!(r.mean_rpm_last_rev > 50.0, "rpm={}", r.mean_rpm_last_rev);
        assert!(r.ripple_rpm > 1.0, "ripple={}", r.ripple_rpm);
        assert!(r.mech_closure.abs() < 1e-3);
        assert!(r.spark.available);
        assert!(r.e_supplied_j > r.e_copper_j && r.e_copper_j >= 0.0);
    }

    #[test]
    fn failures_prevent_cranking_but_never_invent_start() {
        // Con combustible Y chispa presentes, S15 no arranca jamás.
        let r = cold_crank(&geom(), &inputs()).unwrap();
        assert!(!r.started);
        // Batería descargada: apenas gira.
        let mut dead = inputs();
        dead.battery.soc = 0.0;
        let rd = cold_crank(&geom(), &dead).unwrap();
        assert!(rd.mean_rpm_last_rev < r.mean_rpm_last_rev);
        assert!(!rd.started);
        // Masa abierta: no gira.
        let mut open = inputs();
        open.battery.r_ground_ohm = 1.0;
        let ro = cold_crank(&geom(), &open).unwrap();
        assert!(ro.mean_rpm_last_rev < 5.0);
        assert!(!ro.started);
        // Starter ausente: no gira (decae a 0 desde reposo).
        let mut gone = inputs();
        gone.starter.present = false;
        let rg = cold_crank(&geom(), &gone).unwrap();
        assert!(rg.mean_rpm_last_rev < 5.0);
        assert!(!rg.started);
        // Chispa nula (dwell 0): el motor sigue sin arrancar y la chispa
        // figura no disponible.
        let mut nospark = inputs();
        nospark.coil.dwell_s = 0.0;
        let rn = cold_crank(&geom(), &nospark).unwrap();
        assert!(!rn.spark.available);
        assert!(!rn.started);
    }
}
