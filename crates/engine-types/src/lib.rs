//! Tipos canónicos del núcleo: IDs, errores, snapshots y hashing.
//!
//! S02: sin física. Este crate solo define el vocabulario y el error
//! tipado que los futuros solvers deberán retornar.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Identificador opaco de un motor instanciado.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EngineId(pub String);

/// Versión del paquete de modelos físicos (p. ej. `physics-pack-v0.1`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelPackVersion(pub String);

/// Error estructurado de todo el núcleo. Un estado inválido produce
/// `SimulationError` trazable, nunca una avería ficticia (Foundation §3.3).
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    /// Funcionalidad física aún no implementada (S02: todos los solvers).
    #[error("physics not implemented: {feature} (sprint {needed_by_sprint})")]
    PhysicsNotImplemented {
        feature: &'static str,
        needed_by_sprint: &'static str,
    },

    /// Manifiesto o dataset con esquema inválido.
    #[error("schema validation failed: {0}")]
    SchemaValidation(String),

    /// Hash de replay distinto al registrado.
    #[error("replay mismatch: expected {expected}, got {actual}")]
    ReplayMismatch { expected: String, actual: String },

    /// E/S.
    #[error("io error at {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// JSON.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Snapshot versionado e inmutable del estado del motor (contenido opaco
/// en S02; la cinemática/física llegan en S04–S09).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineSnapshot {
    pub schema_version: u32,
    pub engine_id: EngineId,
    pub model_pack: ModelPackVersion,
    pub payload: serde_json::Value,
}

impl EngineSnapshot {
    /// Hash canónico del snapshot (excluye campos de UI; aquí no hay).
    #[must_use]
    pub fn canonical_hash(&self) -> String {
        sha256_hex(canonical_json(self).as_bytes())
    }
}

/// Serialización canónica: orden de campos de struct + claves de mapa
/// ordenadas (`serde_json::Map` es `BTreeMap` sin `preserve_order`).
/// Cambiar el serializador invalida todos los hashes.
#[must_use]
pub fn canonical_json<T: Serialize>(value: &T) -> String {
    match serde_json::to_string(value) {
        Ok(s) => s,
        Err(e) => {
            // serde_json::to_string sobre tipos Serialize construidos en
            // memoria no falla en la práctica; si fallara, el hash debe
            // reflejar el fallo de forma determinista en vez de pánico.
            format!(
                "{{\"serialization_error\":{}}}",
                serde_json::json!(e.to_string())
            )
        }
    }
}

/// SHA-256 en hexadecimal minúscula.
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut out = String::with_capacity(64);
    for b in digest {
        // `write!` sobre String no falla; el lint `unwrap_used` prohíbe
        // `unwrap()` así que se propaga el `fmt::Result` con `let`.
        let _ = std::fmt::Write::write_fmt(&mut out, format_args!("{b:02x}"));
    }
    out
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn canonical_json_is_stable_across_key_insertion_order() {
        let a = serde_json::json!({"z": 1, "a": {"y": 2, "b": 3}});
        let b = serde_json::json!({"a": {"b": 3, "y": 2}, "z": 1});
        assert_eq!(canonical_json(&a), canonical_json(&b));
        assert_eq!(sha256_hex(b"abc"), sha256_hex(b"abc"));
    }

    #[test]
    fn snapshot_hash_changes_when_payload_changes() {
        // Prueba que falla sin el mecanismo: sin hashing canónico no hay
        // forma de detectar divergencia de estado (gate S05 futuro).
        let base = EngineSnapshot {
            schema_version: 1,
            engine_id: EngineId("bench-001".to_owned()),
            model_pack: ModelPackVersion("physics-pack-v0.0-none".to_owned()),
            payload: serde_json::json!({"step": 1}),
        };
        let mut altered = base.clone();
        altered.payload = serde_json::json!({"step": 2});
        assert_ne!(base.canonical_hash(), altered.canonical_hash());
    }

    #[test]
    fn error_is_structured_not_a_panic() {
        let e = EngineError::PhysicsNotImplemented {
            feature: "closed-cylinder solver",
            needed_by_sprint: "S09",
        };
        assert!(e.to_string().contains("S09"));
    }
}
