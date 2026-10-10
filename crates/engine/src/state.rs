//! Estado del motor integrado (S16): cigüeñal + 4 cámaras + nodos +
//! combustible, serializable para snapshot/replay.

use serde::{Deserialize, Serialize};

/// Estado de una cámara (sistema abierto homogéneo, dos especies vía fresca).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CylState {
    pub m_kg: f64,
    pub u_j: f64,
    pub fresh_frac: f64,
    /// Combustible presente en la cámara (kg, vapor L1).
    pub fuel_kg: f64,
    /// Combustible fijado en la chispa del ciclo en curso (kg).
    pub burn_latched_kg: f64,
    /// ¿Quemado contabilizado ya en este ciclo?
    pub burn_accounted: bool,
    // Reloj de eventos por flanco vive en WorkState.prev_local_deg
    // (ver `crossed_edge` en run), no por cilindro.
}

/// Estado completo e inmutable del motor (snapshot).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EngineState {
    pub schema_version: u32,
    pub time_s: f64,
    pub theta_global_rad: f64,
    pub omega_rad_s: f64,
    /// Locales previos por cilindro (reloj de flancos `crossed_edge`).
    pub prev_local_deg: [f64; 4],
    pub cyls: [CylState; 4],
    pub plen_m_kg: f64,
    pub plen_t_k: f64,
    pub plen_fresh: f64,
    pub exh_m_kg: f64,
    pub exh_t_k: f64,
    pub exh_fresh: f64,
    pub fuel_injected_kg: f64,
    pub fuel_burned_kg: f64,
    pub heat_released_j: f64,
    pub work_ind_j: f64,
    pub work_fric_j: f64,
    pub work_starter_j: f64,
    pub work_load_j: f64,
}

/// Snapshot con hash canónico (excluye UI: no hay).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EngineSnapshot {
    pub state: EngineState,
    pub state_hash: String,
}

impl EngineSnapshot {
    /// Captura con hash (debe coincidir con el productor del estado).
    #[must_use]
    pub fn capture(state: &EngineState) -> Self {
        Self {
            state: state.clone(),
            state_hash: engine_types::sha256_hex(engine_types::canonical_json(state).as_bytes()),
        }
    }

    /// Verifica el hash contra el contenido.
    #[must_use]
    pub fn verified(&self) -> bool {
        engine_types::sha256_hex(engine_types::canonical_json(&self.state).as_bytes())
            == self.state_hash
    }
}
