//! Cinemática biela-manivela (S06): posiciones exactas en el modelo
//! geométrico declarado. Sin combustión, sin RPM simuladas: la velocidad
//! angular es un parámetro de evaluación, no un estado integrado.
//!
//! Convenciones (ADR-014): `theta = 0` en PMS; `x` desplazamiento desde PMS;
//! ciclo 720° con PMS de encendido en local 360°; orden 1-3-4-2.

pub mod geometry;
pub mod phases;

pub use geometry::{GeometryPack, IllustrativeSource, KinematicsError, SliderState};
pub use phases::{cylinder_local_angle, CylinderId, STROKES};
