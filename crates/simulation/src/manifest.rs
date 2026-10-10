//! Manifiesto de simulación (Foundation §12.1, schema v1).

use serde::{Deserialize, Serialize};

/// Manifiesto canónico de una corrida. Todos los campos son obligatorios;
/// el parser rechaza valores ausentes en vez de rellenarlos.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunManifest {
    pub schema_version: u32,
    pub experiment_id: String,
    pub engine_variant: String,
    pub model_pack: String,
    pub parameter_pack_sha256: String,
    pub assembly_snapshot_sha256: String,
    pub operation_log_sha256: String,
    pub seed: u64,
    pub software: SoftwareInfo,
    pub integrator: IntegratorInfo,
    pub scenario: ScenarioInfo,
    pub commands: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoftwareInfo {
    pub rust_toolchain: String,
    pub target_triple: String,
    pub build_profile: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegratorInfo {
    pub name: String,
    pub abs_tol: String,
    pub rel_tol: String,
    pub max_step_s: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScenarioInfo {
    pub ambient_pressure_pa: String,
    pub ambient_temperature_k: String,
    pub engine_initial_omega_rad_s: f64,
    pub simulated_ticks: u64,
}

/// Manifiesto de ejemplo para `init` y CI. Los `*_sha256` son placeholders
/// de formato (64 hex) que el usuario sustituye por los reales; el hash
/// del manifiesto se calcula sobre el contenido efectivo.
#[must_use]
pub fn example_manifest() -> RunManifest {
    RunManifest {
        schema_version: 1,
        experiment_id: "plumbing_selftest_001".to_owned(),
        engine_variant: "us_1997_civic_dx_5mt_obd2a_pending".to_owned(),
        model_pack: "physics-pack-v0.0-none".to_owned(),
        parameter_pack_sha256: "0".repeat(64),
        assembly_snapshot_sha256: "0".repeat(64),
        operation_log_sha256: "0".repeat(64),
        seed: 20261010,
        software: SoftwareInfo {
            rust_toolchain: "1.99.0".to_owned(),
            target_triple: "x86_64-unknown-linux-gnu".to_owned(),
            build_profile: "dev-or-release-per-invocation".to_owned(),
        },
        integrator: IntegratorInfo {
            name: "deterministic-counter-plumbing".to_owned(),
            abs_tol: "n/a-no-physics".to_owned(),
            rel_tol: "n/a-no-physics".to_owned(),
            max_step_s: "n/a-no-physics".to_owned(),
        },
        scenario: ScenarioInfo {
            ambient_pressure_pa: "101325".to_owned(),
            ambient_temperature_k: "293.15".to_owned(),
            engine_initial_omega_rad_s: 0.0,
            simulated_ticks: 64,
        },
        commands: vec![],
    }
}
