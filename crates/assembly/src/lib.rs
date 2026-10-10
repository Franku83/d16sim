//! Grafo de montaje (S04): definiciones, instancias, joints y puertos.
//!
//! - `PartDefinition`: diseño nominal inmutable (sin dimensiones inventadas:
//!   `geometry_ref`/`material_ref` son referencias, no valores).
//! - `PartInstance`: pieza concreta con serial persistente e historial.
//! - `Joint`/`Port`: uniones entre instancias por tipo de conexión.
//! - Operaciones atómicas `install/remove/connect/disconnect` con resultado
//!   tri-estado: aplicada, aplicada con riesgo (permitida pero posiblemente
//!   destructiva — el daño lo decide el modelo material futuro, no este
//!   crate), o rechazada por imposibilidad estructural.
//! - Persistencia versionada con fail-closed ante esquema desconocido.
//! - Sin física: este crate no simula comportamiento, solo estructura.

pub mod d16y7;
pub mod graph;
pub mod ops;
pub mod types;

pub use graph::EngineGraph;
pub use ops::{apply_operation, Operation, OperationResult};
pub use types::{
    AssemblyError, DataConfidence, InterfaceSpec, Joint, PartDefinition, PartInstance, Port,
    PortKind,
};

/// Versión actual del esquema de `EngineGraph`. Subirla exige migración
/// explícita o rechazo fail-closed en `load` (nunca lectura silenciosa).
pub const SCHEMA_VERSION: u32 = 1;
