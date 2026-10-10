//! Red térmica localizada L1 (S21): metal, pistón, aceite, refrigerante.
//!
//! Todo calor tiene fuente/sumidero declarados. Parámetros ilustrativos de
//! orden de magnitud (masas×cp, hA), NO medidos en el D16Y7. La fricción
//! calienta el aceite (antes sumidero no modelado); el quemado reparte una
//! fracción a paredes (el resto queda en el gas, como en S10/S16).

pub mod network;

pub use network::{ThermalParams, ThermalState};
