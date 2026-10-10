//! Auditoría S30: 4 cadenas causales, perfiles numéricos, licencias y
//! benchmark. Todo verificado aquí es NUMERICALLY_VERIFIED; nada declara
//! validación experimental (gate S30).

pub mod chains;
pub mod env;
pub mod profiles;

pub use chains::{audit_chains, ChainReport};
pub use env::{audit_env, Benchmark, EnvReport};
pub use profiles::{audit_profiles, ProfileRow};
