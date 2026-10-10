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

/// Estado completo e inmutable del motor (snapshot, esquema 2).
///
/// Compatibilidad (R02-F05/F06): los campos añadidos en la remediación R01
/// (inventario de combustible en nodos y frontera emitida, historia de
/// presión para fricción) son opcionales y se omiten de la representación
/// canónica cuando valen su valor neutro. Así los snapshots v1/v2 previos
/// conservan su hash histórico y los nuevos serializan el estado físico
/// completo. El esquema nativo sigue siendo 2.
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
    /// Red térmica S21 (None = instantáneas S16; `default` migra v1).
    #[serde(default)]
    pub thermal: Option<ThermalSnap>,
    /// Combustible (kg) presente en el plenum (R01-F01). 0 = campo ausente
    /// en snapshots previos; se omite de la forma canónica cuando es 0.
    #[serde(default, skip_serializing_if = "is_zero_f64")]
    pub plen_fuel_kg: f64,
    /// Combustible (kg) presente en el escape (R01-F01).
    #[serde(default, skip_serializing_if = "is_zero_f64")]
    pub exh_fuel_kg: f64,
    /// Combustible (kg) cruzado a la frontera ambiente (R01-F01).
    #[serde(default, skip_serializing_if = "is_zero_f64")]
    pub fuel_emitted_kg: f64,
    /// Máximo de presión de cilindro (Pa) usado por la fricción (R01-F07);
    /// `None` = snapshot previo sin historia de presión.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pmax_pa: Option<f64>,
}

/// Auxiliar de `skip_serializing_if` para campos numéricos opcionales.
#[must_use]
fn is_zero_f64(v: &f64) -> bool {
    *v == 0.0
}

/// Red térmica serializada (S21).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ThermalSnap {
    pub t_wall_c: f64,
    pub t_piston_c: f64,
    pub t_oil_c: f64,
    pub t_cool_c: f64,
    pub ect_filt_c: f64,
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
            state_hash: canonical_state_hash(state),
        }
    }

    /// Verifica el hash contra el contenido, bajo el esquema declarado.
    #[must_use]
    pub fn verified(&self) -> bool {
        canonical_state_hash(&self.state) == self.state_hash
    }
}

/// Hash canónico del estado según su esquema.
///
/// v1 histórico (anterior a S21) no tenía campo `thermal`: se reconstruye la
/// representación original ANTES de migrar (R02-F06). v2 y nativo usan la
/// serialización canónica completa; los campos nuevos de R01 se omiten cuando
/// valen su valor neutro, preservando hashes de snapshots previos.
fn canonical_state_hash(state: &EngineState) -> String {
    let text = engine_types::canonical_json(state);
    let text = if state.schema_version == 1 {
        text.replace(",\"thermal\":null", "")
    } else {
        text
    };
    engine_types::sha256_hex(text.as_bytes())
}
