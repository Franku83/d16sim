//! Runner de plumbing determinista + replay + reporte (S02).
//!
//! Determinismo: el autocontador usa aritmética `wrapping_*` sobre `u64`
//! (comportamiento definido e idéntico en toda plataforma para el mismo
//! manifiesto). Equivalencia bit a bit entre SO/toolchains distintos NO
//! garantizada a nivel binario; lo garantizado es: mismo manifiesto =>
//! mismos bytes de salida (verificado por `replay`).

use crate::manifest::RunManifest;
use engine_types::{canonical_json, sha256_hex, EngineError};
use serde::{Deserialize, Serialize};

/// Resumen de una corrida (lo que `report` vuelve a leer del disco).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunSummary {
    pub experiment_id: String,
    pub seed: u64,
    pub manifest_sha256: String,
    pub traces_sha256: String,
    pub metrics_sha256: String,
    pub ticks: u64,
    pub runner_plumbing_only: bool,
}

/// Ejecuta el escenario y escribe `traces.jsonl`, `metrics.json`,
/// `hashes.json` en `out_dir` (creado si no existe).
pub fn run_scenario(
    manifest_path: &std::path::Path,
    out_dir: &std::path::Path,
) -> Result<RunSummary, EngineError> {
    let manifest_bytes = std::fs::read(manifest_path).map_err(|e| EngineError::Io {
        path: manifest_path.display().to_string(),
        source: e,
    })?;
    let manifest: RunManifest =
        serde_json::from_slice(&manifest_bytes).map_err(EngineError::Json)?;
    if manifest.schema_version != 1 {
        return Err(EngineError::SchemaValidation(format!(
            "unsupported schema_version {} (expected 1) in {}",
            manifest.schema_version,
            manifest_path.display()
        )));
    }

    let manifest_sha256 = sha256_hex(canonical_json(&manifest).as_bytes());
    let ticks = manifest.scenario.simulated_ticks;

    // Autocontador determinista derivado de la seed (LCG 64-bit, constantes
    // de Knuth; wrapping = definido en Rust en toda plataforma).
    let mut state = manifest.seed;
    let mut traces = String::new();
    for tick in 0..ticks {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let row = serde_json::json!({"tick": tick, "counter": state});
        traces.push_str(&canonical_json(&row));
        traces.push('\n');
    }
    let traces_sha256 = sha256_hex(traces.as_bytes());

    let metrics = serde_json::json!({
        "experiment_id": manifest.experiment_id,
        "seed": manifest.seed,
        "model_pack": manifest.model_pack,
        "engine_variant": manifest.engine_variant,
        "input_manifest_sha256": manifest_sha256,
        "output_traces_sha256": traces_sha256,
        "ticks": ticks,
        "runner_plumbing_only": true,
        "note": "S02 deterministic I/O plumbing self-test; no physics computed",
    });
    let metrics_str = format!("{}\n", canonical_json(&metrics));
    let metrics_sha256 = sha256_hex(metrics_str.as_bytes());

    let hashes = serde_json::json!({
        "manifest_sha256": manifest_sha256,
        "traces_sha256": traces_sha256,
        "metrics_sha256": metrics_sha256,
    });
    let hashes_str = format!("{}\n", canonical_json(&hashes));

    std::fs::create_dir_all(out_dir).map_err(|e| EngineError::Io {
        path: out_dir.display().to_string(),
        source: e,
    })?;
    for (name, bytes) in [
        ("traces.jsonl", traces.as_bytes()),
        ("metrics.json", metrics_str.as_bytes()),
        ("hashes.json", hashes_str.as_bytes()),
    ] {
        std::fs::write(out_dir.join(name), bytes).map_err(|e| EngineError::Io {
            path: out_dir.join(name).display().to_string(),
            source: e,
        })?;
    }

    Ok(RunSummary {
        experiment_id: manifest.experiment_id,
        seed: manifest.seed,
        manifest_sha256,
        traces_sha256,
        metrics_sha256,
        ticks,
        runner_plumbing_only: true,
    })
}

/// Reejecuta en memoria y compara contra los artefactos registrados.
pub fn replay(
    manifest_path: &std::path::Path,
    run_dir: &std::path::Path,
) -> Result<RunSummary, EngineError> {
    let manifest_bytes = std::fs::read(manifest_path).map_err(|e| EngineError::Io {
        path: manifest_path.display().to_string(),
        source: e,
    })?;
    let manifest: RunManifest =
        serde_json::from_slice(&manifest_bytes).map_err(EngineError::Json)?;
    if manifest.schema_version != 1 {
        return Err(EngineError::SchemaValidation(format!(
            "unsupported schema_version {} (expected 1)",
            manifest.schema_version
        )));
    }

    let recorded_traces =
        std::fs::read(run_dir.join("traces.jsonl")).map_err(|e| EngineError::Io {
            path: run_dir.join("traces.jsonl").display().to_string(),
            source: e,
        })?;
    let recorded_hashes =
        std::fs::read_to_string(run_dir.join("hashes.json")).map_err(|e| EngineError::Io {
            path: run_dir.join("hashes.json").display().to_string(),
            source: e,
        })?;
    let recorded: serde_json::Value =
        serde_json::from_str(&recorded_hashes).map_err(EngineError::Json)?;

    let mut state = manifest.seed;
    let mut fresh = String::new();
    for tick in 0..manifest.scenario.simulated_ticks {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let row = serde_json::json!({"tick": tick, "counter": state});
        fresh.push_str(&canonical_json(&row));
        fresh.push('\n');
    }
    let fresh_traces_sha = sha256_hex(fresh.as_bytes());
    let expected = recorded
        .get("traces_sha256")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("<missing>");
    if fresh_traces_sha != expected || fresh.as_bytes() != recorded_traces.as_slice() {
        return Err(EngineError::ReplayMismatch {
            expected: expected.to_owned(),
            actual: fresh_traces_sha,
        });
    }

    let manifest_sha256 = sha256_hex(canonical_json(&manifest).as_bytes());
    Ok(RunSummary {
        experiment_id: manifest.experiment_id,
        seed: manifest.seed,
        manifest_sha256,
        traces_sha256: fresh_traces_sha,
        metrics_sha256: recorded
            .get("metrics_sha256")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("<missing>")
            .to_owned(),
        ticks: manifest.scenario.simulated_ticks,
        runner_plumbing_only: true,
    })
}

/// Lee un directorio de corrida y devuelve el resumen legible.
pub fn report_summary(run_dir: &std::path::Path) -> Result<RunSummary, EngineError> {
    let hashes_str =
        std::fs::read_to_string(run_dir.join("hashes.json")).map_err(|e| EngineError::Io {
            path: run_dir.join("hashes.json").display().to_string(),
            source: e,
        })?;
    let metrics_str =
        std::fs::read_to_string(run_dir.join("metrics.json")).map_err(|e| EngineError::Io {
            path: run_dir.join("metrics.json").display().to_string(),
            source: e,
        })?;
    let hashes: serde_json::Value = serde_json::from_str(&hashes_str).map_err(EngineError::Json)?;
    let metrics: serde_json::Value =
        serde_json::from_str(&metrics_str).map_err(EngineError::Json)?;
    let get = |obj: &serde_json::Value, key: &str| -> Result<String, EngineError> {
        obj.get(key)
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| EngineError::SchemaValidation(format!("missing key `{key}`")))
    };
    let ticks = metrics
        .get("ticks")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| EngineError::SchemaValidation("missing key `ticks`".to_owned()))?;
    let seed = metrics
        .get("seed")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| EngineError::SchemaValidation("missing key `seed`".to_owned()))?;
    Ok(RunSummary {
        experiment_id: get(&metrics, "experiment_id")?,
        seed,
        manifest_sha256: get(&hashes, "manifest_sha256")?,
        traces_sha256: get(&hashes, "traces_sha256")?,
        metrics_sha256: get(&hashes, "metrics_sha256")?,
        ticks,
        runner_plumbing_only: metrics
            .get("runner_plumbing_only")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::manifest::example_manifest;

    #[test]
    fn run_then_replay_matches() {
        let dir = std::env::temp_dir().join("d16sim-s02-selftest");
        let _ = std::fs::remove_dir_all(&dir);
        let manifest_path = dir.join("manifest.json");
        std::fs::create_dir_all(&dir).unwrap();
        let manifest = example_manifest();
        std::fs::write(&manifest_path, canonical_json(&manifest)).unwrap();
        let out = dir.join("run");
        let summary = run_scenario(&manifest_path, &out).unwrap();
        assert!(summary.runner_plumbing_only);
        let replayed = replay(&manifest_path, &out).unwrap();
        assert_eq!(summary.traces_sha256, replayed.traces_sha256);
        let reported = report_summary(&out).unwrap();
        assert_eq!(summary, reported);
    }

    #[test]
    fn tampered_traces_fail_replay() {
        // Prueba que falla sin el mecanismo: sin hashing, la manipulación
        // pasaría inadvertida (requisito S05 adelantado como contrato).
        let dir = std::env::temp_dir().join("d16sim-s02-tamper");
        let _ = std::fs::remove_dir_all(&dir);
        let manifest_path = dir.join("manifest.json");
        std::fs::create_dir_all(&dir).unwrap();
        let manifest = example_manifest();
        std::fs::write(&manifest_path, canonical_json(&manifest)).unwrap();
        let out = dir.join("run");
        run_scenario(&manifest_path, &out).unwrap();
        std::fs::write(out.join("traces.jsonl"), "{}\n").unwrap();
        let err = replay(&manifest_path, &out).expect_err("tamper must be detected");
        assert!(matches!(err, EngineError::ReplayMismatch { .. }));
    }

    #[test]
    fn bad_schema_version_is_rejected() {
        let dir = std::env::temp_dir().join("d16sim-s02-badschema");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut manifest = example_manifest();
        manifest.schema_version = 999;
        let manifest_path = dir.join("manifest.json");
        std::fs::write(&manifest_path, canonical_json(&manifest)).unwrap();
        let err = run_scenario(&manifest_path, &dir.join("run"))
            .expect_err("schema 999 must be rejected");
        assert!(matches!(err, EngineError::SchemaValidation(_)));
    }
}
