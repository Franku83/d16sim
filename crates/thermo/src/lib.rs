//! Cilindro sellado ideal sin combustión (S09): primera ley con geometría
//! variable, Q = 0, RK4 por ángulo contra referencia adiabática analítica.
//!
//! Sin mezcla real (R/γ explícitos), sin fugas, sin pared térmica: la
//! auditoría energética y la convergencia son el entregable, no una
//! predicción de motor.

pub mod closed_cycle;
pub mod mixture;
pub mod state;

pub use closed_cycle::{Audit, ClosedCycle, CyclePoint};
pub use mixture::Mixture;
pub use state::ThermoState;
