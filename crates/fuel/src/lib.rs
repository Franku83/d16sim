//! Suministro de combustible L1 (S13): depósito/rail/bomba/inyector.
//!
//! - Comandado vs entregado son tipos distintos: la ECU comandará pulsos;
//!   lo entregado depende de ΔP real, deadtime, tensión y capacidad.
//! - Sin DTC en este crate (gate S13): la falta de presión es un ESTADO
//!   (`DeliveryStatus::NoPressure`), no un código.
//! - Procedencia: identidad P2A-000 EPC; Q 185 cc/min @3 bar = HIPÓTESIS
//!   (claim 180–190 sin ΔP); ley Q∝√ΔP ±2% [D]; deadtime analógico
//!   CBR600RR ±0.3 ms (NO es el P2A-000); rail OEM-TEXT candidato.

pub mod injector;
pub mod rail;
pub mod schedule;

pub use injector::{DeliveredMass, Injector, InjectorError, PulseCommand};
pub use rail::{resolve_rail, Fuel, PumpState, RailState, Regulator};
pub use schedule::{InjectionEvent, InjectionStrategy};
