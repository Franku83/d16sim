//! Pernos, precarga y junta L1 (S08): torque causal sin penalizadores.
//!
//! - Precarga por `T = K·F·d` con `K` explícito e incertidumbre: el mismo
//!   torque con distinto lubricante da distinta precarga (gate).
//! - El modelo L1 no tiene memoria de trayectoria: la secuencia queda
//!   registrada pero no finge distorsión (sin mecanismo capaz).
//! - La junta calcula campo de presión por segmentos; el sellado real y la
//!   fuga llegan en S23 vía [`SealConductance`] (costura futura).
//! - Calibración pendiente declarada: ningún número aquí es OEM Honda.

pub mod gasket;
pub mod preload;

pub use gasket::{distribute_clamp, GasketField, SealConductance, SealSegment, SealStatus};
pub use preload::{
    illustrative_k_range, preload_of_step, LubricantCondition, PreloadEstimate, TorqueStep,
    YieldCheck,
};
