//! Daño mecánico S27: mecanismos con dominio de validez y criterio físico.
//! Distinción estricta: contacto geométrico ≠ daño; riesgo ≠ rotura.
//! Ruptura solo con material+geometría+calibración declarados (aquí: solo
//! fluencia de perno con material GENÉRICO ISO 898-1, nunca D16Y7).

pub mod mechanism;

pub use mechanism::{
    bearing_starvation_risk, bolt_yield_state, piston_valve_state, thread_state, BoltState,
    ContactOutcome, DamageError, DamageMechanism, MechanismState, Support as MechSupport,
};
