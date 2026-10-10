//! Calibración S25: parámetros identificables, manifiesto train/holdout,
//! ajuste con covarianzas y sensibilidad. Cierre honesto:
//! **NUMERICALLY_VERIFIED_ONLY** — sin dataset de campo no hay validación.
//!
//! Regla: `DataSource::Real` exige procedencia de banco (fail-closed);
//! los sintéticos solo verifican la maquinaria (self-consistency) y van
//! rotulados `SYNTHETIC_SELFTEST`, jamás como validación.

pub mod dataset;
pub mod fit;
pub mod params;
pub mod sensitivity;

pub use dataset::{DataSource, Manifest, Observation, Trial};
pub use fit::{least_squares, FitResult};
pub use params::{calibration_set, CalParam};
pub use sensitivity::{local_elasticities, morris_effects};
