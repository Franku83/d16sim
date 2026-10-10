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
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("d16sim: error: {e:#}");
        std::process::exit(1);
    }
}
