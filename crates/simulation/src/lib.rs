//! Orquestación headless — S02: manifiesto, validación de datos,
//! runner de plumbing determinista, replay y reporte.
//!
//! Sin física: las trazas son un autocontador determinista derivado de la
//! seed (`runner_plumbing_only = true`). Ningún número aquí representa un
//! motor; solo demuestra I/O determinista + hashing para los gates S02/S05.

pub mod history;
pub mod manifest;
pub mod runner;
pub mod validation;

pub use history::{generate_sequence, replay_history, HistoryEvent, ReplayOutcome, SnapshotFile};
pub use manifest::RunManifest;
pub use runner::{replay, report_summary, run_scenario, RunSummary};
pub use validation::validate_data_dir;
