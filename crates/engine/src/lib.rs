//! Motor integrado (S16): cuatro cámaras independientes acopladas al mismo
//! cigüeñal, con aire (S12), combustible (S13), combustión (S10), dinámica
//! (S14) y arranque (S15) acoplados causalmente por paso temporal.
//!
//! Sin clamp de RPM, sin torque mágico, sin bootstrap: el ralentí (si lo
//! hay) emerge del balance de torques. Controlador open-loop declarado
//! (mapa ηv fijo + AFR objetivo + avance fijo): lazo cerrado y mapas
//! reales son S18/S25. AFR→quemado sin acoplar (limitación S16).

pub mod control;
pub mod run;
pub mod state;

pub use control::{FuelCommand, OpenLoopControl};
pub use run::{
    healthy_inputs, high_friction_inputs, no_fuel_inputs, no_spark_inputs, resume, run_engine,
    EngineInputs, RunAudit, RunTrace, ThermalSummary, TraceSample, WorkState,
};
pub use state::{CylState, EngineSnapshot, EngineState};
