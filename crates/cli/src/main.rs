//! CLI headless `d16sim` — S02: init, validate-data, run-scenario,
//! replay, report. Sin física: los comandos de corrida usan el runner de
//! plumbing determinista y etiquetan su salida en consecuencia.

use clap::{Parser, Subcommand};
use engine_types::{canonical_json, EngineError};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "d16sim",
    version,
    about = "D16Y7 engineering sandbox runner (S02: plumbing only, no physics)"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Escribe un manifiesto de ejemplo en la ruta indicada.
    Init {
        #[arg(long)]
        manifest: PathBuf,
    },
    /// Valida `data/d16y7/` contra los contratos de esquema de S01.
    ValidateData {
        #[arg(long, default_value = "data")]
        data_dir: PathBuf,
    },
    /// Ejecuta un escenario y escribe traces/metrics/hashes.
    RunScenario {
        #[arg(long)]
        manifest: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    /// Reejecuta y compara contra los artefactos registrados.
    Replay {
        #[arg(long)]
        manifest: PathBuf,
        #[arg(long)]
        run_dir: PathBuf,
    },
    /// Muestra el resumen de una corrida.
    Report {
        #[arg(long)]
        run_dir: PathBuf,
    },
    /// Estado del crate de física (siempre stub en S02).
    PhysicsStatus,
    /// Demostración reproducible de unidades e interpolación (S03).
    UnitsDemo,
    /// Demostración reproducible del grafo de montaje (S04).
    AssemblyDemo {
        #[arg(long)]
        out: PathBuf,
    },
}

fn run() -> Result<(), EngineError> {
    let cli = Cli::parse();
    match cli.command {
        Command::Init { manifest } => {
            let example = simulation::manifest::example_manifest();
            if let Some(parent) = manifest.parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent).map_err(|e| EngineError::Io {
                        path: parent.display().to_string(),
                        source: e,
                    })?;
                }
            }
            std::fs::write(&manifest, format!("{}\n", canonical_json(&example))).map_err(|e| {
                EngineError::Io {
                    path: manifest.display().to_string(),
                    source: e,
                }
            })?;
            println!("wrote example manifest to {}", manifest.display());
            Ok(())
        }
        Command::ValidateData { data_dir } => {
            let issues = simulation::validate_data_dir(&data_dir)?;
            if issues.is_empty() {
                println!("validate-data: OK ({})", data_dir.display());
                Ok(())
            } else {
                for issue in &issues {
                    println!("schema issue: {issue}");
                }
                Err(EngineError::SchemaValidation(format!(
                    "{} issue(s) in {}",
                    issues.len(),
                    data_dir.display()
                )))
            }
        }
        Command::RunScenario { manifest, out } => {
            let summary = simulation::run_scenario(&manifest, &out)?;
            println!(
                "run-scenario: experiment={} ticks={} manifest={} traces={} metrics={} plumbing_only={}",
                summary.experiment_id,
                summary.ticks,
                summary.manifest_sha256,
                summary.traces_sha256,
                summary.metrics_sha256,
                summary.runner_plumbing_only,
            );
            Ok(())
        }
        Command::Replay { manifest, run_dir } => {
            let summary = simulation::replay(&manifest, &run_dir)?;
            println!(
                "replay: MATCH experiment={} traces={}",
                summary.experiment_id, summary.traces_sha256
            );
            Ok(())
        }
        Command::Report { run_dir } => {
            let summary = simulation::report_summary(&run_dir)?;
            println!(
                "report: experiment={} seed={} ticks={} manifest={} traces={} metrics={} plumbing_only={}",
                summary.experiment_id,
                summary.seed,
                summary.ticks,
                summary.manifest_sha256,
                summary.traces_sha256,
                summary.metrics_sha256,
                summary.runner_plumbing_only,
            );
            Ok(())
        }
        Command::PhysicsStatus => {
            println!("{}", physics::status());
            match physics::closed_cycle_placeholder() {
                Ok(()) => Ok(()),
                Err(e) => {
                    println!("closed-cycle solver: {e}");
                    Ok(())
                }
            }
        }
        Command::UnitsDemo => units_demo(),
        Command::AssemblyDemo { out } => assembly_demo(&out),
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("d16sim: error: {e:#}");
        std::process::exit(1);
    }
}

/// Demostración S04: bloque desnudo + cigüeñal + pistón, joints,
/// guardado, recarga y comparación de hashes. Sin física.
fn assembly_demo(out: &std::path::Path) -> Result<(), EngineError> {
    use assembly::{apply_operation, EngineGraph, Operation, OperationResult};

    let mut graph = assembly::d16y7::bare_block_with_catalog("demo-engine-001");
    let h0 = graph.state_hash();
    let mut v = graph.op_version();
    let seq = [
        Operation::Install {
            serial: "crank-001".to_owned(),
            definition_id: "honda.crankshaft".to_owned(),
            actor: "demo".to_owned(),
            tool_id: "hand".to_owned(),
        },
        Operation::Install {
            serial: "piston-001".to_owned(),
            definition_id: "honda.piston.std".to_owned(),
            actor: "demo".to_owned(),
            tool_id: "hand".to_owned(),
        },
    ];
    for op in seq {
        match apply_operation(&mut graph, op, v) {
            Ok(OperationResult::Applied { .. }) => {}
            Ok(OperationResult::AppliedWithRisk { risk_note, .. }) => {
                println!("assembly-demo note (risk): {risk_note}");
            }
            Ok(OperationResult::Rejected { reason }) => {
                return Err(EngineError::SchemaValidation(reason));
            }
            Err(e) => {
                return Err(EngineError::SchemaValidation(e.to_string()));
            }
        }
        v = graph.op_version();
    }
    // Joint cigüeñal-bloque por puertos mecánicos compatibles.
    let joint = Operation::Connect {
        joint_id: "crank-mains".to_owned(),
        ports: vec![
            "block-001::main_journals".to_owned(),
            "crank-001::main_journal".to_owned(),
        ],
        actor: "demo".to_owned(),
        tool_id: "hand".to_owned(),
    };
    match apply_operation(&mut graph, joint, v) {
        Ok(OperationResult::Applied { .. }) => {}
        Ok(other) => {
            return Err(EngineError::SchemaValidation(format!(
                "unexpected joint outcome: {other:?}"
            )));
        }
        Err(e) => return Err(EngineError::SchemaValidation(e.to_string())),
    }
    let h1 = graph.state_hash();
    std::fs::create_dir_all(out).map_err(|e| EngineError::Io {
        path: out.display().to_string(),
        source: e,
    })?;
    let saved = graph
        .save_to(&out.join("graph.json"))
        .map_err(|e| EngineError::SchemaValidation(e.to_string()))?;
    let reloaded = EngineGraph::load_from(&out.join("graph.json"))
        .map_err(|e| EngineError::SchemaValidation(e.to_string()))?;
    println!("assembly-demo:");
    println!("  bare_block hash : {h0}");
    println!("  assembled hash  : {h1}");
    println!("  saved sha256    : {saved}");
    println!(
        "  reload match    : {}",
        if reloaded.state_hash() == h1 {
            "true"
        } else {
            "false"
        }
    );
    println!(
        "  instances       : {} joints: {} log: {}",
        graph.instances.len(),
        graph.joints.len(),
        graph.operation_log.len()
    );
    if reloaded.state_hash() != h1 {
        return Err(EngineError::ReplayMismatch {
            expected: h1,
            actual: reloaded.state_hash(),
        });
    }
    Ok(())
}
/// Todo valor impreso deriva de constructores validados; cualquier fallo
/// de validación aborta con error estructurado (sin pánico).
fn units_demo() -> Result<(), EngineError> {
    use math::{OutOfRange, Table1};
    use units::{display, fuel::Afr, mechanics::Rpm, pressure::PressureAbs, thermal::Kelvin};

    let rpm = Rpm::from_rpm(750.0).map_err(|e| EngineError::SchemaValidation(e.to_string()))?;
    let omega = rpm.to_rad_per_s();
    let rail =
        PressureAbs::from_kpa(270.0).map_err(|e| EngineError::SchemaValidation(e.to_string()))?;
    let ambient =
        PressureAbs::from_kpa(101.325).map_err(|e| EngineError::SchemaValidation(e.to_string()))?;
    let gauge = rail.to_gauge(ambient);
    let ect = Kelvin::from_kelvin(display::celsius_to_kelvin(80.0))
        .map_err(|e| EngineError::SchemaValidation(e.to_string()))?;
    let stoich = Afr::from_ratio(14.7).map_err(|e| EngineError::SchemaValidation(e.to_string()))?;
    let lambda = Afr::from_ratio(13.2)
        .map_err(|e| EngineError::SchemaValidation(e.to_string()))?
        .to_lambda(stoich)
        .map_err(|e| EngineError::SchemaValidation(e.to_string()))?;
    let table = Table1::new("demo", &[0.0, 1.0, 2.0], &[0.0, 10.0, 30.0])
        .map_err(|e| EngineError::SchemaValidation(e.to_string()))?;
    let mid = table
        .eval(1.5, OutOfRange::Reject)
        .map_err(|e| EngineError::SchemaValidation(e.to_string()))?;

    println!("units-demo:");
    println!("  750 rpm = {:.6} rad/s", omega.as_rad_per_s());
    println!(
        "  rail 270 kPa abs @ ambient 101.325 kPa = {:.3} kPa gauge",
        gauge.as_pa() / 1_000.0
    );
    println!(
        "  ECT 80 C = {:.2} K; 270 kPa = {:.3} psi (display)",
        ect.as_kelvin(),
        display::pa_to_psi(rail.as_pa())
    );
    println!(
        "  AFR 13.2 @ stoich 14.7 -> lambda {:.6}",
        lambda.as_value()
    );
    println!("  table(1.5) = {mid:.3}");
    Ok(())
}
