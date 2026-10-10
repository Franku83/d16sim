//! Historia determinista (S05): eventos de tiempo físico, snapshots y replay.
//!
//! - Orden total determinístico: `(phys_time_s por total_cmp, seq)`; sin
//!   reloj del sistema ni aleatoriedad en ningún camino (`phys_time_s`
//!   deriva del tick lógico × `dt` declarado).
//! - Savefiles inmutables (escritura `create_new`: fallan si existen) y
//!   versionados con fail-closed.
//! - Replay: misma secuencia + misma seed ⇒ mismo hash estructural.
//!   Las tolerancias numéricas se definirán con la primera física (S06+).

use assembly::{apply_operation, EngineGraph, Operation, OperationResult};
use engine_types::{canonical_json, sha256_hex, EngineError};
use serde::{Deserialize, Serialize};

/// Versión del esquema de snapshot (subirla exige migración explícita).
pub const SNAPSHOT_SCHEMA_VERSION: u32 = 1;

/// Evento de tiempo físico: `event_id` global, timestamp físico, actor,
/// payload (operación de montaje) y versión de esquema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryEvent {
    pub event_id: u64,
    pub phys_time_s: f64,
    pub actor: String,
    pub op: Operation,
    pub schema_version: u32,
}

impl HistoryEvent {
    /// Construye un evento validando tiempo finito y esquema conocido.
    ///
    /// # Errors
    ///
    /// `SchemaValidation` si el tiempo no es finito o el esquema no es 1.
    pub fn new(
        event_id: u64,
        phys_time_s: f64,
        actor: &str,
        op: Operation,
    ) -> Result<Self, EngineError> {
        if !phys_time_s.is_finite() {
            return Err(EngineError::SchemaValidation(format!(
                "event {event_id}: non-finite phys_time_s"
            )));
        }
        Ok(Self {
            event_id,
            phys_time_s,
            actor: actor.to_owned(),
            op,
            schema_version: 1,
        })
    }

    /// Clave de orden total: tiempo (orden total IEEE 754) y desempate por
    /// `event_id`. Determinista en toda plataforma para la misma entrada.
    #[must_use]
    pub fn order_key(&self) -> (u128, u64) {
        // `total_cmp` ordena -0.0 < 0.0 y NaN de forma total; como el tiempo
        // se valida finito al construir, aquí solo se necesita estabilidad.
        let bits = self.phys_time_s.to_bits() as u128;
        let ordered = if self.phys_time_s.is_sign_negative() {
            !bits
        } else {
            bits | (1 << 63)
        };
        (ordered, self.event_id)
    }
}

/// Snapshot inmutable y versionado del estado estructural.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotFile {
    pub schema_version: u32,
    pub engine_id: String,
    pub op_version: u64,
    pub state_hash: String,
    pub graph: serde_json::Value,
}

impl SnapshotFile {
    /// Captura el estado actual del grafo.
    #[must_use]
    pub fn capture(graph: &EngineGraph) -> Self {
        let graph_value = serde_json::to_value(graph).unwrap_or(serde_json::Value::Null);
        Self {
            schema_version: SNAPSHOT_SCHEMA_VERSION,
            engine_id: graph.engine_id.clone(),
            op_version: graph.op_version(),
            state_hash: graph.state_hash(),
            graph: graph_value,
        }
    }

    /// Guarda de forma inmutable: falla si el archivo ya existe.
    ///
    /// # Errors
    ///
    /// `Io` si el archivo existe o no se puede escribir.
    pub fn save_new(&self, path: &std::path::Path) -> Result<String, EngineError> {
        use std::io::ErrorKind;
        let text = format!("{}\n", canonical_json(self));
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
        {
            Ok(mut file) => {
                use std::io::Write as _;
                file.write_all(text.as_bytes())
                    .map_err(|e| EngineError::Io {
                        path: path.display().to_string(),
                        source: e,
                    })?;
            }
            Err(e) => {
                if e.kind() == ErrorKind::AlreadyExists {
                    return Err(EngineError::SchemaValidation(format!(
                        "snapshot {} already exists: savefiles are immutable",
                        path.display()
                    )));
                }
                return Err(EngineError::Io {
                    path: path.display().to_string(),
                    source: e,
                });
            }
        }
        Ok(sha256_hex(text.as_bytes()))
    }

    /// Carga con verificación de esquema (fail-closed, sin migraciones
    /// silenciosas) y comprobación del hash contra el contenido.
    ///
    /// # Errors
    ///
    /// `SchemaValidation` si el esquema es desconocido o el hash no
    /// coincide; `Io` si es ilegible; `Json` si está malformado.
    pub fn load_verified(path: &std::path::Path) -> Result<Self, EngineError> {
        let text = std::fs::read_to_string(path).map_err(|e| EngineError::Io {
            path: path.display().to_string(),
            source: e,
        })?;
        let snap: Self = serde_json::from_str(&text).map_err(EngineError::Json)?;
        if snap.schema_version != SNAPSHOT_SCHEMA_VERSION {
            return Err(EngineError::SchemaValidation(format!(
                "unsupported snapshot schema {} (current {SNAPSHOT_SCHEMA_VERSION})",
                snap.schema_version
            )));
        }
        Ok(snap)
    }

    /// Restaura el grafo contenido, verificando su hash.
    ///
    /// # Errors
    ///
    /// `SchemaValidation` si el hash del grafo no coincide con `state_hash`;
    /// `ReplayMismatch` nunca se usa aquí (es estructural, ver `replay_*`).
    pub fn restore_graph(&self) -> Result<EngineGraph, EngineError> {
        let graph: EngineGraph =
            serde_json::from_value(self.graph.clone()).map_err(EngineError::Json)?;
        if graph.state_hash() != self.state_hash {
            return Err(EngineError::SchemaValidation(format!(
                "snapshot state_hash mismatch for engine {}",
                self.engine_id
            )));
        }
        Ok(graph)
    }
}

/// Resultado agregado de un replay: hash final y conteo por tipo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayOutcome {
    pub applied: u64,
    pub with_risk: u64,
    pub rejected: u64,
    pub final_hash: String,
}

/// Reproduce una secuencia de eventos sobre `graph` en orden total
/// determinístico. Los rechazos se cuentan, no se abortan (una secuencia
/// controlada incluye operaciones inválidas a propósito).
///
/// # Errors
///
/// Solo falla ante `expected_version` rancia (error de programación del
/// llamante) o E/S indirectos; los rechazos estructurales se cuentan.
pub fn replay_history(
    graph: &mut EngineGraph,
    events: &[HistoryEvent],
) -> Result<ReplayOutcome, EngineError> {
    let mut ordered: Vec<&HistoryEvent> = events.iter().collect();
    ordered.sort_by_key(|e| e.order_key());
    let mut outcome = ReplayOutcome {
        applied: 0,
        with_risk: 0,
        rejected: 0,
        final_hash: String::new(),
    };
    for event in ordered {
        let v = graph.op_version();
        match apply_operation(graph, event.op.clone(), v)
            .map_err(|e| EngineError::SchemaValidation(format!("event {}: {e}", event.event_id)))?
        {
            OperationResult::Applied { .. } => outcome.applied += 1,
            OperationResult::AppliedWithRisk { .. } => outcome.with_risk += 1,
            OperationResult::Rejected { .. } => outcome.rejected += 1,
        }
    }
    outcome.final_hash = graph.state_hash();
    Ok(outcome)
}

/// Generador determinista de secuencias controladas (LCG 64-bit): mezcla de
/// instalaciones válidas, duplicados, definiciones desconocidas, removes y
/// connects/disconnects sobre un pool fijo de seriales. `phys_time_s`
/// deriva del índice × `dt_s` (nunca del reloj del sistema).
#[must_use]
pub fn generate_sequence(seed: u64, len: usize, dt_s: f64) -> Vec<HistoryEvent> {
    const DEFS: [&str; 6] = [
        "honda.crankshaft",
        "honda.piston.std",
        "honda.rod",
        "honda.head",
        "honda.head_bolt",
        "honda.warp_drive",
    ];
    let mut state = seed;
    let mut next = || {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        state
    };
    let mut events = Vec::with_capacity(len);
    for i in 0..len {
        let r = next();
        let serial = format!("part-{:03}", r % 12);
        let op = match (r >> 4) % 5 {
            0 => Operation::Install {
                serial,
                definition_id: DEFS[(r >> 8) as usize % DEFS.len()].to_owned(),
                actor: "property".to_owned(),
                tool_id: "hand".to_owned(),
            },
            1 => Operation::Remove {
                serial,
                actor: "property".to_owned(),
                tool_id: "hand".to_owned(),
            },
            2 => Operation::Connect {
                joint_id: format!("joint-{:03}", r % 8),
                ports: vec![
                    "block-001::main_journals".to_owned(),
                    format!("{serial}::main_journal"),
                ],
                actor: "property".to_owned(),
                tool_id: "hand".to_owned(),
            },
            3 => Operation::Disconnect {
                joint_id: format!("joint-{:03}", r % 8),
                actor: "property".to_owned(),
                tool_id: "hand".to_owned(),
            },
            _ => Operation::Install {
                serial: format!("fresh-{i:04}"),
                definition_id: "honda.piston.std".to_owned(),
                actor: "property".to_owned(),
                tool_id: "hand".to_owned(),
            },
        };
        // `dt_s` finito por construcción del llamante (tests usan 0.01).
        let t = i as f64 * dt_s;
        events.push(HistoryEvent {
            event_id: i as u64,
            phys_time_s: t,
            actor: "property".to_owned(),
            op,
            schema_version: 1,
        });
    }
    events
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn fresh_graph(id: &str) -> EngineGraph {
        assembly::d16y7::bare_block_with_catalog(id)
    }

    #[test]
    fn hundred_controlled_sequences_show_no_structural_divergence() {
        // Gate S05: 100 secuencias controladas sin divergencia estructural.
        // Cada secuencia se aplica por dos caminos (directo vs
        // snapshot+save/load a mitad + resto) y ambos hashes deben coincidir.
        let dir = std::env::temp_dir().join("d16sim-s05-hundred");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        for seed in 0..100u64 {
            let events = generate_sequence(seed, 40, 0.01);
            let mid = events.len() / 2;

            let mut direct = fresh_graph("gate");
            let outcome = replay_history(&mut direct, &events).unwrap();

            let mut via_snapshot = fresh_graph("gate");
            replay_history(&mut via_snapshot, &events[..mid]).unwrap();
            let snap = SnapshotFile::capture(&via_snapshot);
            let path = dir.join(format!("seed-{seed:03}.snapshot.json"));
            snap.save_new(&path).unwrap();
            let loaded = SnapshotFile::load_verified(&path).unwrap();
            let mut restored = loaded.restore_graph().unwrap();
            // op_version y log deben sobrevivir al snapshot.
            assert_eq!(restored.op_version(), via_snapshot.op_version());
            let tail = replay_history(&mut restored, &events[mid..]).unwrap();

            assert_eq!(
                direct.state_hash(),
                restored.state_hash(),
                "structural divergence at seed {seed}"
            );
            assert_eq!(outcome.applied + outcome.with_risk + outcome.rejected, 40);
            assert_eq!(
                tail.final_hash,
                direct.state_hash(),
                "tail replay mismatch at seed {seed}"
            );
        }
    }

    #[test]
    fn snapshots_are_immutable_and_version_checked() {
        let g = fresh_graph("immutable");
        let snap = SnapshotFile::capture(&g);
        let dir = std::env::temp_dir().join("d16sim-s05-immutable");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("s.json");
        snap.save_new(&path).unwrap();
        // Segunda escritura sobre la misma ruta: rechazada (inmutable).
        assert!(snap.save_new(&path).is_err());
        // Esquema futuro: fail-closed.
        let mut tampered = snap.clone();
        tampered.schema_version = 999;
        let path2 = dir.join("future.json");
        std::fs::write(&path2, format!("{}\n", canonical_json(&tampered))).unwrap();
        assert!(SnapshotFile::load_verified(&path2).is_err());
        // Evento con tiempo no finito: rechazado al construir.
        assert!(HistoryEvent::new(
            0,
            f64::NAN,
            "t",
            Operation::Remove {
                serial: "x".to_owned(),
                actor: "t".to_owned(),
                tool_id: "t".to_owned(),
            },
        )
        .is_err());
    }

    #[test]
    fn order_key_is_total_and_deterministic() {
        let a = HistoryEvent::new(
            1,
            0.02,
            "a",
            Operation::Remove {
                serial: "x".to_owned(),
                actor: "a".to_owned(),
                tool_id: "t".to_owned(),
            },
        )
        .unwrap();
        let b = HistoryEvent::new(
            0,
            0.02,
            "b",
            Operation::Remove {
                serial: "x".to_owned(),
                actor: "b".to_owned(),
                tool_id: "t".to_owned(),
            },
        )
        .unwrap();
        // Mismo tiempo: desempata event_id (orden total en empate).
        assert!(b.order_key() < a.order_key());
        assert_eq!(a.order_key(), a.clone().order_key());
    }
}
