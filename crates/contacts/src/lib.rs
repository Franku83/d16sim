//! Contactos geométricos y herramientas (S07): prototipo de detección de
//! interferencias con geometría simplificada. Sin modelo material: todo
//! contacto reporta `needs_material_model = true` y jamás un daño.
//!
//! Todas las dimensiones son entradas del llamante (SI); este crate no
//! contiene cotas OEM. La posición del pistón viene de `kinematics`; la
//! alzada de válvula es un valor impuesto (el perfil real es S11).

pub mod interference;
pub mod measurement;
pub mod tools;

pub use interference::{
    check_piston_valve, check_rod_block, thread_fit, ContactReport, PistonValveInput,
};
pub use measurement::{correct_reading, Measurement};
pub use tools::{Calibration, Tool, ToolKind};
