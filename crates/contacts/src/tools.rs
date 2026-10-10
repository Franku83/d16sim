//! Herramientas e instrumentos (tipos S07, sin UI).

use serde::{Deserialize, Serialize};

/// Clase de herramienta/operación.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolKind {
    HandTorqueWrench,
    BoreGauge,
    Micrometer,
    DialIndicator,
    AirSupply,
}

/// Estado de calibración declarado (sin certificados inventados).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Calibration {
    Uncalibrated,
    CheckedAgainstReference { reference_id: String },
}

/// Herramienta identificada: aplica magnitudes, no cambia daño directamente.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tool {
    pub id: String,
    pub kind: ToolKind,
    pub calibration: Calibration,
}
