//! Tipos del grafo de montaje.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error estructural (nunca un daño físico: eso lo decide el modelo
/// material futuro con criterio y parámetros).
#[derive(Debug, Clone, PartialEq, Error)]
pub enum AssemblyError {
    /// La operación es geométricamente/estructuralmente imposible.
    #[error("impossible operation: {reason}")]
    Impossible { reason: String },

    /// Esquema persistido desconocido o migración ausente: fail-closed.
    #[error("unsupported schema version {found} (current {current})")]
    UnsupportedSchema { found: u32, current: u32 },

    /// JSON inválido o E/S.
    #[error("graph store error: {0}")]
    Store(String),
}

/// Confianza del dato de una definición (S01: nada es ExactOem).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataConfidence {
    /// Identidad de catálogo (EPC): existe y es compatible; sin dinámica.
    CatalogIdentity,
    /// Candidato pendiente de auditoría OEM.
    PendingVerification,
}

/// Clase de conexión física de un puerto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortKind {
    /// Encaje mecánico / asiento.
    MechanicalMount,
    /// Transmisión de torque.
    TorqueTransmit,
    /// Contacto sin unión (p. ej. corona frente a cámara).
    Contact,
    /// Flujo de aceite.
    OilFlow,
    /// Flujo de refrigerante.
    CoolantFlow,
    /// Flujo de aire/mezcla.
    AirFlow,
    /// Señal eléctrica.
    ElectricalSignal,
}

/// Especificación de una interfaz de montaje en una definición.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterfaceSpec {
    pub name: String,
    pub kind: PortKind,
    /// Otra definición con la que es compatible (`None` = cualquiera del
    /// mismo `PortKind`; la compatibilidad fina llega con geometría S07).
    pub mates_with_definition: Option<String>,
}

/// Diseño nominal inmutable de una pieza.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartDefinition {
    pub id: String,
    pub label: String,
    /// Referencia a geometría (ruta/documento), no valores.
    pub geometry_ref: String,
    /// Referencia a material (ruta/documento), no propiedades.
    pub material_ref: String,
    pub mounting_interfaces: Vec<InterfaceSpec>,
    pub data_confidence: DataConfidence,
    pub source_refs: Vec<String>,
}

/// Pieza concreta: serial persistente, definición y condición general.
/// El desgaste específico por mecanismo vive en S26; aquí solo ciclos e
/// historial de operaciones.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartInstance {
    pub serial: String,
    pub definition_id: String,
    pub cycles: u64,
    pub history: Vec<String>,
}

/// Puerto instanciado: punto de conexión tipado de una instancia.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Port {
    pub id: String,
    pub instance_serial: String,
    pub kind: PortKind,
}

/// Unión entre instancias a través de puertos compatibles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Joint {
    pub id: String,
    pub ports: Vec<String>,
    /// Historial de apriete/medición (el modelo de precarga es S08; aquí
    /// solo se conserva la traza).
    pub install_history: Vec<String>,
}
