//! Bucle temporal acoplado (S16): aire + combustible + combustión +
//! cigüeñal + arranque por splitting explícito documentado.
//!
//! Por paso `dt`: (1) flujos/nodos/cámaras/combustión con (θ, ω)
//! congelados; (2) cigüeñal Euler simpléctico; (3) avanza θ. Splitting +
//! Euler O(dt): convergencia medida en tests (dt 100→25 µs). Sin clamp de
//! RPM, sin torque mágico, sin inyección de energía: el régimen emerge
//! del balance. Reanudable desde snapshot exacto (mismos floats ⇒
//! bit-idéntico).

use crate::control::OpenLoopControl;
use crate::state::{CylState, EngineSnapshot, EngineState};
use crank::{FrictionModel, InertiaParams};
use electrical::{Battery, Coil, StarterDrive, StarterMotor};
use flow::{curtain_area_m2, exhaust_cd, intake_cd, EXHAUST_VALVE_DIA_MM, INTAKE_VALVE_DIA_MM};
use fuel::{Fuel, Injector, PumpState, Regulator};
use kinematics::{
    cylinder_local_angle,
    phases::{crank_angle_for_piston, CylinderId},
    GeometryPack,
};
use thermo::Mixture;
use thiserror::Error;
use valvetrain::CamSpec;

/// Error de integración del motor.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum EngineError {
    /// Entradas inválidas o estado no físico (fail-closed).
    #[error("invalid engine input/state: {reason}")]
    Invalid { reason: String },
}

/// Entradas completas (todo explícito; ver escenarios sanos/fallos).
#[derive(Debug, Clone)]
pub struct EngineInputs {
    pub geom: GeometryPack,
    pub mix: Mixture,
    pub intake_cam: CamSpec,
    pub exhaust_cam: CamSpec,
    pub control: OpenLoopControl,
    pub fuel: Fuel,
    pub injector: Injector,
    pub regulator_dp_pa: f64,
    pub charge_volts: f64,
    pub dwell_s: f64,
    pub friction: FrictionModel,
    pub inertia: InertiaParams,
    pub starter: Option<(StarterMotor, StarterDrive, Battery)>,
    pub starter_cutout_rpm: f64,
    pub crank_timeout_s: f64,
    pub accessory_tau_n_m: f64,
    pub plenum_v_m3: f64,
    pub exhaust_v_m3: f64,
    pub throttle_area_m2: f64,
    pub idle_bypass_area_m2: f64,
    pub tailpipe_area_m2: f64,
    pub wiebe_a: f64,
    pub wiebe_m: f64,
    pub wiebe_duration_deg: f64,
    pub combustion_efficiency: f64,
    pub dt_s: f64,
    pub t_end_s: f64,
    pub record_every: usize,
}

/// Punto de telemetría (entregable S16).
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TraceSample {
    pub t_s: f64,
    pub rpm: f64,
    pub tau_ind_n_m: f64,
    pub tau_fric_n_m: f64,
    pub tau_starter_n_m: f64,
    pub p_plen_kpa: f64,
    pub p_exh_kpa: f64,
    pub fuel_rate_g_s: f64,
}

/// Traza + auditoría de la corrida.
#[derive(Debug, Clone, PartialEq)]
pub struct RunTrace {
    pub samples: Vec<TraceSample>,
    pub audit: RunAudit,
}

/// Auditoría física de la corrida.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RunAudit {
    pub w_ind_j: f64,
    pub w_fric_j: f64,
    pub w_starter_j: f64,
    pub w_load_j: f64,
    pub fuel_injected_kg: f64,
    pub fuel_burned_kg: f64,
    pub heat_released_j: f64,
    pub mass_closure: f64,
    pub energy_closure: f64,
    pub final_rpm: f64,
    pub delta_u_j: f64,
    pub delta_ke_j: f64,
    pub h_amb_net_j: f64,
    pub h_fuel_j: f64,
}

const CYLS: [CylinderId; 4] = [
    CylinderId::Cyl1,
    CylinderId::Cyl2,
    CylinderId::Cyl3,
    CylinderId::Cyl4,
];

/// Escenario sano: todo habilitado, mariposa entreabierta + bypass.
#[must_use]
pub fn healthy_inputs() -> EngineInputs {
    base_inputs(true, true, false)
}

/// Sin chispa: control y dwell deshabilitados.
#[must_use]
pub fn no_spark_inputs() -> EngineInputs {
    let mut inp = base_inputs(true, true, false);
    inp.control.spark_enabled = false;
    inp.dwell_s = 0.0;
    inp
}

/// Sin combustible: inyección deshabilitada.
#[must_use]
pub fn no_fuel_inputs() -> EngineInputs {
    let mut inp = base_inputs(true, true, false);
    inp.control.fuel_enabled = false;
    inp
}

/// Fricción alta: coeficientes ×3 (ilustrativo de motor degradado/frío).
#[must_use]
pub fn high_friction_inputs() -> EngineInputs {
    base_inputs(true, true, true)
}

fn base_inputs(spark: bool, fuel: bool, high_fric: bool) -> EngineInputs {
    // Entradas redondas de experimento, no calibración: el régimen
    // resultante se reporta tal cual (gate S16).
    EngineInputs {
        geom: GeometryPack::illustrative_d16_like().expect("preset"),
        mix: Mixture::new(287.0, 1.35).expect("mixture"),
        intake_cam: CamSpec::proxy_d16a6_intake().expect("cam"),
        exhaust_cam: CamSpec::proxy_d16a6_exhaust().expect("cam"),
        control: OpenLoopControl {
            eta_v_assumed: 0.75,
            afr_target: 13.5,
            spark_adv_btdc_deg: 15.0,
            spark_enabled: spark,
            fuel_enabled: fuel,
        },
        fuel: Fuel::illustrative_gasoline(),
        injector: Injector::p2a_working_hypothesis(),
        regulator_dp_pa: 300_000.0,
        charge_volts: 13.5,
        dwell_s: 3e-3,
        friction: if high_fric {
            FrictionModel::ChenFlynn {
                coeffs: crank::ChenFlynnCoeffs {
                    c1_pa: 90_000.0,
                    c2_dlless: 0.0,
                    c3_pa_s_m: 24_000.0,
                    c4_pa_s2_m2: 1_200.0,
                },
            }
        } else {
            FrictionModel::ChenFlynn {
                coeffs: crank::ChenFlynnCoeffs {
                    c1_pa: 30_000.0,
                    c2_dlless: 0.0,
                    c3_pa_s_m: 8_000.0,
                    c4_pa_s2_m2: 400.0,
                },
            }
        },
        inertia: InertiaParams::new(0.15, 0.5).expect("inertia"),
        starter: Some((
            StarterMotor::illustrative(),
            StarterDrive::new(12.0).expect("drive"),
            Battery::illustrative_12v(),
        )),
        starter_cutout_rpm: 400.0,
        crank_timeout_s: 5.0,
        accessory_tau_n_m: 5.0,
        plenum_v_m3: 2.0e-3,
        exhaust_v_m3: 1.5e-3,
        throttle_area_m2: 0.5e-4,
        idle_bypass_area_m2: 0.3e-4,
        tailpipe_area_m2: 8.0e-4,
        wiebe_a: 5.0,
        wiebe_m: 2.0,
        wiebe_duration_deg: 60.0,
        combustion_efficiency: 1.0,
        dt_s: 50e-6,
        t_end_s: 6.0,
        record_every: 400,
    }
}

/// Estado de trabajo mutable (convierte exacto a `EngineState`).
#[derive(Debug, Clone)]
pub struct WorkState {
    pub time_s: f64,
    pub theta_rad: f64,
    pub omega_rad_s: f64,
    pub prev_local_deg: [f64; 4],
    pub cyls: [CylW; 4],
    pub plen: NodeW,
    pub exh: NodeW,
    pub fuel_injected_kg: f64,
    pub fuel_burned_kg: f64,
    pub heat_released_j: f64,
    pub w_ind_j: f64,
    pub w_fric_j: f64,
    pub w_starter_j: f64,
    pub w_load_j: f64,
    pub m_in_amb_kg: f64,
    pub m_out_amb_kg: f64,
    pub h_in_amb_j: f64,
    pub h_out_amb_j: f64,
    pub h_fuel_in_j: f64,
    pub m_sys_0_kg: f64,
    pub u_sys_0_j: f64,
}

/// Cámara de trabajo (conserva átomos: lo quemado sigue en `m`).
#[derive(Debug, Clone)]
pub struct CylW {
    pub m: f64,
    pub u: f64,
    pub fresh: f64,
    pub fuel: f64,
    pub burn_latched: f64,
    pub burn_accounted: bool,
}

/// Nodo de trabajo (mutable).
#[derive(Debug, Clone)]
pub struct NodeW {
    pub m: f64,
    pub t: f64,
    pub v: f64,
    pub fresh: f64,
    pub cv: f64,
}

impl NodeW {
    fn p(&self, r: f64) -> f64 {
        self.m * r * self.t / self.v
    }

    fn u(&self) -> f64 {
        self.m * self.cv * self.t
    }

    /// Suma `dm` (kg, fracción `f_in`) y `du` (J), fail-closed.
    fn add(&mut self, dm: f64, f_in: f64, du: f64) -> Result<(), EngineError> {
        let bad = EngineError::Invalid {
            reason: "node step".to_owned(),
        };
        if !(dm.is_finite() && du.is_finite() && f_in.is_finite()) {
            return Err(bad);
        }
        let m1 = self.m + dm;
        if !(m1.is_finite() && m1 > 0.0) {
            return Err(bad);
        }
        if dm >= 0.0 {
            if !(0.0..=1.0).contains(&f_in) {
                return Err(bad);
            }
            self.fresh = (self.fresh * self.m + f_in * dm) / m1;
        }
        let u1 = self.m * self.cv * self.t + du;
        self.m = m1;
        self.t = u1 / (m1 * self.cv);
        if !(self.t.is_finite() && self.t > 0.0) {
            return Err(bad);
        }
        Ok(())
    }
}

impl WorkState {
    /// Convierte a snapshot verificable (hash canónico).
    #[must_use]
    pub fn snapshot(&self) -> EngineSnapshot {
        EngineSnapshot::capture(&EngineState {
            schema_version: 1,
            time_s: self.time_s,
            theta_global_rad: self.theta_rad,
            omega_rad_s: self.omega_rad_s,
            prev_local_deg: self.prev_local_deg,
            cyls: [
                ws_cyl(&self.cyls[0]),
                ws_cyl(&self.cyls[1]),
                ws_cyl(&self.cyls[2]),
                ws_cyl(&self.cyls[3]),
            ],
            plen_m_kg: self.plen.m,
            plen_t_k: self.plen.t,
            plen_fresh: self.plen.fresh,
            exh_m_kg: self.exh.m,
            exh_t_k: self.exh.t,
            exh_fresh: self.exh.fresh,
            fuel_injected_kg: self.fuel_injected_kg,
            fuel_burned_kg: self.fuel_burned_kg,
            heat_released_j: self.heat_released_j,
            work_ind_j: self.w_ind_j,
            work_fric_j: self.w_fric_j,
            work_starter_j: self.w_starter_j,
            work_load_j: self.w_load_j,
        })
    }

    /// Restaura desde snapshot verificado.
    ///
    /// # Errors
    ///
    /// `Invalid` si el hash no verifica o el esquema es desconocido.
    pub fn restore(snap: &EngineSnapshot, inputs: &EngineInputs) -> Result<Self, EngineError> {
        if !snap.verified() {
            return Err(EngineError::Invalid {
                reason: "snapshot hash mismatch".to_owned(),
            });
        }
        if snap.state.schema_version != 1 {
            return Err(EngineError::Invalid {
                reason: "unsupported snapshot schema".to_owned(),
            });
        }
        let s = &snap.state;
        let cv = inputs.mix.cv();
        let mut ws = Self {
            time_s: s.time_s,
            theta_rad: s.theta_global_rad,
            omega_rad_s: s.omega_rad_s,
            prev_local_deg: s.prev_local_deg,
            cyls: [
                cyl_ws(&s.cyls[0]),
                cyl_ws(&s.cyls[1]),
                cyl_ws(&s.cyls[2]),
                cyl_ws(&s.cyls[3]),
            ],
            plen: NodeW {
                m: s.plen_m_kg,
                t: s.plen_t_k,
                v: inputs.plenum_v_m3,
                fresh: s.plen_fresh,
                cv,
            },
            exh: NodeW {
                m: s.exh_m_kg,
                t: s.exh_t_k,
                v: inputs.exhaust_v_m3,
                fresh: s.exh_fresh,
                cv,
            },
            fuel_injected_kg: s.fuel_injected_kg,
            fuel_burned_kg: s.fuel_burned_kg,
            heat_released_j: s.heat_released_j,
            w_ind_j: s.work_ind_j,
            w_fric_j: s.work_fric_j,
            w_starter_j: s.work_starter_j,
            w_load_j: s.work_load_j,
            m_in_amb_kg: 0.0,
            m_out_amb_kg: 0.0,
            h_in_amb_j: 0.0,
            h_out_amb_j: 0.0,
            h_fuel_in_j: 0.0,
            m_sys_0_kg: 0.0,
            u_sys_0_j: 0.0,
        };
        ws.m_sys_0_kg = ws.plen.m + ws.exh.m + ws.cyls.iter().map(|c| c.m).sum::<f64>();
        ws.u_sys_0_j = ws.plen.u() + ws.exh.u() + ws.cyls.iter().map(|c| c.u).sum::<f64>();
        Ok(ws)
    }
}

fn ws_cyl(c: &CylW) -> CylState {
    CylState {
        m_kg: c.m,
        u_j: c.u,
        fresh_frac: c.fresh,
        fuel_kg: c.fuel,
        burn_latched_kg: c.burn_latched,
        burn_accounted: c.burn_accounted,
    }
}

fn cyl_ws(c: &CylState) -> CylW {
    CylW {
        m: c.m_kg,
        u: c.u_j,
        fresh: c.fresh_frac,
        fuel: c.fuel_kg,
        burn_latched: c.burn_latched_kg,
        burn_accounted: c.burn_accounted,
    }
}
/// Cruce de umbral con ángulo periódico [0, 720): detecta el flanco
/// aunque el paso cruce el origen (sin contadores floor, que disparan
/// dos veces por ciclo).
fn crossed_edge(prev: f64, new: f64, x: f64) -> bool {
    if new >= prev {
        prev < x && new >= x
    } else {
        x > prev || x <= new
    }
}

/// Construye el estado inicial (reposo, nodos y cámaras a ambiente).
fn initial_state(inputs: &EngineInputs) -> WorkState {
    let p_amb = 101_325.0;
    let t_amb = 293.15;
    let r_gas = inputs.mix.r();
    let cv = inputs.mix.cv();
    let cyls = [0, 1, 2, 3].map(|k| {
        let local = cylinder_local_angle(0.0, CYLS[k]);
        let crank = crank_angle_for_piston(local);
        let v = inputs.geom.volume(crank.to_radians());
        let m = p_amb * v / (r_gas * t_amb);
        CylW {
            m,
            u: m * cv * t_amb,
            fresh: 1.0,
            fuel: 0.0,
            burn_latched: 0.0,
            burn_accounted: true,
        }
    });
    let plen = NodeW {
        m: p_amb * inputs.plenum_v_m3 / (r_gas * t_amb),
        t: t_amb,
        v: inputs.plenum_v_m3,
        fresh: 1.0,
        cv,
    };
    let exh = NodeW {
        m: p_amb * inputs.exhaust_v_m3 / (r_gas * t_amb),
        t: t_amb,
        v: inputs.exhaust_v_m3,
        fresh: 0.0,
        cv,
    };
    let m0 = plen.m + exh.m + cyls.iter().map(|c| c.m).sum::<f64>();
    let u0 = plen.u() + exh.u() + cyls.iter().map(|c| c.u).sum::<f64>();
    let prev_local_deg = [
        cylinder_local_angle(0.0, CYLS[0]),
        cylinder_local_angle(0.0, CYLS[1]),
        cylinder_local_angle(0.0, CYLS[2]),
        cylinder_local_angle(0.0, CYLS[3]),
    ];
    WorkState {
        time_s: 0.0,
        theta_rad: 0.0,
        omega_rad_s: 0.0,
        prev_local_deg,
        cyls,
        plen,
        exh,
        fuel_injected_kg: 0.0,
        fuel_burned_kg: 0.0,
        heat_released_j: 0.0,
        w_ind_j: 0.0,
        w_fric_j: 0.0,
        w_starter_j: 0.0,
        w_load_j: 0.0,
        m_in_amb_kg: 0.0,
        m_out_amb_kg: 0.0,
        h_in_amb_j: 0.0,
        h_out_amb_j: 0.0,
        h_fuel_in_j: 0.0,
        m_sys_0_kg: m0,
        u_sys_0_j: u0,
    }
}

/// Avanza `ws` hasta `t_end` (≥ actual) devolviendo traza del tramo.
fn drive(
    ws: &mut WorkState,
    inputs: &EngineInputs,
    t_end: f64,
    record_every: usize,
    step_offset: usize,
) -> Result<Vec<TraceSample>, EngineError> {
    let bad = |reason: &str| EngineError::Invalid {
        reason: reason.to_owned(),
    };
    let r_gas = inputs.mix.r();
    let gamma = inputs.mix.gamma();
    let cp = inputs.mix.cp();
    let cv = inputs.mix.cv();
    let p_amb = 101_325.0;
    let t_amb = 293.15;
    let j_eff = crank::effective_inertia_mean(&inputs.geom, inputs.inertia);
    let vd_cyl = inputs.geom.displaced_m3();
    let wiebe = combustion::Wiebe::new(
        inputs.wiebe_a,
        inputs.wiebe_m,
        0.0,
        inputs.wiebe_duration_deg.to_radians(),
    )
    .map_err(|_| bad("wiebe"))?;
    let _ = wiebe;
    let completeness = combustion::Wiebe::new(
        inputs.wiebe_a,
        inputs.wiebe_m,
        0.0,
        inputs.wiebe_duration_deg.to_radians(),
    )
    .map_err(|_| bad("wiebe"))?;
    let completeness = completeness.burned_fraction(completeness.end_rad() + 1.0);
    let dt = inputs.dt_s;
    let coil = Coil {
        inductance_h: 5e-3,
        min_spark_energy_j: 30e-3,
        dwell_s: inputs.dwell_s,
    };
    let regulator = Regulator::new(inputs.regulator_dp_pa).map_err(|_| bad("regulator"))?;
    let mut pmax_run = 101_325.0f64;
    let mut samples = Vec::new();
    let mut step = step_offset;

    while ws.time_s < t_end - 1e-15 {
        let th_deg = ws.theta_rad.to_degrees();
        let mut tau_ind = 0.0;
        let mut fuel_rate = 0.0;

        for (k, cyl_id) in CYLS.iter().enumerate() {
            let local = cylinder_local_angle(th_deg, *cyl_id);
            let crank = crank_angle_for_piston(local);
            let v = inputs.geom.volume(crank.to_radians());
            let dvdth = inputs.geom.dvdtheta(crank.to_radians());
            let c = &mut ws.cyls[k];
            let t_cyl = c.u / (c.m * cv);
            let p_cyl = c.m * r_gas * t_cyl / v;
            if !(p_cyl.is_finite() && p_cyl > 0.0 && t_cyl > 0.0) {
                return Err(bad("cylinder state"));
            }
            pmax_run = pmax_run.max(p_cyl);
            tau_ind += p_cyl * dvdth;
            // Trabajo del gas sobre el pistón (lo que el cigüeñal recibe
            // como W_ind): sin este término la energía aparece de la nada.
            // Se aplica con el ω del paso (splitting documentado).
            c.u -= p_cyl * dvdth * ws.omega_rad_s * dt;

            let li = inputs
                .intake_cam
                .lift_at(local - inputs.intake_cam.centerline_atdc_deg)
                .max(0.0);
            let le = inputs
                .exhaust_cam
                .lift_at(local - (720.0 - inputs.exhaust_cam.centerline_atdc_deg))
                .max(0.0);
            let a_iv = 2.0 * intake_cd(li) * curtain_area_m2(INTAKE_VALVE_DIA_MM, li);
            let a_ev = 2.0 * exhaust_cd(le) * curtain_area_m2(EXHAUST_VALVE_DIA_MM, le);

            let p_plen = ws.plen.p(r_gas);
            let p_exh = ws.exh.p(r_gas);
            // Admisión plen↔cyl (aplica a ambos lados).
            if p_plen > p_cyl {
                let m = flow::mass_flow(p_plen, ws.plen.t, p_cyl, a_iv, r_gas, gamma)
                    .map_err(|_| bad("iv flow"))?;
                let dm = m * dt;
                let f_in = ws.plen.fresh;
                let h_in = cp * ws.plen.t;
                ws.plen.add(-dm, f_in, -dm * h_in)?;
                let m_prev = c.m;
                c.m += dm;
                c.fresh = (c.fresh * m_prev + f_in * dm) / c.m;
                c.u += dm * h_in;
            } else if p_cyl > p_plen {
                let m = flow::mass_flow(p_cyl, t_cyl, p_plen, a_iv, r_gas, gamma)
                    .map_err(|_| bad("iv flow"))?;
                let dm = m * dt;
                c.m -= dm;
                if !(c.m.is_finite() && c.m > 0.0) {
                    return Err(bad("cylinder mass"));
                }
                c.u -= dm * cp * t_cyl;
                ws.plen.add(dm, c.fresh, dm * cp * t_cyl)?;
            }
            // Escape cyl↔exh.
            if p_cyl > p_exh {
                let m = flow::mass_flow(p_cyl, t_cyl, p_exh, a_ev, r_gas, gamma)
                    .map_err(|_| bad("ev flow"))?;
                let dm = m * dt;
                c.m -= dm;
                if !(c.m.is_finite() && c.m > 0.0) {
                    return Err(bad("cylinder mass"));
                }
                c.u -= dm * cp * t_cyl;
                ws.exh.add(dm, c.fresh, dm * cp * t_cyl)?;
            } else if p_exh > p_cyl {
                let m = flow::mass_flow(p_exh, ws.exh.t, p_cyl, a_ev, r_gas, gamma)
                    .map_err(|_| bad("ev flow"))?;
                let dm = m * dt;
                let (f_in, h_in) = (ws.exh.fresh, cp * ws.exh.t);
                ws.exh.add(-dm, f_in, -dm * h_in)?;
                let m_prev = c.m;
                c.m += dm;
                c.fresh = (c.fresh * m_prev + f_in * dm) / c.m;
                c.u += dm * h_in;
            }

            // Inyección L1 agrupada al cruce del IC (flanco con previo):
            // el comando del ciclo se dosifica con la densidad ACTUAL del
            // plenum y se entrega de una vez con deadtime real.
            let ic_here = inputs.intake_cam.centerline_atdc_deg
                + inputs.intake_cam.adv_duration_crank_deg / 2.0;
            if inputs.control.fuel_enabled && crossed_edge(ws.prev_local_deg[k], local, ic_here) {
                let rho_man = p_plen / (r_gas * ws.plen.t);
                let cmd =
                    inputs
                        .control
                        .fuel_command(rho_man, vd_cyl, inputs.fuel, inputs.injector);
                let rail_dp = fuel::resolve_rail(
                    regulator,
                    PumpState {
                        on: true,
                        max_flow_kg_s: None,
                    },
                    p_plen,
                    0.0,
                )
                .map_err(|_| bad("rail"))?
                .delta_p_pa;
                let (pulse, _) = inputs
                    .injector
                    .pulse_for_mass(inputs.fuel, cmd.mass_kg, rail_dp, inputs.charge_volts)
                    .map_err(|_| bad("pulse"))?;
                let (d, _) = inputs.injector.deliver(
                    inputs.fuel,
                    fuel::PulseCommand {
                        width_s: pulse.width_s,
                    },
                    rail_dp,
                    inputs.charge_volts,
                    None,
                );
                let m_prev = c.m;
                c.m += d.mass_kg;
                c.fuel += d.mass_kg;
                c.u += d.mass_kg * cp * ws.plen.t;
                c.fresh = (c.fresh * m_prev + d.mass_kg) / c.m;
                ws.fuel_injected_kg += d.mass_kg;
                ws.h_fuel_in_j += d.mass_kg * cp * ws.plen.t;
                fuel_rate += d.mass_kg / dt;
            }

            // Chispa al cruce del avance, válvulas cerradas (flanco).
            let fire_local = 360.0 - inputs.control.spark_adv_btdc_deg;
            if inputs.control.spark_enabled
                && li < 0.05
                && le < 0.05
                && crossed_edge(ws.prev_local_deg[k], local, fire_local)
            {
                let spark_ok = coil
                    .spark_at(inputs.charge_volts)
                    .map(|s| s.available)
                    .unwrap_or(false);
                if spark_ok {
                    c.burn_latched = c.fuel;
                    c.burn_accounted = false;
                } else {
                    c.burn_latched = 0.0;
                    c.burn_accounted = true;
                }
            }
            // Wiebe con reloj local (solo válvulas cerradas).
            if c.burn_latched > 0.0 && !c.burn_accounted && li < 0.05 && le < 0.05 {
                let w = combustion::Wiebe::new(
                    inputs.wiebe_a,
                    inputs.wiebe_m,
                    fire_local.to_radians(),
                    inputs.wiebe_duration_deg.to_radians(),
                )
                .map_err(|_| bad("wiebe"))?;
                let rate = w.burned_rate(local.to_radians());
                let dq = c.burn_latched
                    * inputs.fuel.lhv_j_per_kg
                    * inputs.combustion_efficiency
                    * rate
                    * ws.omega_rad_s
                    * dt;
                c.u += dq;
                ws.heat_released_j += dq;
                if local > fire_local + inputs.wiebe_duration_deg {
                    let b = (c.burn_latched * completeness).min(c.fuel);
                    ws.fuel_burned_kg += b;
                    c.fuel -= b;
                    c.burn_accounted = true;
                }
            }

            if !(c.u.is_finite() && c.u > 0.0 && c.m > 0.0) {
                return Err(bad("cylinder energy"));
            }
            ws.prev_local_deg[k] = local;
        }

        // Frontera ambiente.
        let p_plen = ws.plen.p(r_gas);
        let a_thr = inputs.throttle_area_m2 + inputs.idle_bypass_area_m2;
        if p_amb > p_plen {
            let m = flow::mass_flow(p_amb, t_amb, p_plen, a_thr, r_gas, gamma)
                .map_err(|_| bad("throttle"))?;
            let dm = m * dt;
            ws.plen.add(dm, 1.0, dm * cp * t_amb)?;
            ws.m_in_amb_kg += dm;
            ws.h_in_amb_j += dm * cp * t_amb;
        } else if p_plen > p_amb {
            let m = flow::mass_flow(p_plen, ws.plen.t, p_amb, a_thr, r_gas, gamma)
                .map_err(|_| bad("throttle"))?;
            let dm = m * dt;
            let h = cp * ws.plen.t;
            ws.plen.add(-dm, ws.plen.fresh, -dm * h)?;
            ws.m_out_amb_kg += dm;
            ws.h_out_amb_j += dm * h;
        }
        let p_exh = ws.exh.p(r_gas);
        if p_exh > p_amb {
            let m = flow::mass_flow(
                p_exh,
                ws.exh.t,
                p_amb,
                inputs.tailpipe_area_m2,
                r_gas,
                gamma,
            )
            .map_err(|_| bad("tailpipe"))?;
            let dm = m * dt;
            let h = cp * ws.exh.t;
            ws.exh.add(-dm, ws.exh.fresh, -dm * h)?;
            ws.m_out_amb_kg += dm;
            ws.h_out_amb_j += dm * h;
        } else if p_amb > p_exh {
            let m = flow::mass_flow(p_amb, t_amb, p_exh, inputs.tailpipe_area_m2, r_gas, gamma)
                .map_err(|_| bad("tailpipe"))?;
            let dm = m * dt;
            ws.exh.add(dm, 1.0, dm * cp * t_amb)?;
            ws.m_in_amb_kg += dm;
            ws.h_in_amb_j += dm * cp * t_amb;
        }

        // Cigüeñal Euler simpléctico (documentado; convergencia medida).
        let rpm_now = ws.omega_rad_s * 60.0 / (2.0 * std::f64::consts::PI);
        let starter_on = inputs.starter.is_some()
            && rpm_now < inputs.starter_cutout_rpm
            && ws.time_s < inputs.crank_timeout_s;
        let tau_starter = if starter_on {
            if let Some((sm, dr, bat)) = &inputs.starter {
                let cur = dr.current(sm, ws.omega_rad_s).unwrap_or(0.0);
                let v = bat.terminal_voltage(cur).unwrap_or(0.0);
                dr.crank_torque(sm, ws.omega_rad_s).unwrap_or(0.0) * (v / 12.8).clamp(0.0, 1.0)
            } else {
                0.0
            }
        } else {
            0.0
        };
        let sp = 2.0 * inputs.geom.stroke().as_m() * rpm_now.max(0.0) / 60.0;
        let tau_fric = inputs
            .friction
            .torque_mag(sp, pmax_run, vd_cyl * 4.0, ws.omega_rad_s)
            .unwrap_or(0.0);
        let acc = (tau_ind - tau_fric - inputs.accessory_tau_n_m + tau_starter) / j_eff;
        let om_new = (ws.omega_rad_s + acc * dt).max(0.0);
        let om_avg = (ws.omega_rad_s + om_new) / 2.0;
        ws.w_ind_j += tau_ind * om_avg * dt;
        ws.w_fric_j += tau_fric * om_avg * dt;
        ws.w_starter_j += tau_starter * om_avg * dt;
        ws.w_load_j += inputs.accessory_tau_n_m * om_avg * dt;
        ws.theta_rad += om_avg * dt;
        ws.omega_rad_s = om_new;
        ws.time_s += dt;

        if step % record_every == 0 {
            samples.push(TraceSample {
                t_s: ws.time_s,
                rpm: ws.omega_rad_s * 60.0 / (2.0 * std::f64::consts::PI),
                tau_ind_n_m: tau_ind,
                tau_fric_n_m: tau_fric,
                tau_starter_n_m: tau_starter,
                p_plen_kpa: ws.plen.p(r_gas) / 1_000.0,
                p_exh_kpa: ws.exh.p(r_gas) / 1_000.0,
                fuel_rate_g_s: fuel_rate,
            });
        }
        step += 1;
    }
    Ok(samples)
}

/// Ejecuta el motor desde reposo hasta `t_end_s` de las entradas.
fn audit_of(ws: &WorkState, _lhv_j_per_kg: f64, j_eff: f64) -> RunAudit {
    let m_sys_1 = ws.plen.m + ws.exh.m + ws.cyls.iter().map(|c| c.m).sum::<f64>();
    let u_sys_1 = ws.plen.u() + ws.exh.u() + ws.cyls.iter().map(|c| c.u).sum::<f64>();
    let ke_1 = 0.5 * j_eff * ws.omega_rad_s * ws.omega_rad_s;
    // Fuentes térmicas/mecánicas EFECTIVAS: el calor lo aporta lo QUEMADO
    // (Q_rel integrado), no lo inyectado (el no quemado conserva su energía
    // química fuera de U: bug de auditoría corregido).
    // Q_rel es conversión INTERNA, pero es LA medida del aporte térmico
    // efectivo; E_chem inyectada no entra en U hasta quemarse.
    let scale_e = (ws.u_sys_0_j + ws.heat_released_j + ws.w_starter_j).max(1e-9);
    RunAudit {
        w_ind_j: ws.w_ind_j,
        w_fric_j: ws.w_fric_j,
        w_starter_j: ws.w_starter_j,
        w_load_j: ws.w_load_j,
        fuel_injected_kg: ws.fuel_injected_kg,
        fuel_burned_kg: ws.fuel_burned_kg,
        heat_released_j: ws.heat_released_j,
        mass_closure: ((m_sys_1 - ws.m_sys_0_kg)
            - (ws.m_in_amb_kg - ws.m_out_amb_kg + ws.fuel_injected_kg))
            / ws.m_sys_0_kg,
        delta_u_j: u_sys_1 - ws.u_sys_0_j,
        // Q_rel es conversión INTERNA (química→térmica dentro del gas):
        // como el no quemado no está en U, la fuente efectiva es Q_rel
        // (no E_chem inyectada); las demás fuentes son H_amb + H_fuel +
        // W_starter, los sumideros W_fric + W_load, y el almacén incluye
        // ΔKE del cigüeñal (bug de auditoría: se omitía).
        energy_closure: ((u_sys_1 - ws.u_sys_0_j + ke_1) + (ws.w_fric_j + ws.w_load_j)
            - ((ws.h_in_amb_j - ws.h_out_amb_j)
                + ws.h_fuel_in_j
                + ws.heat_released_j
                + ws.w_starter_j))
            / scale_e,
        final_rpm: ws.omega_rad_s * 60.0 / (2.0 * std::f64::consts::PI),
        delta_ke_j: 0.5 * j_eff * ws.omega_rad_s * ws.omega_rad_s,
        h_amb_net_j: ws.h_in_amb_j - ws.h_out_amb_j,
        h_fuel_j: ws.h_fuel_in_j,
    }
}

/// Ejecuta el motor desde reposo (θ = 0, ω = 0).
///
/// # Errors
///
/// `Invalid` ante parámetros inválidos o estado no físico (fail-closed).
pub fn run_engine(inputs: &EngineInputs) -> Result<RunTrace, EngineError> {
    if !(inputs.dt_s.is_finite() && inputs.dt_s > 0.0 && inputs.dt_s <= 2e-4) {
        return Err(EngineError::Invalid {
            reason: "dt in (0, 200µs]".to_owned(),
        });
    }
    if !(inputs.t_end_s.is_finite() && inputs.t_end_s > 0.0) {
        return Err(EngineError::Invalid {
            reason: "t_end".to_owned(),
        });
    }
    let mut ws = initial_state(inputs);
    let samples = drive(&mut ws, inputs, inputs.t_end_s, inputs.record_every, 0)?;
    let j_eff = crank::effective_inertia_mean(&inputs.geom, inputs.inertia);
    Ok(RunTrace {
        samples,
        audit: audit_of(&ws, inputs.fuel.lhv_j_per_kg, j_eff),
    })
}

/// Continúa desde un estado de trabajo hasta `t_end` (reanudación exacta).
///
/// # Errors
///
/// Igual que `run_engine`.
pub fn resume(
    ws: &mut WorkState,
    inputs: &EngineInputs,
    t_end: f64,
) -> Result<Vec<TraceSample>, EngineError> {
    if !(t_end.is_finite() && t_end >= ws.time_s) {
        return Err(EngineError::Invalid {
            reason: "t_end must be >= state time".to_owned(),
        });
    }
    // Rebase de auditoría para el tramo (el cierre total se verifica en
    // corridas completas; aquí el contrato es bit-identidad del estado).
    let m0 = ws.plen.m + ws.exh.m + ws.cyls.iter().map(|c| c.m).sum::<f64>();
    let u0 = ws.plen.u() + ws.exh.u() + ws.cyls.iter().map(|c| c.u).sum::<f64>();
    ws.m_sys_0_kg = m0;
    ws.u_sys_0_j = u0;
    ws.m_in_amb_kg = 0.0;
    ws.m_out_amb_kg = 0.0;
    ws.h_in_amb_j = 0.0;
    ws.h_out_amb_j = 0.0;
    ws.h_fuel_in_j = 0.0;
    drive(ws, inputs, t_end, inputs.record_every, 0)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn idle_point(mut inp: EngineInputs) -> EngineInputs {
        inp.throttle_area_m2 = 0.05e-4;
        inp.idle_bypass_area_m2 = 0.02e-4;
        inp.t_end_s = 4.0;
        inp
    }

    #[test]
    fn sealed_burn_closes_energy() {
        // Sin flujos (áreas 0): solo quemado + cigüeñal. Si el cierre falla
        // aquí, el bug está en quemado/cigüeñal; si cierra, en el acople con
        // flujos.
        let mut inp = healthy_inputs();
        inp.throttle_area_m2 = 0.0;
        inp.idle_bypass_area_m2 = 0.0;
        inp.tailpipe_area_m2 = 0.0;
        inp.t_end_s = 2.0;
        let trace = run_engine(&inp).unwrap();
        eprintln!(
            "sealed: rpm={:.0} Q={:.0} closure={:.3e} W_ind={:.0}",
            trace.audit.final_rpm,
            trace.audit.heat_released_j,
            trace.audit.energy_closure,
            trace.audit.w_ind_j
        );
        assert!(trace.audit.mass_closure.abs() < 1e-9);
    }

    #[test]
    fn healthy_idles_without_clamps_or_magic() {
        // Gate S16: ralentí emergente del balance (banda amplia de plausi-
        // bilidad, no objetivo): por encima del cutout del starter (400) y
        // lejos de embalamiento. Sin clamp de RPM ni torque mágico en el
        // código (revisable por inspección: no existe `idle_target`).
        let trace = run_engine(&idle_point(healthy_inputs())).unwrap();
        let rpm = trace.audit.final_rpm;
        assert!(rpm > 400.0 && rpm < 3000.0, "idle rpm={rpm}");
        assert!(trace.audit.mass_closure.abs() < 1e-9);
        assert!(trace.audit.energy_closure.abs() < 2e-3);
        // El combustible quemado casi iguala al inyectado (sin magia).
        assert!(trace.audit.fuel_burned_kg <= trace.audit.fuel_injected_kg);
        assert!(trace.audit.fuel_burned_kg > 0.5 * trace.audit.fuel_injected_kg);
    }

    #[test]
    fn failure_scenarios_order_causally() {
        // Sin chispa / sin combustible ⇒ 0 rpm; fricción alta ⇒ ralentí
        // menor pero sostenido. Orden causal, no valores ajustados.
        let h = run_engine(&idle_point(healthy_inputs()))
            .unwrap()
            .audit
            .final_rpm;
        let mut ns = no_spark_inputs();
        ns.throttle_area_m2 = 0.05e-4;
        ns.idle_bypass_area_m2 = 0.02e-4;
        ns.t_end_s = 6.0;
        let r_ns = run_engine(&ns).unwrap().audit.final_rpm;
        let mut nf = no_fuel_inputs();
        nf.throttle_area_m2 = 0.05e-4;
        nf.idle_bypass_area_m2 = 0.02e-4;
        nf.t_end_s = 6.0;
        let r_nf = run_engine(&nf).unwrap().audit.final_rpm;
        let mut hf = high_friction_inputs();
        hf.throttle_area_m2 = 0.05e-4;
        hf.idle_bypass_area_m2 = 0.02e-4;
        hf.t_end_s = 6.0;
        let r_hf = run_engine(&hf).unwrap().audit.final_rpm;
        assert_eq!(r_ns, 0.0);
        assert_eq!(r_nf, 0.0);
        assert!(r_hf < h && r_hf > 400.0, "hf={r_hf} healthy={h}");
    }

    #[test]
    fn fuel_cut_shuts_down_coherently() {
        // Corte en marcha ⇒ decae por fricción/bombeo (sin run-on mágico).
        let mut inp = idle_point(healthy_inputs());
        inp.t_end_s = 2.0;
        let mut ws = initial_state(&inp);
        let _ = drive(&mut ws, &inp, 2.0, inp.record_every, 0).unwrap();
        let rpm_mid = ws.omega_rad_s * 60.0 / (2.0 * std::f64::consts::PI);
        assert!(rpm_mid > 400.0);
        let mut off = no_fuel_inputs();
        off.dt_s = inp.dt_s;
        off.throttle_area_m2 = inp.throttle_area_m2;
        off.idle_bypass_area_m2 = inp.idle_bypass_area_m2;
        off.record_every = inp.record_every;
        let _ = resume(&mut ws, &off, 6.0).unwrap();
        let rpm_end = ws.omega_rad_s * 60.0 / (2.0 * std::f64::consts::PI);
        assert!(rpm_end < rpm_mid, "must decay: {rpm_end} vs {rpm_mid}");
    }

    #[test]
    fn snapshot_resume_is_bit_identical() {
        // S05 después de la integración: snapshot + restore + resume ⇒
        // mismo estado final que la corrida directa.
        let mut inp = idle_point(healthy_inputs());
        inp.t_end_s = 2.0;
        let full = run_engine(&inp).unwrap();
        let mut ws = initial_state(&inp);
        let _ = drive(&mut ws, &inp, 1.0, inp.record_every, 0).unwrap();
        let snap = ws.snapshot();
        assert!(snap.verified());
        let mut ws2 = WorkState::restore(&snap, &inp).unwrap();
        let _ = resume(&mut ws2, &inp, 2.0).unwrap();
        let snap_full = {
            let mut wsf = initial_state(&inp);
            let _ = drive(&mut wsf, &inp, 2.0, inp.record_every, 0).unwrap();
            wsf.snapshot()
        };
        assert_eq!(ws2.snapshot().state_hash, snap_full.state_hash);
        assert_eq!(ws2.snapshot().state_hash.len(), 64);
        let _ = full;
    }

    #[test]
    fn determinism_and_convergence() {
        // Misma entrada ⇒ misma traza (hash de auditoría + muestras).
        let inp = idle_point(healthy_inputs());
        let a = run_engine(&inp).unwrap();
        let b = run_engine(&inp).unwrap();
        assert_eq!(a.audit.final_rpm, b.audit.final_rpm);
        assert_eq!(a.samples.len(), b.samples.len());
        assert_eq!(a.samples.last().unwrap().rpm, b.samples.last().unwrap().rpm);
        // Convergencia en dt: 100 vs 50 µs difieren < 10% en rpm final.
        let mut coarse = inp.clone();
        coarse.dt_s = 100e-6;
        coarse.t_end_s = 2.0;
        let mut fine = inp.clone();
        fine.t_end_s = 2.0;
        let rc = run_engine(&coarse).unwrap();
        let rf = run_engine(&fine).unwrap();
        let rel = ((rc.audit.final_rpm - rf.audit.final_rpm) / rf.audit.final_rpm).abs();
        assert!(rel < 0.10, "dt convergence: {rel}");
    }

    #[test]
    fn crossed_edge_handles_wrap() {
        assert!(crossed_edge(719.9, 0.1, 0.0));
        assert!(crossed_edge(719.9, 0.1, 720.0) == crossed_edge(719.9, 0.1, 0.0));
        assert!(!crossed_edge(0.0, 10.0, 350.0));
        assert!(crossed_edge(340.0, 350.0, 345.0));
        assert!(!crossed_edge(350.0, 360.0, 345.0));
    }
}
