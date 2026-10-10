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
use crate::state::{CylState, EngineSnapshot, EngineState, MigrationProvenance, ModelIdentity};
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
    /// Red térmica L1 (`None` = comportamiento S16 exacto: quemado íntegro
    /// al gas, fricción a sumidero no modelado).
    pub thermal: Option<thermal::ThermalParams>,
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
    /// Combustible almacenado al final en cilindros + nodos (kg, R01-F01).
    pub fuel_stored_kg: f64,
    /// Combustible cruzado a la frontera ambiente (kg, R01-F01).
    pub fuel_emitted_kg: f64,
    /// Calor neto cedido al ambiente por la red térmica (J, R01-F10).
    pub th_q_amb_j: f64,
    /// Máximo de presión de cilindro persistido usado por fricción (Pa, F07).
    pub pmax_pa: f64,
    /// Resumen térmico final (None = red desactivada, modo S16).
    pub thermal: Option<ThermalSummary>,
}

/// Resumen térmico final (telemetría S21).
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ThermalSummary {
    pub ect_c: f64,
    pub t_wall_c: f64,
    pub t_oil_c: f64,
    pub t_cool_c: f64,
    pub thermostat_0_1: f64,
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
        thermal: None,
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
    /// Red térmica + parámetros (None = modo S16).
    pub therm: Option<(thermal::ThermalState, thermal::ThermalParams)>,
    /// Energía térmica inicial (J, sobre 0 °C) para la auditoría.
    pub th_u0_j: f64,
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
    /// Máximo de presión de cilindro (Pa) desde el arranque; alimenta el
    /// término de presión de Chen–Flynn y persiste en el snapshot (R01-F07).
    pub pmax_pa: f64,
    /// Calor neto acumulado cedido al ambiente por la red térmica (J, R01-F10).
    pub th_q_amb_j: f64,
    /// Combustible acumulado cruzado a la frontera ambiente (kg, R01-F01).
    pub fuel_emitted_kg: f64,
    /// Procedencia de migración legacy (R02-C1/RV04); `None` si nativo.
    pub provenance: Option<MigrationProvenance>,
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
    /// Combustible (kg) homogéneo en el nodo (R01-F01).
    pub fuel: f64,
}

impl NodeW {
    fn p(&self, r: f64) -> f64 {
        self.m * r * self.t / self.v
    }

    fn u(&self) -> f64 {
        self.m * self.cv * self.t
    }

    /// Fracción másica de combustible del nodo.
    #[allow(dead_code)]
    fn fuel_frac(&self) -> f64 {
        if self.m > 0.0 {
            self.fuel / self.m
        } else {
            0.0
        }
    }

    /// Entrada de masa `dm` (kg ≥ 0) con fracción fresca `f_in` [0,1], masa
    /// de combustible `fuel_in` (kg) y entalpía `du` (J). Mezcla homogénea.
    fn add_in(&mut self, dm: f64, f_in: f64, fuel_in: f64, du: f64) -> Result<(), EngineError> {
        let bad = EngineError::Invalid {
            reason: "node inflow".to_owned(),
        };
        if !(dm.is_finite() && du.is_finite() && f_in.is_finite() && fuel_in.is_finite()) {
            return Err(bad);
        }
        if dm < 0.0 || !(0.0..=1.0).contains(&f_in) || fuel_in < 0.0 {
            return Err(bad);
        }
        let m1 = self.m + dm;
        if !(m1.is_finite() && m1 > 0.0) {
            return Err(bad);
        }
        if dm > 0.0 {
            self.fresh = (self.fresh * self.m + f_in * dm) / m1;
        }
        self.fuel += fuel_in;
        let u1 = self.m * self.cv * self.t + du;
        self.m = m1;
        self.t = u1 / (m1 * self.cv);
        if !(self.t.is_finite() && self.t > 0.0) {
            return Err(bad);
        }
        if !(self.fuel.is_finite() && self.fuel >= 0.0 && self.fuel <= m1 * (1.0 + 1e-12)) {
            return Err(bad);
        }
        Ok(())
    }

    /// Salida de masa `dm` (kg ≥ 0) homogénea con entalpía específica
    /// `h_spec` (J/kg); devuelve la masa de combustible que sale (kg).
    fn remove(&mut self, dm: f64, h_spec: f64) -> Result<f64, EngineError> {
        let bad = EngineError::Invalid {
            reason: "node outflow".to_owned(),
        };
        if !(dm.is_finite() && dm >= 0.0 && h_spec.is_finite()) {
            return Err(bad);
        }
        if !(self.m - dm > 0.0) {
            return Err(bad);
        }
        let df = self.fuel * dm / self.m;
        let u1 = self.m * self.cv * self.t - dm * h_spec;
        self.m -= dm;
        self.fuel -= df;
        self.t = u1 / (self.m * self.cv);
        if !(self.t.is_finite() && self.t > 0.0) {
            return Err(bad);
        }
        if !(self.fuel.is_finite() && self.fuel >= 0.0) {
            return Err(bad);
        }
        Ok(df)
    }
}

impl WorkState {
    /// Convierte a snapshot verificable (hash canónico, esquema 2).
    #[must_use]
    pub fn snapshot(&self) -> EngineSnapshot {
        EngineSnapshot::capture_with(
            &EngineState {
                schema_version: 2,
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
                thermal: self
                    .therm
                    .as_ref()
                    .map(|(st, _)| crate::state::ThermalSnap {
                        t_wall_c: st.t_wall_c,
                        t_piston_c: st.t_piston_c,
                        t_oil_c: st.t_oil_c,
                        t_cool_c: st.t_cool_c,
                        ect_filt_c: st.ect_filt_c,
                    }),
                plen_fuel_kg: self.plen.fuel,
                exh_fuel_kg: self.exh.fuel,
                fuel_emitted_kg: self.fuel_emitted_kg,
                pmax_pa: Some(self.pmax_pa),
            },
            self.provenance.clone(),
        )
    }

    /// Restaura desde snapshot verificado (acepta esquema 1 migrando la
    /// red térmica a frío, y esquema 2 nativo).
    ///
    /// # Errors
    ///
    /// `Invalid` si el hash no verifica, el esquema es desconocido o la
    /// presencia térmica difiere de las entradas.
    pub fn restore(snap: &EngineSnapshot, inputs: &EngineInputs) -> Result<Self, EngineError> {
        if !snap.verified() {
            return Err(EngineError::Invalid {
                reason: "snapshot hash mismatch".to_owned(),
            });
        }
        if snap.state.schema_version != 1 && snap.state.schema_version != 2 {
            return Err(EngineError::Invalid {
                reason: "unsupported snapshot schema".to_owned(),
            });
        }
        // Contrato de frontera: los inputs también se validan al restaurar
        // (R02-C1/RV05: volúmenes/capacidades/record_every etc.).
        validate_inputs(inputs)?;
        let s = &snap.state;
        // v1 histórico no tenía campo térmico: su presencia es inconsistente.
        if s.schema_version == 1 && s.thermal.is_some() {
            return Err(EngineError::Invalid {
                reason: "v1 snapshot must not carry thermal state".to_owned(),
            });
        }
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
                fuel: s.plen_fuel_kg,
            },
            exh: NodeW {
                m: s.exh_m_kg,
                t: s.exh_t_k,
                v: inputs.exhaust_v_m3,
                fresh: s.exh_fresh,
                cv,
                fuel: s.exh_fuel_kg,
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
            therm: None,
            th_u0_j: 0.0,
            pmax_pa: 0.0,
            th_q_amb_j: 0.0,
            fuel_emitted_kg: s.fuel_emitted_kg,
            provenance: None,
        };
        // Red térmica: nativa en v2; en v1 migra a frío si las entradas la
        // piden, o ausente si no (error si difiere de las entradas).
        let want_therm = inputs.thermal.is_some();
        let got_therm = snap.state.thermal.is_some();
        if want_therm != got_therm && snap.state.schema_version == 2 {
            return Err(EngineError::Invalid {
                reason: "thermal presence mismatch vs inputs".to_owned(),
            });
        }
        if let (Some(tp), Some(ts)) = (inputs.thermal, snap.state.thermal) {
            ws.therm = Some((
                thermal::ThermalState {
                    t_wall_c: ts.t_wall_c,
                    t_piston_c: ts.t_piston_c,
                    t_oil_c: ts.t_oil_c,
                    t_cool_c: ts.t_cool_c,
                    ect_filt_c: ts.ect_filt_c,
                },
                tp,
            ));
        } else if inputs.thermal.is_some() {
            let tp = inputs
                .thermal
                .unwrap_or(thermal::ThermalParams::illustrative());
            ws.therm = Some((thermal::ThermalState::cold(20.0), tp));
        }
        if let Some((st, p)) = &ws.therm {
            // Energía inicial representable; un snapshot autenticado no se
            // asocia con una configuración térmica inválida (R02-C2/C1-02).
            ws.th_u0_j = st.stored_j_checked(p).ok_or_else(|| EngineError::Invalid {
                reason: "thermal stored energy not representable".to_owned(),
            })?;
        }
        // Historia de presión para fricción (F07): nativa si existe; en
        // snapshots previos se migra a una cota inferior física declarada
        // (máximo entre la presión ambiente y las presiones de cilindro
        // actuales), nunca a un valor inventado de régimen.
        let r_gas = inputs.mix.r();
        let p_amb = 101_325.0;
        let p_cyl_max = ws
            .cyls
            .iter()
            .enumerate()
            .map(|(k, c)| {
                let local = cylinder_local_angle(ws.theta_rad.to_degrees(), CYLS[k]);
                let crank = crank_angle_for_piston(local);
                let v = inputs.geom.volume(crank.to_radians());
                let t_cyl = c.u / (c.m * cv);
                c.m * r_gas * t_cyl / v
            })
            .fold(p_amb, f64::max);
        ws.pmax_pa = s.pmax_pa.unwrap_or(p_cyl_max);
        // Identidad de modelo (R02-C2/C1-05): el esquema de serialización NO
        // identifica al productor. Un snapshot sin marcador `model` es
        // ambiguo entre R02 (contabilidad diferida) y C1 (por paso): se migra
        // con procedencia aproximada, nunca como nativo exacto. Un marcador
        // con convención desconocida se rechaza (fail-closed).
        let native_model = if s.schema_version == 1 {
            false
        } else {
            match &snap.model {
                None => false,
                Some(m) => {
                    if m.schema_version != s.schema_version {
                        return Err(EngineError::Invalid {
                            reason: "model/schema version mismatch".to_owned(),
                        });
                    }
                    if m.combustion_accounting != crate::state::COMBUSTION_ACCOUNTING_STEPWISE {
                        return Err(EngineError::Invalid {
                            reason: "unsupported combustion accounting version".to_owned(),
                        });
                    }
                    true
                }
            }
        };
        // Procedencia de migración legacy (R02-C1/RV04 + C1-05): separa
        // integridad de lectura de equivalencia de continuación física.
        ws.provenance = snap.provenance.clone();
        let legacy = !native_model;
        if ws.provenance.is_none() && legacy {
            ws.provenance = Some(MigrationProvenance {
                from_schema: s.schema_version,
                original_hash: snap.state_hash.clone(),
                transformation: legacy_transformation(s, snap.model.as_ref()),
                missing_fields: missing_history_fields(s),
                continuation: if is_neutral_initial(s) {
                    "exact_neutral"
                } else {
                    "approximate_hot"
                }
                .to_owned(),
            });
        }
        ws.m_sys_0_kg = ws.plen.m + ws.exh.m + ws.cyls.iter().map(|c| c.m).sum::<f64>();
        ws.u_sys_0_j = ws.plen.u() + ws.exh.u() + ws.cyls.iter().map(|c| c.u).sum::<f64>();
        // Fail-closed: hash válido no implica estado físico válido (R02-F15).
        validate_physical_state(&ws, "restored snapshot")?;
        Ok(ws)
    }
}

/// Campos históricos que un snapshot legacy no representaba (R02-C2/C1-05).
fn missing_history_fields(s: &EngineState) -> Vec<String> {
    let mut missing = Vec::new();
    if s.schema_version == 1 {
        missing.push("thermal".to_owned());
    }
    if s.schema_version == 1 || s.pmax_pa.is_none() {
        missing.extend(["plen_fuel_kg", "exh_fuel_kg", "fuel_emitted_kg"].map(str::to_owned));
    }
    if s.pmax_pa.is_none() {
        missing.push("pmax_pa".to_owned());
    }
    // Todo snapshot pre-C2 carece del marcador de convención de contabilidad.
    missing.push("combustion_accounting".to_owned());
    missing
}

/// ¿Estado inicial neutro (sin historia)? Entonces los valores por defecto
/// son la historia conocida y la continuación no arrastra convención previa.
fn is_neutral_initial(s: &EngineState) -> bool {
    s.time_s == 0.0
        && s.omega_rad_s == 0.0
        && s.fuel_injected_kg == 0.0
        && s.fuel_burned_kg == 0.0
        && s.heat_released_j == 0.0
        && s.cyls.iter().all(|c| c.fuel_kg == 0.0)
        && s.plen_fuel_kg == 0.0
        && s.exh_fuel_kg == 0.0
}

/// Descripción de la transformación aplicada al migrar un snapshot legacy.
fn legacy_transformation(s: &EngineState, model: Option<&ModelIdentity>) -> String {
    let accounting = match model {
        None => "combustion accounting absent (pre-C2: ambiguous R02 deferred vs C1 stepwise)",
        Some(m) => match m.combustion_accounting {
            crate::state::COMBUSTION_ACCOUNTING_STEPWISE => "combustion accounting stepwise",
            _ => "combustion accounting unknown",
        },
    };
    format!(
        "schema {} -> 2: {accounting}; node fuel inventories/pmax defaulted per policy; thermal per inputs",
        s.schema_version
    )
}

/// Valida magnitudes físicas del estado de trabajo (masas/temperaturas/
/// fracciones/energías finitas y en dominio). Fail-closed antes de avanzar.
fn validate_physical_state(ws: &WorkState, ctx: &str) -> Result<(), EngineError> {
    let bad = |why: &str| EngineError::Invalid {
        reason: format!("{ctx}: {why}"),
    };
    if !(ws.time_s.is_finite()
        && ws.time_s >= 0.0
        && ws.theta_rad.is_finite()
        && ws.omega_rad_s.is_finite()
        && ws.omega_rad_s >= 0.0
        && ws.pmax_pa.is_finite()
        && ws.pmax_pa > 0.0)
    {
        return Err(bad("scalar state"));
    }
    for c in &ws.cyls {
        if !(c.m.is_finite()
            && c.m > 0.0
            && c.u.is_finite()
            && c.u > 0.0
            && c.fresh.is_finite()
            && (0.0..=1.0).contains(&c.fresh)
            && c.fuel.is_finite()
            && c.fuel >= 0.0
            && c.fuel <= c.m * (1.0 + 1e-9)
            && c.burn_latched.is_finite()
            && c.burn_latched >= 0.0)
        {
            return Err(bad("cylinder state"));
        }
    }
    for n in [&ws.plen, &ws.exh] {
        if !(n.m.is_finite()
            && n.m > 0.0
            && n.t.is_finite()
            && n.t > 0.0
            && n.fresh.is_finite()
            && (0.0..=1.0).contains(&n.fresh)
            && n.fuel.is_finite()
            && n.fuel >= 0.0
            && n.fuel <= n.m * (1.0 + 1e-9)
            // Volumen y cv del nodo: finitos y positivos (R02-C2/C1-01).
            && n.v.is_finite()
            && n.v > 0.0
            && n.cv.is_finite()
            && n.cv > 0.0)
        {
            return Err(bad("node state"));
        }
    }
    // Historia de eventos y contadores/ledgers (R02-C1/RV05): un hash válido
    // o un estado con masa positiva no autoriza NaN/Inf ni contadores
    // negativos imposibles.
    if !ws.prev_local_deg.iter().all(|v| v.is_finite()) {
        return Err(bad("event history"));
    }
    for (v, name) in [
        (ws.heat_released_j, "heat_released"),
        (ws.fuel_injected_kg, "fuel_injected"),
        (ws.fuel_burned_kg, "fuel_burned"),
        (ws.fuel_emitted_kg, "fuel_emitted"),
        (ws.m_in_amb_kg, "m_in_amb"),
        (ws.m_out_amb_kg, "m_out_amb"),
        (ws.h_in_amb_j, "h_in_amb"),
        (ws.h_out_amb_j, "h_out_amb"),
        (ws.h_fuel_in_j, "h_fuel_in"),
        (ws.w_ind_j, "w_ind"),
        (ws.w_fric_j, "w_fric"),
        (ws.w_starter_j, "w_starter"),
        (ws.w_load_j, "w_load"),
        (ws.th_q_amb_j, "th_q_amb"),
        (ws.m_sys_0_kg, "m_sys_0"),
        (ws.u_sys_0_j, "u_sys_0"),
    ] {
        if !v.is_finite() {
            return Err(bad(name));
        }
    }
    for (v, name) in [
        (ws.heat_released_j, "heat_released"),
        (ws.fuel_injected_kg, "fuel_injected"),
        (ws.fuel_burned_kg, "fuel_burned"),
        (ws.fuel_emitted_kg, "fuel_emitted"),
        (ws.m_in_amb_kg, "m_in_amb"),
        (ws.m_out_amb_kg, "m_out_amb"),
    ] {
        if v < 0.0 {
            return Err(bad(name));
        }
    }
    if !ws.th_u0_j.is_finite() {
        return Err(bad("thermal initial energy"));
    }
    if let Some((st, p)) = &ws.therm {
        // Parámetros y energía derivada representable (R02-C2/C1-02, C1-04).
        p.validated().map_err(|_| bad("thermal params"))?;
        for v in [
            st.t_wall_c,
            st.t_piston_c,
            st.t_oil_c,
            st.t_cool_c,
            st.ect_filt_c,
        ] {
            if !(v.is_finite() && v > -273.15) {
                return Err(bad("thermal state"));
            }
        }
        if st.stored_j_checked(p).is_none() {
            return Err(bad("thermal stored energy not representable"));
        }
    }
    Ok(())
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
fn initial_state(inputs: &EngineInputs) -> Result<WorkState, EngineError> {
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
        fuel: 0.0,
    };
    let exh = NodeW {
        m: p_amb * inputs.exhaust_v_m3 / (r_gas * t_amb),
        t: t_amb,
        v: inputs.exhaust_v_m3,
        fresh: 0.0,
        cv,
        fuel: 0.0,
    };
    let m0 = plen.m + exh.m + cyls.iter().map(|c| c.m).sum::<f64>();
    let u0 = plen.u() + exh.u() + cyls.iter().map(|c| c.u).sum::<f64>();
    // Red térmica en frío a 20 °C (coherente con t_amb de gases).
    let (therm, th_u0) = match inputs.thermal {
        Some(p) => {
            p.validated().map_err(|_| EngineError::Invalid {
                reason: "thermal params".to_owned(),
            })?;
            let st = thermal::ThermalState::cold(20.0);
            let u = st
                .stored_j_checked(&p)
                .ok_or_else(|| EngineError::Invalid {
                    reason: "thermal stored energy not representable".to_owned(),
                })?;
            (Some((st, p)), u)
        }
        None => (None, 0.0),
    };
    let prev_local_deg = [
        cylinder_local_angle(0.0, CYLS[0]),
        cylinder_local_angle(0.0, CYLS[1]),
        cylinder_local_angle(0.0, CYLS[2]),
        cylinder_local_angle(0.0, CYLS[3]),
    ];
    Ok(WorkState {
        time_s: 0.0,
        theta_rad: 0.0,
        omega_rad_s: 0.0,
        prev_local_deg,
        cyls,
        plen,
        exh,
        therm,
        th_u0_j: th_u0,
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
        pmax_pa: p_amb,
        th_q_amb_j: 0.0,
        fuel_emitted_kg: 0.0,
        provenance: None,
    })
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
    let dt = inputs.dt_s;
    let coil = Coil {
        inductance_h: 5e-3,
        min_spark_energy_j: 30e-3,
        dwell_s: inputs.dwell_s,
    };
    let regulator = Regulator::new(inputs.regulator_dp_pa).map_err(|_| bad("regulator"))?;
    let mut samples = Vec::new();
    let mut step = step_offset;

    while ws.time_s < t_end - 1e-15 {
        let th_deg = ws.theta_rad.to_degrees();
        let mut tau_ind = 0.0;
        let mut fuel_rate = 0.0;
        let mut dq_wall_step = 0.0;

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
            ws.pmax_pa = ws.pmax_pa.max(p_cyl);
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
            // Admisión plen↔cyl (aplica a ambos lados). R01-F01: la especie
            // combustible se advecta con la masa (homogénea) y se reintegra
            // en el reflujo.
            if p_plen > p_cyl {
                let m = flow::mass_flow(p_plen, ws.plen.t, p_cyl, a_iv, r_gas, gamma)
                    .map_err(|_| bad("iv flow"))?;
                let dm = m * dt;
                let f_in = ws.plen.fresh;
                let h_in = cp * ws.plen.t;
                let df = ws.plen.remove(dm, h_in)?;
                let m_prev = c.m;
                c.m += dm;
                c.fuel += df;
                c.fresh = (c.fresh * m_prev + f_in * dm) / c.m;
                c.u += dm * h_in;
            } else if p_cyl > p_plen {
                let m = flow::mass_flow(p_cyl, t_cyl, p_plen, a_iv, r_gas, gamma)
                    .map_err(|_| bad("iv flow"))?;
                let dm = m * dt;
                if !(c.m - dm > 0.0) {
                    return Err(bad("cylinder mass"));
                }
                let df = c.fuel * dm / c.m;
                c.m -= dm;
                c.fuel -= df;
                c.u -= dm * cp * t_cyl;
                ws.plen.add_in(dm, c.fresh, df, dm * cp * t_cyl)?;
            }
            // Escape cyl↔exh.
            if p_cyl > p_exh {
                let m = flow::mass_flow(p_cyl, t_cyl, p_exh, a_ev, r_gas, gamma)
                    .map_err(|_| bad("ev flow"))?;
                let dm = m * dt;
                if !(c.m - dm > 0.0) {
                    return Err(bad("cylinder mass"));
                }
                let df = c.fuel * dm / c.m;
                c.m -= dm;
                c.fuel -= df;
                c.u -= dm * cp * t_cyl;
                ws.exh.add_in(dm, c.fresh, df, dm * cp * t_cyl)?;
            } else if p_exh > p_cyl {
                let m = flow::mass_flow(p_exh, ws.exh.t, p_cyl, a_ev, r_gas, gamma)
                    .map_err(|_| bad("ev flow"))?;
                let dm = m * dt;
                let (f_in, h_in) = (ws.exh.fresh, cp * ws.exh.t);
                let df = ws.exh.remove(dm, h_in)?;
                let m_prev = c.m;
                c.m += dm;
                c.fuel += df;
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
                let mut cmd =
                    inputs
                        .control
                        .fuel_command(rho_man, vd_cyl, inputs.fuel, inputs.injector);
                // Enriquecimiento por ECT (S21, cadena L1 declarada).
                let enrich = ws
                    .therm
                    .as_ref()
                    .map(|(st, tp)| st.enrich_factor(tp))
                    .unwrap_or(1.0);
                cmd.mass_kg *= enrich;
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
            // Wiebe con reloj local (solo válvulas cerradas). Con red
            // térmica, una fracción fw va a paredes (el gas conserva 1−fw):
            // sin red, fw = 0 (comportamiento S16 exacto).
            //
            // R01-C1/RV01: la energía liberada en el paso consume el
            // inventario químico EN LA MISMA CONVENCIÓN TEMPORAL. La masa
            // quemada del paso es la que corresponde a dQ/(LHV·η); no se
            // difiere la contabilidad al final de la ventana (eso permitía
            // sumar calor liberado y combustible aún intacto como energías
            // independientes). El remanente no quemado (completitud < 1)
            // permanece en `c.fuel` y se transporta como especie.
            if c.burn_latched > 0.0 && !c.burn_accounted && li < 0.05 && le < 0.05 {
                let w = combustion::Wiebe::new(
                    inputs.wiebe_a,
                    inputs.wiebe_m,
                    fire_local.to_radians(),
                    inputs.wiebe_duration_deg.to_radians(),
                )
                .map_err(|_| bad("wiebe"))?;
                let rate = w.burned_rate(local.to_radians());
                let dm_burn = (c.burn_latched * rate * ws.omega_rad_s * dt)
                    .max(0.0)
                    .min(c.fuel);
                let dq = dm_burn * inputs.fuel.lhv_j_per_kg * inputs.combustion_efficiency;
                let fw = ws.therm.as_ref().map(|(_, p)| p.wall_frac).unwrap_or(0.0);
                c.u += dq * (1.0 - fw);
                ws.heat_released_j += dq;
                dq_wall_step += dq * fw;
                c.fuel -= dm_burn;
                ws.fuel_burned_kg += dm_burn;
                if local > fire_local + inputs.wiebe_duration_deg {
                    c.burn_accounted = true;
                }
            }

            if !(c.u.is_finite() && c.u > 0.0 && c.m > 0.0) {
                return Err(bad("cylinder energy"));
            }
            ws.prev_local_deg[k] = local;
        }

        // Frontera ambiente. R01-F01: el combustible que cruza a ambiente se
        // contabiliza aparte (aire fresco entra con fracción de fuel nula).
        let p_plen = ws.plen.p(r_gas);
        let a_thr = inputs.throttle_area_m2 + inputs.idle_bypass_area_m2;
        if p_amb > p_plen {
            let m = flow::mass_flow(p_amb, t_amb, p_plen, a_thr, r_gas, gamma)
                .map_err(|_| bad("throttle"))?;
            let dm = m * dt;
            ws.plen.add_in(dm, 1.0, 0.0, dm * cp * t_amb)?;
            ws.m_in_amb_kg += dm;
            ws.h_in_amb_j += dm * cp * t_amb;
        } else if p_plen > p_amb {
            let m = flow::mass_flow(p_plen, ws.plen.t, p_amb, a_thr, r_gas, gamma)
                .map_err(|_| bad("throttle"))?;
            let dm = m * dt;
            let h = cp * ws.plen.t;
            let df = ws.plen.remove(dm, h)?;
            ws.fuel_emitted_kg += df;
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
            let df = ws.exh.remove(dm, h)?;
            ws.fuel_emitted_kg += df;
            ws.m_out_amb_kg += dm;
            ws.h_out_amb_j += dm * h;
        } else if p_amb > p_exh {
            let m = flow::mass_flow(p_amb, t_amb, p_exh, inputs.tailpipe_area_m2, r_gas, gamma)
                .map_err(|_| bad("tailpipe"))?;
            let dm = m * dt;
            ws.exh.add_in(dm, 1.0, 0.0, dm * cp * t_amb)?;
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
                // R02-C2/C1-01: los errores del starter se propagan; no se
                // sustituyen por un par físico cero silencioso.
                let cur = dr
                    .current(sm, ws.omega_rad_s)
                    .map_err(|_| bad("starter current"))?;
                let v = bat
                    .terminal_voltage(cur)
                    .map_err(|_| bad("battery voltage"))?;
                dr.crank_torque(sm, ws.omega_rad_s)
                    .map_err(|_| bad("starter torque"))?
                    * (v / 12.8).clamp(0.0, 1.0)
            } else {
                0.0
            }
        } else {
            0.0
        };
        let sp = 2.0 * inputs.geom.stroke().as_m() * rpm_now.max(0.0) / 60.0;
        // R02-C1/RV05: un error de fricción no se convierte en torque cero
        // silencioso; se propaga como error tipado.
        let tau_fric = inputs
            .friction
            .torque_mag(sp, ws.pmax_pa, vd_cyl * 4.0, ws.omega_rad_s)
            .map_err(|_| bad("friction torque"))?;
        let acc = (tau_ind - tau_fric - inputs.accessory_tau_n_m + tau_starter) / j_eff;
        let om_new = (ws.omega_rad_s + acc * dt).max(0.0);
        let om_avg = (ws.omega_rad_s + om_new) / 2.0;
        let p_fric_w = tau_fric * om_avg;
        ws.w_ind_j += tau_ind * om_avg * dt;
        ws.w_fric_j += p_fric_w * dt;
        ws.w_starter_j += tau_starter * om_avg * dt;
        ws.w_load_j += inputs.accessory_tau_n_m * om_avg * dt;
        // Red térmica: paredes reciben su fracción del quemado y el aceite
        // la fricción (mismos julios que la auditoría: exacto). El calor neto
        // cedido al ambiente se acumula para el ledger (R01-F10).
        if let Some((st, tp)) = ws.therm.as_mut() {
            let q_amb = st
                .step(dt, dq_wall_step, p_fric_w, 20.0, tp)
                .map_err(|_| bad("thermal step"))?;
            ws.th_q_amb_j += q_amb;
        }
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
    // Con red térmica, la fricción calienta el aceite (interna, no sumidero)
    // y el almacén suma la red; sin red, fórmula S16 exacta.
    let (u_th_1, fric_sink) = match &ws.therm {
        Some((st, tp)) => (st.stored_j(tp), 0.0),
        None => (0.0, ws.w_fric_j),
    };
    let scale_e = (ws.u_sys_0_j + ws.th_u0_j + ws.heat_released_j + ws.w_starter_j).max(1e-9);
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
        // W_starter; con red térmica el sumidero es solo W_load (la fricción
        // queda en el aceite) y el almacén suma la red; sin red, W_fric es
        // sumidero (fórmula S16 exacta). El calor cedido al ambiente por la
        // red (R01-F10) entra como sumidero declarado.
        energy_closure: ((u_sys_1 - ws.u_sys_0_j + (u_th_1 - ws.th_u0_j) + ke_1)
            + (fric_sink + ws.w_load_j + ws.th_q_amb_j)
            - ((ws.h_in_amb_j - ws.h_out_amb_j)
                + ws.h_fuel_in_j
                + ws.heat_released_j
                + ws.w_starter_j))
            / scale_e,
        fuel_stored_kg: ws.cyls.iter().map(|c| c.fuel).sum::<f64>() + ws.plen.fuel + ws.exh.fuel,
        fuel_emitted_kg: ws.fuel_emitted_kg,
        th_q_amb_j: ws.th_q_amb_j,
        pmax_pa: ws.pmax_pa,
        final_rpm: ws.omega_rad_s * 60.0 / (2.0 * std::f64::consts::PI),
        delta_ke_j: 0.5 * j_eff * ws.omega_rad_s * ws.omega_rad_s,
        h_amb_net_j: ws.h_in_amb_j - ws.h_out_amb_j,
        h_fuel_j: ws.h_fuel_in_j,
        thermal: ws.therm.as_ref().map(|(st, tp)| ThermalSummary {
            ect_c: st.ect_filt_c,
            t_wall_c: st.t_wall_c,
            t_oil_c: st.t_oil_c,
            t_cool_c: st.t_cool_c,
            thermostat_0_1: st.thermostat_factor(tp),
        }),
    }
}

/// Ejecuta el motor desde reposo (θ = 0, ω = 0).
///
/// # Errors
///
/// `Invalid` ante parámetros inválidos o estado no físico (fail-closed).
pub fn run_engine(inputs: &EngineInputs) -> Result<RunTrace, EngineError> {
    validate_inputs(inputs)?;
    let mut ws = initial_state(inputs).map_err(|_| EngineError::Invalid {
        reason: "initial state".to_owned(),
    })?;
    let samples = drive(&mut ws, inputs, inputs.t_end_s, inputs.record_every, 0)?;
    // Un `Ok` debe contener estado y auditoría finitos (R02-C2/C1-01, C1-04):
    // no se devuelve un ledger energético no finito ni un estado imposible.
    validate_physical_state(&ws, "run_engine output")?;
    let j_eff = crank::effective_inertia_mean(&inputs.geom, inputs.inertia);
    let audit = audit_of(&ws, inputs.fuel.lhv_j_per_kg, j_eff);
    if !(audit.mass_closure.is_finite()
        && audit.energy_closure.is_finite()
        && audit.fuel_injected_kg.is_finite()
        && audit.fuel_burned_kg.is_finite()
        && audit.fuel_stored_kg.is_finite()
        && audit.fuel_emitted_kg.is_finite()
        && audit.heat_released_j.is_finite()
        && audit.w_ind_j.is_finite()
        && audit.w_fric_j.is_finite()
        && audit.w_starter_j.is_finite()
        && audit.w_load_j.is_finite()
        && audit.th_q_amb_j.is_finite()
        && audit.pmax_pa.is_finite()
        && audit.delta_u_j.is_finite()
        && audit.delta_ke_j.is_finite()
        && audit.h_amb_net_j.is_finite()
        && audit.h_fuel_j.is_finite()
        && audit.final_rpm.is_finite())
    {
        return Err(EngineError::Invalid {
            reason: "non-finite run audit".to_owned(),
        });
    }
    Ok(RunTrace { samples, audit })
}

/// Valida el contrato de entradas antes de cualquier mutación/cálculo
/// (fail-closed, R02-F15 / R02-C2/C1-01): cubre control open-loop, combustible,
/// eléctrico/arranque, inercia, térmico y derivadas representables. Evita
/// panics (p. ej. `record_every = 0`) y configuraciones imposibles. No impone
/// cotas OEM: solo finitud, positividad y coherencia de dominio.
fn validate_inputs(inputs: &EngineInputs) -> Result<(), EngineError> {
    let bad = |r: &str| EngineError::Invalid {
        reason: r.to_owned(),
    };
    // Temporal.
    if !(inputs.dt_s.is_finite() && inputs.dt_s > 0.0 && inputs.dt_s <= 2e-4) {
        return Err(bad("dt in (0, 200us]"));
    }
    if !(inputs.t_end_s.is_finite() && inputs.t_end_s >= 0.0) {
        return Err(bad("t_end finite"));
    }
    if inputs.record_every == 0 {
        return Err(bad("record_every must be > 0"));
    }
    // Combustión.
    if !(inputs.combustion_efficiency.is_finite()
        && (0.0..=1.0).contains(&inputs.combustion_efficiency))
    {
        return Err(bad("combustion_efficiency in [0, 1]"));
    }
    if !(inputs.wiebe_a.is_finite() && inputs.wiebe_a > 0.0) {
        return Err(bad("wiebe_a"));
    }
    if !(inputs.wiebe_m.is_finite() && inputs.wiebe_m >= 0.0) {
        return Err(bad("wiebe_m"));
    }
    if !(inputs.wiebe_duration_deg.is_finite() && inputs.wiebe_duration_deg > 0.0) {
        return Err(bad("wiebe_duration"));
    }
    // Control open-loop (R02-C2/C1-01).
    if !(inputs.control.afr_target.is_finite() && inputs.control.afr_target > 0.0) {
        return Err(bad("afr_target"));
    }
    if !(inputs.control.eta_v_assumed.is_finite()
        && inputs.control.eta_v_assumed > 0.0
        && inputs.control.eta_v_assumed <= 1.0)
    {
        return Err(bad("eta_v_assumed in (0, 1]"));
    }
    if !inputs.control.spark_adv_btdc_deg.is_finite() {
        return Err(bad("spark_adv"));
    }
    // Combustible.
    if !(inputs.fuel.lhv_j_per_kg.is_finite() && inputs.fuel.lhv_j_per_kg > 0.0) {
        return Err(bad("lhv"));
    }
    if !(inputs.fuel.density_kg_m3.is_finite() && inputs.fuel.density_kg_m3 > 0.0) {
        return Err(bad("fuel density"));
    }
    if !(inputs.fuel.stoich_afr.is_finite() && inputs.fuel.stoich_afr > 0.0) {
        return Err(bad("stoich_afr"));
    }
    // Eléctrico / arranque.
    if !(inputs.charge_volts.is_finite() && inputs.charge_volts > 0.0) {
        return Err(bad("charge_volts"));
    }
    if !(inputs.dwell_s.is_finite() && inputs.dwell_s >= 0.0) {
        return Err(bad("dwell"));
    }
    if !(inputs.starter_cutout_rpm.is_finite() && inputs.starter_cutout_rpm >= 0.0) {
        return Err(bad("starter_cutout"));
    }
    if !(inputs.crank_timeout_s.is_finite() && inputs.crank_timeout_s >= 0.0) {
        return Err(bad("crank_timeout"));
    }
    if let Some((sm, dr, bat)) = &inputs.starter {
        if !(dr.ratio.is_finite() && dr.ratio > 0.0) {
            return Err(bad("starter ratio"));
        }
        sm.validated().map_err(|_| bad("starter motor"))?;
        bat.validated().map_err(|_| bad("battery"))?;
    }
    // Inercia.
    if !(inputs.inertia.rotating_j_kg_m2.is_finite() && inputs.inertia.rotating_j_kg_m2 > 0.0) {
        return Err(bad("inertia J > 0"));
    }
    if !(inputs.inertia.recip_mass_kg.is_finite() && inputs.inertia.recip_mass_kg >= 0.0) {
        return Err(bad("inertia mass"));
    }
    // Presión, áreas y fricción.
    if !(inputs.regulator_dp_pa.is_finite() && inputs.regulator_dp_pa >= 0.0) {
        return Err(bad("regulator dp"));
    }
    for (v, name) in [
        (inputs.throttle_area_m2, "throttle_area"),
        (inputs.idle_bypass_area_m2, "idle_bypass_area"),
        (inputs.tailpipe_area_m2, "tailpipe_area"),
    ] {
        // Área negativa no es una sección física (R02-C1/RV05).
        if !(v.is_finite() && v >= 0.0) {
            return Err(bad(name));
        }
    }
    if !inputs.accessory_tau_n_m.is_finite() {
        return Err(bad("accessory_tau"));
    }
    if !(inputs.plenum_v_m3.is_finite() && inputs.plenum_v_m3 > 0.0) {
        return Err(bad("plenum volume"));
    }
    if !(inputs.exhaust_v_m3.is_finite() && inputs.exhaust_v_m3 > 0.0) {
        return Err(bad("exhaust volume"));
    }
    match inputs.friction {
        FrictionModel::None => {}
        FrictionModel::Viscous { c_n_m_s } => {
            if !(c_n_m_s.is_finite() && c_n_m_s >= 0.0) {
                return Err(bad("friction c"));
            }
        }
        FrictionModel::ChenFlynn { coeffs } => {
            for (v, name) in [
                (coeffs.c1_pa, "c1"),
                (coeffs.c2_dlless, "c2"),
                (coeffs.c3_pa_s_m, "c3"),
                (coeffs.c4_pa_s2_m2, "c4"),
            ] {
                if !(v.is_finite() && v >= 0.0) {
                    return Err(bad(name));
                }
            }
        }
    }
    // Térmico: parámetros válidos y energía derivada representable a la
    // temperatura inicial declarada (R02-C2/C1-02, C1-04).
    if let Some(tp) = &inputs.thermal {
        tp.validated().map_err(|_| bad("thermal params"))?;
        if thermal::ThermalState::cold(20.0)
            .stored_j_checked(tp)
            .is_none()
        {
            return Err(bad("thermal stored energy not representable"));
        }
    }
    Ok(())
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
    validate_inputs(inputs)?;
    if !(t_end.is_finite() && t_end >= ws.time_s) {
        return Err(EngineError::Invalid {
            reason: "t_end must be >= state time".to_owned(),
        });
    }
    // Atomicidad (R02-C1/RV05): se valida el estado de entrada y se avanza
    // sobre una COPIA; la copia solo se confirma si `drive` termina bien. Un
    // error no deja el WorkState ni los ledgers parcialmente mutados.
    validate_physical_state(ws, "resume input")?;
    // Coherencia de configuración térmica entre estado e inputs
    // (R02-C2/C1-01): red presente sin inputs o al revés es inconsistente,
    // incluso en un avance nulo.
    if ws.therm.is_some() != inputs.thermal.is_some() {
        return Err(EngineError::Invalid {
            reason: "thermal presence mismatch between state and inputs".to_owned(),
        });
    }
    let mut candidate = ws.clone();
    // Rebase de auditoría para el tramo (el cierre total se verifica en
    // corridas completas; aquí el contrato es bit-identidad del estado).
    let m0 = candidate.plen.m + candidate.exh.m + candidate.cyls.iter().map(|c| c.m).sum::<f64>();
    let u0 =
        candidate.plen.u() + candidate.exh.u() + candidate.cyls.iter().map(|c| c.u).sum::<f64>();
    candidate.m_sys_0_kg = m0;
    candidate.u_sys_0_j = u0;
    candidate.m_in_amb_kg = 0.0;
    candidate.m_out_amb_kg = 0.0;
    candidate.h_in_amb_j = 0.0;
    candidate.h_out_amb_j = 0.0;
    candidate.h_fuel_in_j = 0.0;
    let samples = drive(&mut candidate, inputs, t_end, inputs.record_every, 0)?;
    // El candidato solo se confirma si además es un estado físico válido
    // (R02-C2/C1-01): una ejecución Ok no puede contener ledgers no finitos.
    validate_physical_state(&candidate, "resume output")?;
    *ws = candidate;
    Ok(samples)
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
        let mut ws = initial_state(&inp).unwrap();
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
        let mut ws = initial_state(&inp).unwrap();
        let _ = drive(&mut ws, &inp, 1.0, inp.record_every, 0).unwrap();
        let snap = ws.snapshot();
        assert!(snap.verified());
        let mut ws2 = WorkState::restore(&snap, &inp).unwrap();
        let _ = resume(&mut ws2, &inp, 2.0).unwrap();
        let snap_full = {
            let mut wsf = initial_state(&inp).unwrap();
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

#[cfg(test)]
mod s21_tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn warm_inputs(t_end: f64) -> EngineInputs {
        let mut inp = healthy_inputs();
        inp.throttle_area_m2 = 0.05e-4;
        inp.idle_bypass_area_m2 = 0.02e-4;
        inp.thermal = Some(thermal::ThermalParams::illustrative());
        inp.t_end_s = t_end;
        inp
    }

    #[test]
    fn cold_start_warms_ect_and_enriches() {
        // Gate S21: el calor del quemado calienta ECT (20 °C → arriba) y el
        // enriquecimiento por ECT actúa al principio (más combustible que
        // sin enriquecer). Sin red térmica estos campos no existen.
        let trace = run_engine(&warm_inputs(30.0)).unwrap();
        let th = trace.audit.thermal.expect("thermal summary");
        // Un calentamiento real tarda minutos; 30 s muestran el inicio
        // (dirección correcta desde 20 °C: ECT≈20.33 por lag) + el acople
        // ECT→combustible, que es la evidencia causal del gate.
        assert!(th.ect_c > 20.2, "ect={}", th.ect_c);
        assert!(th.t_oil_c > 20.0 && th.t_wall_c > 20.0);
        assert!(trace.audit.mass_closure.abs() < 1e-9);
        assert!(trace.audit.energy_closure.abs() < 2e-3);
        // Sin enriquecer (misma red) se inyecta menos en frío.
        let mut no_en = warm_inputs(10.0);
        no_en.thermal.as_mut().unwrap().ect_enrich = false;
        let mut en = warm_inputs(10.0);
        let _ = &mut en;
        let a_no = run_engine(&no_en).unwrap().audit;
        let a_en = run_engine(&warm_inputs(10.0)).unwrap().audit;
        assert!(
            a_en.fuel_injected_kg > a_no.fuel_injected_kg,
            "enrich must add fuel when cold"
        );
    }

    #[test]
    fn no_burn_no_wall_heating_and_snapshot_migrates() {
        // Sin quemado las paredes se quedan a ambiente (el aceite solo ve
        // fricción): todo calor con fuente declarada.
        let mut inp = warm_inputs(3.0);
        inp.control.fuel_enabled = false;
        inp.control.spark_enabled = false;
        let trace = run_engine(&inp).unwrap();
        let th = trace.audit.thermal.expect("thermal summary");
        assert!((th.t_wall_c - 20.0).abs() < 3.0, "wall={}", th.t_wall_c);
        assert!(th.t_oil_c >= 20.0);
        assert!(trace.audit.energy_closure.abs() < 2e-3);
        // Migración v1 → v2: snapshot antiguo sin red + entradas con red.
        let cyl = serde_json::json!({
            "m_kg": 4e-4, "u_j": 90.0, "fresh_frac": 1.0, "fuel_kg": 0.0,
            "burn_latched_kg": 0.0, "burn_accounted": true
        });
        let v1 = serde_json::json!({
            "schema_version": 1,
            "time_s": 1.0,
            "theta_global_rad": 0.5,
            "omega_rad_s": 80.0,
            "prev_local_deg": [0.0, 180.0, 540.0, 0.0],
            "cyls": [cyl.clone(), cyl.clone(), cyl.clone(), cyl.clone()],
            "plen_m_kg": 2e-3, "plen_t_k": 293.0, "plen_fresh": 1.0,
            "exh_m_kg": 1e-3, "exh_t_k": 293.0, "exh_fresh": 0.0,
            "fuel_injected_kg": 0.0, "fuel_burned_kg": 0.0, "heat_released_j": 0.0,
            "work_ind_j": 0.0, "work_fric_j": 0.0, "work_starter_j": 0.0, "work_load_j": 0.0
        });
        let snap: EngineSnapshot =
            serde_json::from_value(serde_json::json!({"state": v1, "state_hash": "00"})).unwrap();
        // Hash falso ⇒ rechaza antes de migrar (verificación primero).
        assert!(WorkState::restore(&snap, &warm_inputs(2.0)).is_err());
    }
}

#[cfg(test)]
mod r01_r02_tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn idle_point(mut inp: EngineInputs) -> EngineInputs {
        inp.throttle_area_m2 = 0.05e-4;
        inp.idle_bypass_area_m2 = 0.02e-4;
        inp
    }

    fn with_c2(mut inp: EngineInputs, c2: f64) -> EngineInputs {
        if let FrictionModel::ChenFlynn { ref mut coeffs } = inp.friction {
            coeffs.c2_dlless = c2;
        }
        inp
    }

    // ---- R01-F01: especies / combustible --------------------------------

    #[test]
    fn homogeneous_outflow_carries_fuel_proportionally() {
        // R01-F01: una cámara homogénea pierde combustible en proporción a la
        // masa que sale (la salida no deja el fuel atrás).
        let mut inp = idle_point(healthy_inputs());
        inp.control.fuel_enabled = false;
        inp.control.spark_enabled = false;
        let mut w = initial_state(&inp).unwrap();
        w.omega_rad_s = 100.0;
        w.theta_rad = std::f64::consts::PI / 2.0;
        let v = inp.geom.volume(std::f64::consts::PI / 2.0);
        w.cyls[0].m = 200_000.0 * v / (inp.mix.r() * 293.15);
        w.cyls[0].u = w.cyls[0].m * inp.mix.cv() * 293.15;
        w.cyls[0].fuel = w.cyls[0].m * 0.1;
        let (m0, f0) = (w.cyls[0].m, w.cyls[0].fuel);
        drive(&mut w, &inp, inp.dt_s, inp.record_every, 0).unwrap();
        assert!(w.cyls[0].m < m0, "fixture debe tener salida");
        let expected = f0 * w.cyls[0].m / m0;
        assert!(
            (w.cyls[0].fuel - expected).abs() < 1e-15,
            "fuel {} vs {}",
            w.cyls[0].fuel,
            expected
        );
    }

    #[test]
    fn fuel_inventory_bounded_and_conserved_without_spark() {
        // R01-F01: sin chispa prolongado, 0 ≤ fuel ≤ m y el presupuesto
        // inyectado = almacenado + quemado + emitido (frontera declarada).
        let mut inp = idle_point(healthy_inputs());
        inp.control.spark_enabled = false;
        inp.t_end_s = 4.0;
        let mut w = initial_state(&inp).unwrap();
        let mut max_ratio = 0.0f64;
        for n in 1..=200 {
            drive(&mut w, &inp, n as f64 * 0.02, inp.record_every, 0).unwrap();
            max_ratio = max_ratio.max(w.cyls.iter().map(|c| c.fuel / c.m).fold(0.0, f64::max));
        }
        assert!(max_ratio <= 1.0, "max fuel/mass ratio {max_ratio}");
        let stored: f64 = w.cyls.iter().map(|c| c.fuel).sum::<f64>() + w.plen.fuel + w.exh.fuel;
        let closure = w.fuel_injected_kg - (stored + w.fuel_burned_kg + w.fuel_emitted_kg);
        assert!(closure.abs() < 1e-9, "fuel closure {closure}");
    }

    #[test]
    fn burned_fuel_never_exceeds_injected_and_energy_needs_mass() {
        // R01-F01: no hay energía química sin masa quemada.
        let inp = idle_point(healthy_inputs());
        let trace = run_engine(&inp).unwrap();
        let a = trace.audit;
        assert!(a.fuel_burned_kg <= a.fuel_injected_kg);
        assert!(a.fuel_burned_kg > 0.0 && a.heat_released_j > 0.0);
        assert!(a.fuel_stored_kg >= 0.0 && a.fuel_emitted_kg >= 0.0);
    }

    // ---- R01-F07: historia de presión en replay -------------------------

    #[test]
    fn replay_split_with_pressure_history_is_bit_identical() {
        // R01-F07: con c2≠0, cualquier corte de ciclo reproduce el estado
        // continuo porque pmax se persiste (no se reinicia por llamada).
        let mut inp = idle_point(healthy_inputs());
        inp = with_c2(inp, 0.05);
        inp.t_end_s = 3.0;
        let mut full = initial_state(&inp).unwrap();
        drive(&mut full, &inp, 3.0, inp.record_every, 0).unwrap();
        for cut in [0.5, 1.0, 1.5, 2.0, 2.5] {
            let mut a = initial_state(&inp).unwrap();
            drive(&mut a, &inp, cut, inp.record_every, 0).unwrap();
            let snap = a.snapshot();
            assert!(snap.verified());
            let mut b = WorkState::restore(&snap, &inp).unwrap();
            drive(&mut b, &inp, 3.0, inp.record_every, 0).unwrap();
            assert_eq!(
                full.snapshot().state_hash,
                b.snapshot().state_hash,
                "cut={cut}"
            );
        }
        // Control c2=0 sigue coincidiendo.
        let mut inp0 = idle_point(healthy_inputs());
        inp0 = with_c2(inp0, 0.0);
        inp0.t_end_s = 3.0;
        let mut full0 = initial_state(&inp0).unwrap();
        drive(&mut full0, &inp0, 3.0, inp0.record_every, 0).unwrap();
        let mut a0 = initial_state(&inp0).unwrap();
        drive(&mut a0, &inp0, 1.0, inp0.record_every, 0).unwrap();
        let mut b0 = WorkState::restore(&a0.snapshot(), &inp0).unwrap();
        drive(&mut b0, &inp0, 3.0, inp0.record_every, 0).unwrap();
        assert_eq!(full0.snapshot().state_hash, b0.snapshot().state_hash);
    }

    // ---- R01-F10: ledger térmico con frontera ambiente ------------------

    #[test]
    fn thermal_ambient_loss_enters_energy_ledger() {
        let mut inp = idle_point(healthy_inputs());
        inp.thermal = Some(thermal::ThermalParams::illustrative());
        inp.t_end_s = 30.0;
        let trace = run_engine(&inp).unwrap();
        assert!(trace.audit.th_q_amb_j > 0.0, "debe ceder calor al ambiente");
        assert!(
            trace.audit.energy_closure.abs() < 1e-3,
            "closure {}",
            trace.audit.energy_closure
        );
        // Adiabático (hA a ambiente nulos) ⇒ sin término de sumidero.
        let mut p = thermal::ThermalParams::illustrative();
        p.ha_wall_amb = 0.0;
        p.ha_oil_amb = 0.0;
        p.ha_rad_full = 0.0;
        let mut inp2 = idle_point(healthy_inputs());
        inp2.thermal = Some(p);
        inp2.t_end_s = 30.0;
        let trace2 = run_engine(&inp2).unwrap();
        assert!(trace2.audit.th_q_amb_j.abs() < 1e-9);
        assert!(trace2.audit.energy_closure.abs() < 1e-3);
    }

    // ---- R02-F05: round-trip JSON lossless ------------------------------

    #[test]
    fn json_roundtrip_preserves_bits_and_hash() {
        let mut inp = idle_point(healthy_inputs());
        inp.t_end_s = 1.0;
        let mut w = initial_state(&inp).unwrap();
        drive(&mut w, &inp, 1.0, inp.record_every, 0).unwrap();
        let snap = w.snapshot();
        let bytes = engine_types::canonical_json(&snap);
        let parsed: EngineSnapshot = serde_json::from_str(&bytes).unwrap();
        assert!(parsed.verified());
        assert_eq!(parsed.state, snap.state);
        assert_eq!(
            engine_types::canonical_json(&parsed.state),
            engine_types::canonical_json(&snap.state)
        );
    }

    // ---- R02-F06: migración v1 auténtica --------------------------------

    #[test]
    fn v1_historical_hash_verifies_before_migration() {
        let cyl = serde_json::json!({
            "m_kg": 4e-4, "u_j": 90.0, "fresh_frac": 1.0, "fuel_kg": 0.0,
            "burn_latched_kg": 0.0, "burn_accounted": true
        });
        let v1 = serde_json::json!({
            "schema_version": 1,
            "time_s": 1.0, "theta_global_rad": 0.5, "omega_rad_s": 80.0,
            "prev_local_deg": [0.0, 180.0, 540.0, 0.0],
            "cyls": [cyl.clone(), cyl.clone(), cyl.clone(), cyl.clone()],
            "plen_m_kg": 2e-3, "plen_t_k": 293.0, "plen_fresh": 1.0,
            "exh_m_kg": 1e-3, "exh_t_k": 293.0, "exh_fresh": 0.0,
            "fuel_injected_kg": 0.0, "fuel_burned_kg": 0.0, "heat_released_j": 0.0,
            "work_ind_j": 0.0, "work_fric_j": 0.0, "work_starter_j": 0.0, "work_load_j": 0.0
        });
        let state: EngineState = serde_json::from_value(v1).unwrap();
        assert_eq!(state.schema_version, 1);
        let snap = EngineSnapshot::capture(&state);
        assert!(snap.verified());
        let inp = idle_point(healthy_inputs());
        assert!(WorkState::restore(&snap, &inp).is_ok());
        // Hash alterado ⇒ rechazo antes de migrar.
        let mut tampered = snap.clone();
        tampered.state_hash = "00".to_owned();
        assert!(WorkState::restore(&tampered, &inp).is_err());
        // Esquema desconocido ⇒ rechazo.
        let mut future = state.clone();
        future.schema_version = 99;
        let fs = EngineSnapshot::capture(&future);
        assert!(WorkState::restore(&fs, &inp).is_err());
    }

    // ---- R02-F15: fronteras fail-closed ---------------------------------

    #[test]
    fn invalid_inputs_fail_closed_without_panic() {
        let mut zero = idle_point(healthy_inputs());
        zero.record_every = 0;
        assert!(run_engine(&zero).is_err());
        let mut neg = idle_point(healthy_inputs());
        neg.combustion_efficiency = -1.0;
        assert!(run_engine(&neg).is_err());
        let mut over = idle_point(healthy_inputs());
        over.combustion_efficiency = 2.0;
        assert!(run_engine(&over).is_err());
        let mut bad_dt = idle_point(healthy_inputs());
        bad_dt.dt_s = 0.0;
        assert!(run_engine(&bad_dt).is_err());
    }

    #[test]
    fn restore_rejects_physically_impossible_state_with_valid_hash() {
        let inp = idle_point(healthy_inputs());
        let mut w = initial_state(&inp).unwrap();
        w.cyls[0].m = -1.0;
        // Hash válido sobre un estado físicamente imposible.
        let snap = w.snapshot();
        assert!(snap.verified());
        assert!(WorkState::restore(&snap, &inp).is_err());
        // Masa de nodo nula también se rechaza.
        let mut w2 = initial_state(&inp).unwrap();
        w2.plen.m = 0.0;
        let snap2 = w2.snapshot();
        assert!(snap2.verified());
        assert!(WorkState::restore(&snap2, &inp).is_err());
    }

    #[test]
    fn restore_rejects_thermal_mismatch_and_unknown_schema() {
        let mut inp = idle_point(healthy_inputs());
        inp.thermal = Some(thermal::ThermalParams::illustrative());
        let mut w = initial_state(&inp).unwrap();
        w.therm = None;
        let snap = w.snapshot();
        assert!(snap.verified());
        assert!(WorkState::restore(&snap, &inp).is_err());
    }

    // ---- R01-C1/RV01: presupuesto químico durante la combustión ----------

    #[test]
    fn chemical_energy_matches_consumed_fuel_during_burn() {
        // El calor liberado en cada paso no puede superar la energía del
        // combustible efectivamente consumido en ese mismo paso.
        let inp = idle_point(healthy_inputs());
        let mut w = initial_state(&inp).unwrap();
        let mut max_excess = 0.0f64;
        let mut saw_burn = false;
        for n in 1..=200 {
            drive(&mut w, &inp, n as f64 * 0.005, inp.record_every, 0).unwrap();
            let excess = w.heat_released_j
                - w.fuel_burned_kg * inp.fuel.lhv_j_per_kg * inp.combustion_efficiency;
            max_excess = max_excess.max(excess);
            if w.fuel_burned_kg > 0.0 {
                saw_burn = true;
            }
        }
        assert!(saw_burn, "el fixture debe quemar");
        assert!(max_excess < 1.0, "max chemical excess {max_excess} J");
    }

    #[test]
    fn burn_snapshot_cut_midwindow_resumes_coherently() {
        // Un corte dentro de la ventana de combustión conserva el presupuesto:
        // la continuación reproduce el estado continuo y el calor sigue
        // correspondiendo a la masa consumida.
        let mut inp = idle_point(healthy_inputs());
        inp.t_end_s = 2.0;
        let mut full = initial_state(&inp).unwrap();
        drive(&mut full, &inp, 2.0, inp.record_every, 0).unwrap();
        let mut found = false;
        for cut_ms in (500..1500).step_by(25) {
            let cut = cut_ms as f64 / 1000.0;
            let mut a = initial_state(&inp).unwrap();
            drive(&mut a, &inp, cut, inp.record_every, 0).unwrap();
            if a.fuel_burned_kg > 0.0 && a.fuel_burned_kg < a.fuel_injected_kg {
                let mut b = WorkState::restore(&a.snapshot(), &inp).unwrap();
                drive(&mut b, &inp, 2.0, inp.record_every, 0).unwrap();
                assert_eq!(
                    full.snapshot().state_hash,
                    b.snapshot().state_hash,
                    "cut={cut}"
                );
                let excess = b.heat_released_j
                    - b.fuel_burned_kg * inp.fuel.lhv_j_per_kg * inp.combustion_efficiency;
                assert!(excess.abs() < 1.0, "excess {excess}");
                found = true;
                break;
            }
        }
        assert!(found, "no mid-burn cut found");
    }

    #[test]
    fn insufficient_fuel_burns_no_more_than_available() {
        // Corte de combustible con inventario marcado: se consume como máximo
        // hasta agotarse y nada se vuelve negativo.
        let mut inp = idle_point(healthy_inputs());
        inp.control.fuel_enabled = false;
        let mut w = initial_state(&inp).unwrap();
        for c in &mut w.cyls {
            c.fuel = 0.001 * c.m;
        }
        let f0: f64 = w.cyls.iter().map(|c| c.fuel).sum::<f64>() + w.plen.fuel + w.exh.fuel;
        drive(&mut w, &inp, 1.0, inp.record_every, 0).unwrap();
        for c in &w.cyls {
            assert!(c.fuel >= 0.0 && c.fuel <= c.m);
        }
        let stored: f64 = w.cyls.iter().map(|c| c.fuel).sum::<f64>() + w.plen.fuel + w.exh.fuel;
        let closure = f0 + w.fuel_injected_kg - (stored + w.fuel_burned_kg + w.fuel_emitted_kg);
        assert!(closure.abs() < 1e-9, "fuel closure {closure}");
    }

    // ---- R02-C1/RV03: cero firmado ---------------------------------------

    #[test]
    fn signed_zero_bits_roundtrip() {
        let inp = idle_point(healthy_inputs());
        let w = initial_state(&inp).unwrap();
        for field in 0..3 {
            let mut st = w.snapshot().state;
            match field {
                0 => st.plen_fuel_kg = -0.0,
                1 => st.exh_fuel_kg = -0.0,
                _ => st.fuel_emitted_kg = -0.0,
            }
            let snap = EngineSnapshot::capture(&st);
            let parsed: EngineSnapshot =
                serde_json::from_str(&serde_json::to_string(&snap).unwrap()).unwrap();
            let after = match field {
                0 => parsed.state.plen_fuel_kg,
                1 => parsed.state.exh_fuel_kg,
                _ => parsed.state.fuel_emitted_kg,
            };
            assert_eq!(after.to_bits(), (-0.0f64).to_bits());
            assert!(parsed.verified());
            assert!(WorkState::restore(&parsed, &inp).is_ok());
        }
    }

    // ---- R02-C1/RV04: procedencia de migración ---------------------------

    #[test]
    fn legacy_migration_records_provenance() {
        let cyl = serde_json::json!({
            "m_kg": 4e-4, "u_j": 90.0, "fresh_frac": 1.0, "fuel_kg": 0.0,
            "burn_latched_kg": 0.0, "burn_accounted": true
        });
        let v1 = serde_json::json!({
            "schema_version": 1,
            "time_s": 0.0, "theta_global_rad": 0.0, "omega_rad_s": 0.0,
            "prev_local_deg": [0.0, 180.0, 540.0, 0.0],
            "cyls": [cyl.clone(), cyl.clone(), cyl.clone(), cyl.clone()],
            "plen_m_kg": 2e-3, "plen_t_k": 293.0, "plen_fresh": 1.0,
            "exh_m_kg": 1e-3, "exh_t_k": 293.0, "exh_fresh": 0.0,
            "fuel_injected_kg": 0.0, "fuel_burned_kg": 0.0, "heat_released_j": 0.0,
            "work_ind_j": 0.0, "work_fric_j": 0.0, "work_starter_j": 0.0, "work_load_j": 0.0
        });
        let state: EngineState = serde_json::from_value(v1).unwrap();
        let snap = EngineSnapshot::capture(&state);
        let inp = idle_point(healthy_inputs());
        let w = WorkState::restore(&snap, &inp).unwrap();
        let prov = w.snapshot().provenance.clone().unwrap();
        assert_eq!(prov.from_schema, 1);
        assert_eq!(prov.original_hash, snap.state_hash);
        assert!(prov.missing_fields.iter().any(|f| f == "pmax_pa"));
        assert!(prov.missing_fields.iter().any(|f| f == "thermal"));
        assert_eq!(prov.continuation, "exact_neutral");
        // Estado en marcha con historia ausente ⇒ continuación aproximada.
        // Se simula un productor pre-C2 limpiando el marcador de modelo.
        let mut hot = initial_state(&inp).unwrap();
        hot.provenance = None;
        let mut st = hot.snapshot().state;
        st.pmax_pa = None;
        st.time_s = 1.0;
        st.omega_rad_s = 50.0;
        let mut snap2 = EngineSnapshot::capture(&st);
        snap2.model = None;
        let w2 = WorkState::restore(&snap2, &inp).unwrap();
        let prov2 = w2.snapshot().provenance.unwrap();
        assert_eq!(prov2.continuation, "approximate_hot");
        assert_eq!(prov2.from_schema, 2);
    }

    #[test]
    fn native_snapshot_has_no_migration_provenance() {
        let inp = idle_point(healthy_inputs());
        let w = initial_state(&inp).unwrap();
        assert!(w.snapshot().provenance.is_none());
    }

    // ---- R02-C1/RV05: atomicidad de frontera engine ----------------------

    #[test]
    fn resume_error_leaves_state_untouched() {
        let inp = idle_point(healthy_inputs());
        let mut w = initial_state(&inp).unwrap();
        w.m_in_amb_kg = 0.12; // marcador de ledger preexistente
        let before = format!("{w:?}");
        let mut bad = inp.clone();
        bad.tailpipe_area_m2 = -1.0;
        assert!(resume(&mut w, &bad, 0.001).is_err());
        assert_eq!(before, format!("{w:?}"));
        // Estado imposible: rechazo antes de mutar.
        let mut w2 = initial_state(&inp).unwrap();
        w2.cyls[0].m = -1.0;
        let before2 = format!("{w2:?}");
        assert!(resume(&mut w2, &inp, 0.001).is_err());
        assert_eq!(before2, format!("{w2:?}"));
        // Fricción no finita: error, no torque cero silencioso.
        let mut w3 = initial_state(&inp).unwrap();
        let before3 = format!("{w3:?}");
        let mut bad3 = inp.clone();
        bad3.friction = crank::FrictionModel::Viscous { c_n_m_s: f64::NAN };
        assert!(resume(&mut w3, &bad3, 0.001).is_err());
        assert_eq!(before3, format!("{w3:?}"));
    }

    #[test]
    fn restore_rejects_invalid_history_counters_and_thermal() {
        let inp = idle_point(healthy_inputs());
        let base = initial_state(&inp).unwrap().snapshot().state;
        let mut s = base.clone();
        s.prev_local_deg[0] = f64::NAN;
        assert!(WorkState::restore(&EngineSnapshot::capture(&s), &inp).is_err());
        let mut s = base.clone();
        s.fuel_emitted_kg = -1.0;
        assert!(WorkState::restore(&EngineSnapshot::capture(&s), &inp).is_err());
        let mut s = base.clone();
        s.heat_released_j = f64::NAN;
        assert!(WorkState::restore(&EngineSnapshot::capture(&s), &inp).is_err());
        let mut tinp = idle_point(healthy_inputs());
        tinp.thermal = Some(thermal::ThermalParams::illustrative());
        let mut s = base.clone();
        s.thermal = Some(crate::state::ThermalSnap {
            t_wall_c: -300.0,
            t_piston_c: 20.0,
            t_oil_c: 20.0,
            t_cool_c: 20.0,
            ect_filt_c: 20.0,
        });
        assert!(WorkState::restore(&EngineSnapshot::capture(&s), &tinp).is_err());
        let mut bad = idle_point(healthy_inputs());
        bad.plenum_v_m3 = 0.0;
        assert!(WorkState::restore(&EngineSnapshot::capture(&base), &bad).is_err());
    }
}

#[cfg(test)]
mod r02_c2_tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn idle_point(mut inp: EngineInputs) -> EngineInputs {
        inp.throttle_area_m2 = 0.05e-4;
        inp.idle_bypass_area_m2 = 0.02e-4;
        inp.t_end_s = 4.0;
        inp
    }

    /// Un input inválido debe rechazarse en run/resume/restore sin mutar el
    /// estado (matriz de fronteras R02-C2/C1-01).
    fn assert_rejected(inp: &EngineInputs) {
        assert!(run_engine(inp).is_err(), "run debe rechazar");
        let mut w = initial_state(&idle_point(healthy_inputs())).unwrap();
        w.m_in_amb_kg = 0.12;
        let before = format!("{w:?}");
        assert!(resume(&mut w, inp, 0.001).is_err(), "resume debe rechazar");
        assert_eq!(before, format!("{w:?}"), "estado debe quedar intacto");
        let snap = initial_state(&idle_point(healthy_inputs()))
            .unwrap()
            .snapshot();
        assert!(
            WorkState::restore(&snap, inp).is_err(),
            "restore debe rechazar"
        );
    }

    #[test]
    fn invalid_input_matrix_rejects_atomically() {
        let base = idle_point(healthy_inputs());
        // Control open-loop.
        let mut i = base.clone();
        i.control.afr_target = f64::NAN;
        assert_rejected(&i);
        let mut i = base.clone();
        i.control.afr_target = 0.0;
        assert_rejected(&i);
        let mut i = base.clone();
        i.control.eta_v_assumed = f64::NAN;
        assert_rejected(&i);
        let mut i = base.clone();
        i.control.spark_adv_btdc_deg = f64::INFINITY;
        assert_rejected(&i);
        // Combustible.
        let mut i = base.clone();
        i.fuel.lhv_j_per_kg = f64::NAN;
        assert_rejected(&i);
        let mut i = base.clone();
        i.fuel.density_kg_m3 = -1.0;
        assert_rejected(&i);
        let mut i = base.clone();
        i.fuel.stoich_afr = 0.0;
        assert_rejected(&i);
        // Eléctrico / arranque.
        let mut i = base.clone();
        i.charge_volts = f64::NAN;
        assert_rejected(&i);
        let mut i = base.clone();
        i.dwell_s = -1.0;
        assert_rejected(&i);
        let mut i = base.clone();
        i.starter_cutout_rpm = f64::NAN;
        assert_rejected(&i);
        let mut i = base.clone();
        i.starter.as_mut().unwrap().0.stall_torque_n_m = f64::NAN;
        assert_rejected(&i);
        // Inercia.
        let mut i = base.clone();
        i.inertia.rotating_j_kg_m2 = f64::NAN;
        assert_rejected(&i);
        let mut i = base.clone();
        i.inertia.rotating_j_kg_m2 = 0.0;
        i.inertia.recip_mass_kg = 0.0;
        assert_rejected(&i);
        // Wiebe.
        let mut i = base.clone();
        i.wiebe_duration_deg = f64::INFINITY;
        assert_rejected(&i);
        // Áreas.
        let mut i = base.clone();
        i.tailpipe_area_m2 = -1.0;
        assert_rejected(&i);
        // Térmico.
        let mut i = base.clone();
        let mut p = thermal::ThermalParams::illustrative();
        p.c_wall = 0.0;
        i.thermal = Some(p);
        assert_rejected(&i);
        let mut i = base.clone();
        let mut p = thermal::ThermalParams::illustrative();
        p.wall_frac = f64::NAN;
        i.thermal = Some(p);
        assert_rejected(&i);
        let mut i = base.clone();
        let mut p = thermal::ThermalParams::illustrative();
        p.c_wall = f64::MAX;
        i.thermal = Some(p);
        assert_rejected(&i);
    }

    #[test]
    fn valid_inputs_remain_accepted() {
        let trace = run_engine(&idle_point(healthy_inputs())).unwrap();
        assert!(trace.audit.energy_closure.is_finite());
        assert!(trace.audit.mass_closure.is_finite());
        assert!(trace.audit.final_rpm.is_finite());
    }

    #[test]
    fn noop_resume_validates_state_and_thermal_coherence() {
        let inp = idle_point(healthy_inputs());
        // Nodo con volumen/cv inválidos.
        let mut w = initial_state(&inp).unwrap();
        w.plen.v = 0.0;
        let before = format!("{w:?}");
        let t = w.time_s;
        assert!(resume(&mut w, &inp, t).is_err());
        assert_eq!(before, format!("{w:?}"));
        let mut w = initial_state(&inp).unwrap();
        w.plen.cv = f64::NAN;
        let before = format!("{w:?}");
        let t = w.time_s;
        assert!(resume(&mut w, &inp, t).is_err());
        assert_eq!(before, format!("{w:?}"));
        // Incoherencia térmica: inputs con red, estado sin red.
        let mut tinp = idle_point(healthy_inputs());
        tinp.thermal = Some(thermal::ThermalParams::illustrative());
        let mut w = initial_state(&inp).unwrap();
        let before = format!("{w:?}");
        let t = w.time_s;
        assert!(resume(&mut w, &tinp, t).is_err());
        assert_eq!(before, format!("{w:?}"));
        // Red presente con energía no representable.
        let mut p = thermal::ThermalParams::illustrative();
        p.c_wall = f64::MAX;
        let mut w = initial_state(&inp).unwrap();
        w.therm = Some((thermal::ThermalState::cold(20.0), p));
        w.th_u0_j = 20.0 * p.c_wall;
        let mut i2 = idle_point(healthy_inputs());
        i2.thermal = Some(p);
        let before = format!("{w:?}");
        let t = w.time_s;
        assert!(resume(&mut w, &i2, t).is_err());
        assert_eq!(before, format!("{w:?}"));
    }

    #[test]
    fn pre_c2_snapshot_is_not_declared_exact_native() {
        // Un snapshot estructuralmente completo sin marcador de modelo
        // (R02/C1) se migra como aproximado, no como nativo exacto (C1-05).
        let inp = idle_point(healthy_inputs());
        let mut w = initial_state(&inp).unwrap();
        drive(&mut w, &inp, 0.4, inp.record_every, 0).unwrap();
        let mut snap = w.snapshot();
        snap.model = None; // productor pre-C2
        assert!(snap.verified());
        let restored = WorkState::restore(&snap, &inp).unwrap();
        let prov = restored.snapshot().provenance.unwrap();
        assert_eq!(prov.continuation, "approximate_hot");
        assert!(prov
            .missing_fields
            .iter()
            .any(|f| f == "combustion_accounting"));
        // Un nativo C2 no lleva procedencia y continúa exacto.
        let native = w.snapshot();
        assert!(native.model.is_some());
        let restored2 = WorkState::restore(&native, &inp).unwrap();
        assert!(restored2.snapshot().provenance.is_none());
    }

    #[test]
    fn unknown_or_mismatched_model_is_rejected() {
        let inp = idle_point(healthy_inputs());
        let w = initial_state(&inp).unwrap();
        let snap = w.snapshot();
        let mut bad = snap.clone();
        bad.model.as_mut().unwrap().combustion_accounting = 999;
        assert!(WorkState::restore(&bad, &inp).is_err());
        let mut mism = snap.clone();
        mism.model.as_mut().unwrap().schema_version = 1;
        assert!(WorkState::restore(&mism, &inp).is_err());
        // Convención conocida: acepta.
        assert!(WorkState::restore(&snap, &inp).is_ok());
    }
}
