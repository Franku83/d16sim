//! Desgaste S26: slow-state persistente con leyes explícitas y provenance.
//! Regla: ley solo con parámetros defendibles; lo demás es `UNSUPPORTED`
//! (sin números silenciosos). Sin evidencia, salidas `TREND_CONDITIONED`:
//! tendencias, jamás vida útil exacta ni instantes de fallo.

pub mod laws;
pub mod service;
pub mod state;

pub use laws::{Support, WearLaw};
pub use service::{ServiceEvent, ServiceLog};
pub use state::{OperatingPoint, WearState};
