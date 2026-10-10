//! Distribución SOHC (S11): un árbol y un cigüeñal compartidos, cuatro
//! cilindros derivados. Sin daños automáticos; el perfil Y7 stock es
//! MISSING-DATA y el proxy A6 va etiquetado como tal.

pub mod belt;
pub mod cam;
pub mod events;

pub use belt::BeltDrive;
pub use cam::{CamSpec, ValveEvents};
pub use events::{cylinder_valve_events, CylinderValveEvents};
