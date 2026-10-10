//! Arranque (S15): batería + starter + chispa disponible. Sin bootstrap:
//! el arranque con combustión integrada es S16; aquí el motor gira
//! arrastrado sin arrancar jamás (`started = false` por diseño).

pub mod battery;
pub mod crank;
pub mod ignition;
pub mod sensors;
pub mod starter;
pub mod wiring;

pub use battery::Battery;
pub use crank::{cold_crank, ColdCrankInputs, ColdCrankResult};
pub use ignition::{Coil, SparkEvent};
pub use sensors::{load_from_map, warmup_enrich_from_ect, SensorChannel, Transfer};
pub use starter::{StarterDrive, StarterMotor};
pub use wiring::{RangeEvent, RangeMonitor, Wire, WireFault};
