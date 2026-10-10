//! Grafo persistente del motor: registro, instancias, joints y log.

use crate::ops::OperationRecord;
use crate::types::{AssemblyError, Joint, PartDefinition, PartInstance, Port};
use crate::SCHEMA_VERSION;
use engine_types::{canonical_json, sha256_hex};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Estado estructural completo y persistente del motor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineGraph {
    pub schema_version: u32,
    pub engine_id: String,
    pub definitions: BTreeMap<String, PartDefinition>,
    pub instances: BTreeMap<String, PartInstance>,
    pub ports: BTreeMap<String, Port>,
    pub joints: BTreeMap<String, Joint>,
    pub operation_log: Vec<OperationRecord>,
    op_version: u64,
}

impl EngineGraph {
    /// Grafo vacío con esquema actual.
    #[must_use]
    pub fn empty(engine_id: &str) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            engine_id: engine_id.to_owned(),
            definitions: BTreeMap::new(),
            instances: BTreeMap::new(),
            ports: BTreeMap::new(),
            joints: BTreeMap::new(),
            operation_log: Vec::new(),
            op_version: 0,
        }
    }

    /// Registra una definición nominal (inmutable a partir de aquí).
    pub fn register_definition(&mut self, def: PartDefinition) {
        self.definitions.insert(def.id.clone(), def);
    }

    #[must_use]
    pub fn op_version(&self) -> u64 {
        self.op_version
    }

    pub(crate) fn push_record(&mut self, record: OperationRecord) {
        self.operation_log.push(record);
        self.op_version += 1;
    }

    /// Hash canónico del estado estructural (excluye el log para evitar
    /// autorreferencia; el log cita hashes prev/post por registro).
    #[must_use]
    pub fn state_hash(&self) -> String {
        let view = serde_json::json!({
            "schema_version": self.schema_version,
            "engine_id": self.engine_id,
            "definitions": self.definitions,
            "instances": self.instances,
            "ports": self.ports,
            "joints": self.joints,
            "op_version": self.op_version,
        });
        sha256_hex(canonical_json(&view).as_bytes())
    }

    /// Guarda el grafo en disco (JSON canónico + `.sha256` acompañante).
    ///
    /// # Errors
    ///
    /// Retorna [`AssemblyError::Store`] si la escritura falla.
    pub fn save_to(&self, path: &std::path::Path) -> Result<String, AssemblyError> {
        let text = format!("{}\n", canonical_json(self));
        std::fs::write(path, &text).map_err(|e| AssemblyError::Store(e.to_string()))?;
        let hash = sha256_hex(text.as_bytes());
        let sidecar = path.with_extension("json.sha256");
        std::fs::write(&sidecar, format!("{hash}\n"))
            .map_err(|e| AssemblyError::Store(e.to_string()))?;
        Ok(hash)
    }

    /// Carga un grafo con verificación de esquema (fail-closed).
    ///
    /// # Errors
    ///
    /// `UnsupportedSchema` si la versión no es la actual (sin migración
    /// silenciosa); `Store` si el JSON es inválido o ilegible.
    pub fn load_from(path: &std::path::Path) -> Result<Self, AssemblyError> {
        let text =
            std::fs::read_to_string(path).map_err(|e| AssemblyError::Store(e.to_string()))?;
        let graph: Self =
            serde_json::from_str(&text).map_err(|e| AssemblyError::Store(e.to_string()))?;
        if graph.schema_version != SCHEMA_VERSION {
            return Err(AssemblyError::UnsupportedSchema {
                found: graph.schema_version,
                current: SCHEMA_VERSION,
            });
        }
        Ok(graph)
    }

    /// Instancia por serial (`None` si no existe: las piezas ausentes no
    /// aparecen mágicamente).
    #[must_use]
    pub fn instance(&self, serial: &str) -> Option<&PartInstance> {
        self.instances.get(serial)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::d16y7::bare_block_with_catalog;
    use crate::ops::{apply_operation, Operation, OperationResult};

    fn install_op(serial: &str, def: &str) -> Operation {
        Operation::Install {
            serial: serial.to_owned(),
            definition_id: def.to_owned(),
            actor: "tech".to_owned(),
            tool_id: "hand".to_owned(),
        }
    }

    #[test]
    fn out_of_order_install_is_allowed_when_ports_exist() {
        // Pistón antes que biela: factible si los puertos existen; el orden
        // de misión no lo prohíbe (gate S04: montaje fuera de orden).
        let mut g = bare_block_with_catalog("test-ooo");
        let v = g.op_version();
        assert!(matches!(
            apply_operation(&mut g, install_op("piston-001", "honda.piston.std"), v),
            Ok(OperationResult::Applied { .. })
        ));
    }

    #[test]
    fn missing_part_does_not_appear_and_duplicates_rejected() {
        let mut g = bare_block_with_catalog("test-missing");
        assert!(g.instance("crank-999").is_none());
        let v = g.op_version();
        assert!(matches!(
            apply_operation(&mut g, install_op("crank-001", "honda.crankshaft"), v),
            Ok(OperationResult::Applied { .. })
        ));
        let v = g.op_version();
        assert!(matches!(
            apply_operation(&mut g, install_op("crank-001", "honda.crankshaft"), v),
            Ok(OperationResult::Rejected { .. })
        ));
        // Definición desconocida: imposible, no aparición mágica.
        let v = g.op_version();
        assert!(matches!(
            apply_operation(&mut g, install_op("x-1", "honda.warp_drive"), v),
            Ok(OperationResult::Rejected { .. })
        ));
    }

    #[test]
    fn serial_survives_remove_and_save_load_roundtrip() {
        let mut g = bare_block_with_catalog("test-serial");
        let v = g.op_version();
        assert!(matches!(
            apply_operation(&mut g, install_op("crank-001", "honda.crankshaft"), v),
            Ok(OperationResult::Applied { .. })
        ));
        let v = g.op_version();
        let op = Operation::Remove {
            serial: "crank-001".to_owned(),
            actor: "tech".to_owned(),
            tool_id: "hand".to_owned(),
        };
        assert!(matches!(
            apply_operation(&mut g, op, v),
            Ok(OperationResult::Applied { .. })
        ));
        assert!(g.instance("crank-001").is_none());
        // Reinstalar conserva el serial como identidad (historial previo en log).
        let v = g.op_version();
        assert!(matches!(
            apply_operation(&mut g, install_op("crank-001", "honda.crankshaft"), v),
            Ok(OperationResult::Applied { .. })
        ));
        assert_eq!(g.instance("crank-001").unwrap().serial, "crank-001");

        let dir = std::env::temp_dir().join("d16sim-s04-serial");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("graph.json");
        let h1 = g.save_to(&path).unwrap();
        let g2 = EngineGraph::load_from(&path).unwrap();
        assert_eq!(g.state_hash(), g2.state_hash());
        assert_eq!(g2.instance("crank-001").unwrap().serial, "crank-001");
        let sidecar = std::fs::read_to_string(path.with_extension("json.sha256")).unwrap();
        assert!(sidecar.trim() == h1);
    }

    #[test]
    fn incompatible_ports_rejected_and_stale_version_fails() {
        let mut g = bare_block_with_catalog("test-compat");
        let v = g.op_version();
        let _ = apply_operation(&mut g, install_op("crank-001", "honda.crankshaft"), v);
        let v = g.op_version();
        let _ = apply_operation(&mut g, install_op("head-001", "honda.head"), v);
        // Mecánico contra eléctrico: imposible nominal.
        let v = g.op_version();
        let op = Operation::Connect {
            joint_id: "bad-joint".to_owned(),
            ports: vec![
                "crank-001::main_journal".to_owned(),
                "head-001::ecu_pin".to_owned(),
            ],
            actor: "tech".to_owned(),
            tool_id: "hand".to_owned(),
        };
        // `ecu_pin` no existe en este catálogo mínimo → rechazado por puerto
        // desconocido (antes incluso que por incompatibilidad de clase).
        assert!(matches!(
            apply_operation(&mut g, op, v),
            Ok(OperationResult::Rejected { .. })
        ));
        // Versión rancia: error, no aplicación silenciosa.
        assert!(apply_operation(&mut g, install_op("z-1", "honda.crankshaft"), 0).is_err());
    }

    #[test]
    fn unknown_schema_is_fail_closed() {
        let dir = std::env::temp_dir().join("d16sim-s04-schema");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("future.json");
        std::fs::write(&path, r#"{"schema_version":999,"engine_id":"x","definitions":{},"instances":{},"ports":{},"joints":{},"operation_log":[],"op_version":0}"#).unwrap();
        assert!(matches!(
            EngineGraph::load_from(&path),
            Err(AssemblyError::UnsupportedSchema { found: 999, .. })
        ));
    }
}
