//! Suite adversarial S29: fallos como DELTAS FÍSICOS sobre entradas
//! (nunca flags que gobiernen la física; no existe `fault_count` en este
//! crate). Pares combinados, invariantes, no-linealidad, observabilidad,
//! conmutatividad y Monte Carlo con seed fija.

pub mod faults;
pub mod suite;

pub use faults::Fault;
pub use suite::{run_case, run_pair, CaseResult, Discrepancy, MonteCarlo, PairResult};
