//! Rutas de fuga generales (S23): dominios de fluido + conductancias con
//! ley EXPLÍCITA y evidencia declarada. Sin multiplicadores de potencia,
//! sin rutinas de síntomas: los flujos cambian estados y mediciones.
//!
//! - Anillo: área = gap × profundidad radial (geométrica L1, sin dinámica
//!   de anillo: deuda declarada).
//! - Junta: área = A_seg·max(0, 1 − p_contacto/p_ref)² con p_ref declarado
//!   NO calibrado (S25 lo medirá contra leak-down vs torque). La FORMA es
//!   explícita y la sensibilidad a p_ref se reporta, no se ajusta.

pub mod conductance;
pub mod network;
pub mod scenarios;

pub use conductance::{annulus_leak_area, gasket_leak_area, SealContactState};
pub use network::{run_leakdown, Fluid, LeakAudit, LeakDomain, LeakNetwork, LeakPort};
pub use scenarios::{leak_scenario, pseal_sensitivity, LeakScenario};
