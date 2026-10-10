//! Dyno S28: freno virtual auditable sobre el motor integrado.
//! Regla: la potencia NUNCA entra como dato — sale de τ integrado:
//! indicada, fricción y freno se reportan separadas, con corrección
//! ambiente explícita (observada + corregida, nunca solo corregida) y
//! validez de mapa (cubierto vs extrapolado).

pub mod brake;
pub mod sweep;

pub use brake::{Ambient, Correction, DynoConfig};
pub use sweep::{run_sweep, Manifest, SweepPoint, SweepResult, Validity};
