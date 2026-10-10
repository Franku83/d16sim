//! Lubricación L1 (S22): aceite, bomba, galerías, cojinetes y película.
//!
//! Ruta causal (gate): holgura/bomba/viscosidad ⇒ caudal por rama ⇒
//! presión local ⇒ calidad de película ⇒ multiplicador de fricción.
//! Nunca una bandera `durability = 0`. La película alimenta al futuro
//! módulo de desgaste (S26) como parámetro físico, no como veredicto.

pub mod bearing;
pub mod gallery;
pub mod oil;
pub mod pump;

pub use bearing::{film_quality, film_quality_supplied, BearingGeom, FrictionLink};
pub use gallery::{solve_gallery, BranchResult, GalleryResult};
pub use oil::{OilGrade, Walther};
pub use pump::{Pump, PumpState as LubePumpState};
