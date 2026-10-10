//! Dinámica de cigüeñal (S14): `J·dω/dt = Στ` sin energía gratis.
//!
//! Contabilidad (Foundation §6.2, sin doble conteo): el torque indicado
//! integra el ciclo COMPLETO `∮p·dV` (ya incluye bombeo); la fricción y la
//! carga van por separado. Quien use solo circuito cerrado debe sumar el
//! bombeo explícito (documentado, no implementado dos veces).

pub mod friction;
pub mod inertia;
pub mod integrate;
pub mod torque;

pub use friction::{ChenFlynnCoeffs, FrictionModel};
pub use inertia::{effective_inertia_mean, InertiaParams};
pub use integrate::{spin_down, DynoLaw, SpinResult, SpinState};
pub use torque::IndicatedTorque;
