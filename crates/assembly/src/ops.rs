//! Operaciones atómicas sobre el grafo.

use crate::graph::EngineGraph;
use crate::types::{AssemblyError, Joint, PartInstance, Port, PortKind};
use serde::{Deserialize, Serialize};

/// Comando reversible del taller (actor/herramienta/objetivo/parámetros).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation {
    /// Da de alta una instancia de una definición registrada.
    Install {
        serial: String,
        definition_id: String,
        actor: String,
        tool_id: String,
    },
    /// Retira una instancia (conserva su serial e historial).
    Remove {
        serial: String,
        actor: String,
        tool_id: String,
    },
    /// Une puertos compatibles bajo un joint nuevo.
    Connect {
        joint_id: String,
        ports: Vec<String>,
        actor: String,
        tool_id: String,
    },
    /// Disuelve un joint (las instancias permanecen).
    Disconnect {
        joint_id: String,
        actor: String,
        tool_id: String,
    },
}

/// Registro trazable de una operación aplicada o rechazada (§5.5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationRecord {
    pub seq: u64,
    pub actor: String,
    pub tool_id: String,
    pub target: String,
    pub params: serde_json::Value,
    pub result: String,
    pub prev_hash: String,
    pub post_hash: String,
}

/// Resultado tri-estado: aplicada, aplicada con riesgo estructural anotado,
/// o rechazada por imposibilidad (sin daño inventado en ningún caso).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationResult {
    Applied {
        record: OperationRecord,
    },
    AppliedWithRisk {
        record: OperationRecord,
        risk_note: String,
    },
    Rejected {
        reason: String,
    },
}

/// Aplica una operación atómica y la registra en el log append-only.
pub fn apply_operation(
    graph: &mut EngineGraph,
    op: Operation,
    expected_version: u64,
) -> Result<OperationResult, AssemblyError> {
    if graph.op_version() != expected_version {
        return Err(AssemblyError::Impossible {
            reason: format!(
                "stale op_version: expected {expected_version}, graph at {}",
                graph.op_version()
            ),
        });
    }
    let prev_hash = graph.state_hash();
    let seq = graph.op_version() + 1;

    let outcome: Result<Option<String>, String> = match op.clone() {
        Operation::Install {
            serial,
            definition_id,
            ..
        } => install(graph, &serial, &definition_id),
        Operation::Remove { serial, .. } => remove(graph, &serial),
        Operation::Connect {
            joint_id, ports, ..
        } => connect(graph, &joint_id, &ports),
        Operation::Disconnect { joint_id, .. } => disconnect(graph, &joint_id),
    };

    let (actor, tool_id, target, params) = describe(&op);
    let post_hash = graph.state_hash();
    let mk_record = |result: &str| OperationRecord {
        seq,
        actor: actor.clone(),
        tool_id: tool_id.clone(),
        target: target.clone(),
        params: params.clone(),
        result: result.to_owned(),
        prev_hash: prev_hash.clone(),
        post_hash: post_hash.clone(),
    };

    match outcome {
        Ok(risk) => {
            let record = mk_record(if risk.is_some() {
                "applied_with_risk"
            } else {
                "applied"
            });
            graph.push_record(record.clone());
            if let Some(risk_note) = risk {
                Ok(OperationResult::AppliedWithRisk { record, risk_note })
            } else {
                Ok(OperationResult::Applied { record })
            }
        }
        Err(reason) => Ok(OperationResult::Rejected { reason }),
    }
}

fn install(
    graph: &mut EngineGraph,
    serial: &str,
    definition_id: &str,
) -> Result<Option<String>, String> {
    if graph.instances.contains_key(serial) {
        return Err(format!("duplicate instance serial `{serial}`"));
    }
    let def = graph.definitions.get(definition_id).ok_or_else(|| {
        format!("unknown definition `{definition_id}`: parts do not appear by magic")
    })?;
    let mut ports = Vec::new();
    for iface in &def.mounting_interfaces {
        ports.push(Port {
            id: format!("{serial}::{}", iface.name),
            instance_serial: serial.to_owned(),
            kind: iface.kind,
        });
    }
    graph.instances.insert(
        serial.to_owned(),
        PartInstance {
            serial: serial.to_owned(),
            definition_id: definition_id.to_owned(),
            cycles: 0,
            history: vec![format!("installed as {definition_id}")],
        },
    );
    for port in ports {
        graph.ports.insert(port.id.clone(), port);
    }
    // Riesgo honesto: instalar culata sin junta ni tornillos es
    // estructuralmente posible (las piezas existen y encajan) pero el
    // sellado queda indeterminado hasta el modelo S08/S23.
    if definition_id.contains("head") && !definition_id.contains("gasket") {
        Ok(Some(
            "head installed without gasket/fastener preload model: sealing indeterminate (S08/S23)"
                .to_owned(),
        ))
    } else {
        Ok(None)
    }
}

fn remove(graph: &mut EngineGraph, serial: &str) -> Result<Option<String>, String> {
    if graph.instances.remove(serial).is_none() {
        return Err(format!("no such instance `{serial}`"));
    }
    // Los joints que usaban sus puertos quedan disueltos (trazado en log).
    let dead_ports: Vec<String> = graph
        .ports
        .keys()
        .filter(|id| id.starts_with(&format!("{serial}::")))
        .cloned()
        .collect();
    for id in &dead_ports {
        graph.ports.remove(id);
    }
    let dead_joints: Vec<String> = graph
        .joints
        .iter()
        .filter(|(_, j)| j.ports.iter().any(|p| dead_ports.contains(p)))
        .map(|(id, _)| id.clone())
        .collect();
    for id in &dead_joints {
        graph.joints.remove(id);
    }
    if dead_joints.is_empty() {
        Ok(None)
    } else {
        Ok(Some(format!(
            "removal dissolved joints {dead_joints:?}: connected parts stay in place, unjoined"
        )))
    }
}

fn connect(
    graph: &mut EngineGraph,
    joint_id: &str,
    ports: &[String],
) -> Result<Option<String>, String> {
    if graph.joints.contains_key(joint_id) {
        return Err(format!("duplicate joint `{joint_id}`"));
    }
    if ports.len() < 2 {
        return Err("a joint needs >= 2 ports".to_owned());
    }
    let mut kinds = Vec::new();
    for p in ports {
        match graph.ports.get(p) {
            Some(port) => kinds.push(port.kind),
            None => return Err(format!("unknown port `{p}`")),
        }
    }
    // Compatibilidad nominal S04: todos los puertos del mismo joint deben
    // compartir PortKind. La compatibilidad fina (diámetro/orientación) es S07.
    let first: PortKind = kinds[0];
    if kinds.iter().any(|k| *k != first) {
        return Err(format!(
            "incompatible port kinds in joint `{joint_id}`: {kinds:?} (nominal check S04)"
        ));
    }
    graph.joints.insert(
        joint_id.to_owned(),
        Joint {
            id: joint_id.to_owned(),
            ports: ports.to_vec(),
            install_history: vec!["connected".to_owned()],
        },
    );
    Ok(None)
}

fn disconnect(graph: &mut EngineGraph, joint_id: &str) -> Result<Option<String>, String> {
    if graph.joints.remove(joint_id).is_none() {
        return Err(format!("no such joint `{joint_id}`"));
    }
    Ok(None)
}

fn describe(op: &Operation) -> (String, String, String, serde_json::Value) {
    match op {
        Operation::Install {
            serial,
            definition_id,
            actor,
            tool_id,
        } => (
            actor.clone(),
            tool_id.clone(),
            serial.clone(),
            serde_json::json!({"op": "install", "definition": definition_id}),
        ),
        Operation::Remove {
            serial,
            actor,
            tool_id,
        } => (
            actor.clone(),
            tool_id.clone(),
            serial.clone(),
            serde_json::json!({"op": "remove"}),
        ),
        Operation::Connect {
            joint_id,
            ports,
            actor,
            tool_id,
        } => (
            actor.clone(),
            tool_id.clone(),
            joint_id.clone(),
            serde_json::json!({"op": "connect", "ports": ports}),
        ),
        Operation::Disconnect {
            joint_id,
            actor,
            tool_id,
        } => (
            actor.clone(),
            tool_id.clone(),
            joint_id.clone(),
            serde_json::json!({"op": "disconnect"}),
        ),
    }
}
