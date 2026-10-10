//! Tipos dimensionales SI del núcleo (S03).
//!
//! Reglas:
//! - Todo valor interno es SI (`f64`) con campo privado: solo se construye
//!   mediante constructores que validan (finito, rango físico).
//! - Tipos distintos no interoperan: sumar `Torque` a `Temperature` o pasar
//!   `Rpm` donde se pide `AngularVelocity` **no compila** (sin `impl`
//!   cruzados a propósito).
//! - Presión absoluta, manométrica (gauge) y diferencial son tres tipos:
//!   convertir entre ellos exige el ambiente explícito.
//! - Las conversiones de visualización (psi, °C, lb·ft, hp) viven en
//!   [`display`] y nunca entran al core como tipos.

pub mod display;
pub mod float;
pub mod fuel;
pub mod mechanics;
pub mod pressure;
pub mod thermal;

pub use float::UnitsError;
