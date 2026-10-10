//! Combustión reducida (S10): fracción quemada de Wiebe acoplada a la
//! primera ley de zona única, sin doble conteo, con Q_pared = 0.
//!
//! Parámetros `a/m/inicio/duración` explícitos, no universales. Entradas de
//! combustible ilustrativas (no medidas). Sin calibración a hp de catálogo.

pub mod burn;
pub mod wiebe;

pub use burn::{BurnAudit, BurnCycle, BurnParams, WallHeat};
pub use wiebe::Wiebe;
