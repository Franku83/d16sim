//! Red 0D de gas (S12): nodos, orificios compresibles, ciclo con gas
//! exchange. Sin VE impuesta: el llenado emerge del caudal calculado.
//!
//! Faltantes 1D declarados (ondas, pérdidas de puerto distribuidas,
//! inercia de columna): `docs/physics/gas_exchange.md`.

pub mod cycle;
pub mod data;
pub mod nodes;
pub mod orifice;

pub use cycle::{run_cycle, CycleAudit, ExhaustInit, GasExchangeInputs, TracePoint};
pub use data::{exhaust_cd, intake_cd, EXHAUST_VALVE_DIA_MM, INTAKE_VALVE_DIA_MM};
pub use nodes::Volume;
pub use orifice::{mass_flow, FlowSide};
