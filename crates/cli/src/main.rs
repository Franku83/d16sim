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
    /// Demostración reproducible de historia determinista (S05).
    HistoryDemo {
        #[arg(long)]
        out: PathBuf,
        #[arg(long, default_value_t = 100)]
        sequences: u64,
    },
    /// Cinemática biela-manivela por cilindro (S06, preset ilustrativo).
    Kinematics {
        #[arg(long, default_value_t = 360.0)]
        theta720_deg: f64,
        #[arg(long, default_value_t = 750.0)]
        rpm: f64,
        #[arg(long, default_value_t = 0)]
        sweep: u32,
    },
    /// Chequeo geométrico pistón–válvula con alzada impuesta (S07).
    ContactCheck {
        #[arg(long, default_value_t = 360.0)]
        theta720_deg: f64,
        #[arg(long, default_value_t = 8.0)]
        lift_mm: f64,
        #[arg(long, default_value_t = 6.0)]
        closed_gap_mm: f64,
    },
    /// Precarga de pernos y campo de junta L1 (S08, K ilustrativo).
    FastenerSealTest {
        #[arg(long, default_value_t = 50.0)]
        torque_nm: f64,
        #[arg(long, default_value_t = 10.0)]
        dia_mm: f64,
    },
    /// Ciclo cerrado ideal sin combustión (S09, entradas ilustrativas).
    ClosedCycle {
        #[arg(long, default_value_t = 101.325)]
        p_bdc_kpa: f64,
        #[arg(long, default_value_t = 350.0)]
        t_bdc_k: f64,
        #[arg(long, default_value_t = 1.35)]
        gamma: f64,
        #[arg(long, default_value_t = 0.25)]
        step_deg: f64,
    },
    /// Ciclo con combustión Wiebe (S10, entradas ilustrativas).
    ClosedCycleBurn {
        #[arg(long, default_value_t = 340.0)]
        start_deg: f64,
        #[arg(long, default_value_t = 60.0)]
        duration_deg: f64,
        #[arg(long, default_value_t = 25.0)]
        fuel_mg: f64,
        #[arg(long, default_value_t = 1.35)]
        gamma: f64,
    },
    /// Distribución SOHC: eventos y huelgos (S11, proxy A6, sin daño).
    Valvetrain {
        #[arg(long, default_value_t = 100.0)]
        belt_teeth: f64,
        #[arg(long, default_value_t = 0.0)]
        belt_offset_teeth: f64,
        #[arg(long, default_value_t = 6.0)]
        closed_gap_mm: f64,
    },
    /// Ciclo 720° con gas exchange 0D (S12, entradas ilustrativas).
    GasExchange {
        #[arg(long, default_value_t = 750.0)]
        rpm: f64,
        #[arg(long, default_value_t = 2.0)]
        plenum_l: f64,
        #[arg(long, default_value_t = 1.5)]
        exhaust_l: f64,
        #[arg(long, default_value_t = 8.0)]
        throttle_cm2: f64,
        #[arg(long, default_value_t = 8.0)]
        tailpipe_cm2: f64,
    },
    /// Suministro de combustible L1 (S13, hipótesis etiquetadas).
    FuelSupplyTest,
    /// Motor integrado end-to-end (S16, sin clamp de RPM).
    EngineRun {
        #[arg(long, default_value = "healthy")]
        scenario: String,
        #[arg(long, default_value_t = 6.0)]
        seconds: f64,
    },
    /// Forma de onda CKP/TDC/CYP + flancos (S17, GENERIC_TRIGGER).
    TriggerWaveform {
        #[arg(long, default_value_t = 750.0)]
        rpm: f64,
        #[arg(long, default_value_t = 0)]
        fault: u8,
    },
    /// ECU de referencia: sync + comandos + DTC (S18, P2E-like no OEM).
    EcuDemo {
        #[arg(long, default_value_t = 750.0)]
        rpm: f64,
        #[arg(long, default_value_t = 0)]
        fault: u8,
    },
    /// Cadena sensorial: señal/lectura/acción ECU (S19, sin termodinámica).
    SensorDemo {
        #[arg(long, default_value = "ect_false_cold")]
        case: String,
    },
    /// Casos ambiguos con veredicto (S20, sin panel secreto).
    DiagnoseDemo {
        #[arg(long, default_value = "map")]
        case: String,
    },
    /// Calentamiento con red térmica L1 (S21, arranque en frío).
    Warmup {
        #[arg(long, default_value_t = 30.0)]
        seconds: f64,
    },
    /// Exportación de trazas solo lectura (contrato trace-bundle-v1).
    ExportTrace {
        #[arg(long, default_value = "healthy")]
        scenario: String,
        #[arg(long, default_value_t = 6.0)]
        seconds: f64,
        #[arg(long, default_value = "trace-bundle.json")]
        out: String,
    },
    /// Red de lubricación L1 (S22, sin bandera de daño).
    LubricationTest {
        #[arg(long, default_value = "healthy")]
        case: String,
    },
    /// Banco de fugas con tres escenarios (S23, ley explícita).
    LeakTest {
        #[arg(long, default_value = "underclamped_head")]
        scenario: String,
    },
    /// Protocolo compresión + leak-down con dos mediciones (S24, sin GUI).
    MechDemo {
        #[arg(long, default_value = "ring_leak_cyl3")]
        scenario: String,
    },
    /// Self-consistency del fitter + sensibilidad (S25, sintético rotulado).
    CalibrationDemo,
    /// Motor persistente 50 h sintético + acople a fugas (S26, tendencias).
    WearDemo {
        #[arg(long, default_value_t = 50.0)]
        hours: f64,
    },
    /// Mecanismos de daño con dominio (S27, riesgo vs ruptura).
    DamageDemo {
        #[arg(long, default_value = "pv_clear")]
        case: String,
    },
    /// Dinámica de cigüeñal con traza motored (S14, ilustrativo).
    CrankDynamics {
        #[arg(long, default_value_t = 750.0)]
        rpm0: f64,
        #[arg(long, default_value_t = 2.0)]
        seconds: f64,
    },
    /// Arranque arrastrado en frío (S15, sin bootstrap).
    ColdCrank,
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
        Command::HistoryDemo { out, sequences } => history_demo(&out, sequences),
        Command::Kinematics {
            theta720_deg,
            rpm,
            sweep,
        } => kinematics_cmd(theta720_deg, rpm, sweep),
        Command::ContactCheck {
            theta720_deg,
            lift_mm,
            closed_gap_mm,
        } => contact_check_cmd(theta720_deg, lift_mm, closed_gap_mm),
        Command::FastenerSealTest { torque_nm, dia_mm } => {
            fastener_seal_test_cmd(torque_nm, dia_mm)
        }
        Command::ClosedCycle {
            p_bdc_kpa,
            t_bdc_k,
            gamma,
            step_deg,
        } => closed_cycle_cmd(p_bdc_kpa, t_bdc_k, gamma, step_deg),
        Command::ClosedCycleBurn {
            start_deg,
            duration_deg,
            fuel_mg,
            gamma,
        } => closed_cycle_burn_cmd(start_deg, duration_deg, fuel_mg, gamma),
        Command::Valvetrain {
            belt_teeth,
            belt_offset_teeth,
            closed_gap_mm,
        } => valvetrain_cmd(belt_teeth, belt_offset_teeth, closed_gap_mm),
        Command::GasExchange {
            rpm,
            plenum_l,
            exhaust_l,
            throttle_cm2,
            tailpipe_cm2,
        } => gas_exchange_cmd(rpm, plenum_l, exhaust_l, throttle_cm2, tailpipe_cm2),
        Command::FuelSupplyTest => fuel_supply_test_cmd(),
        Command::EngineRun { scenario, seconds } => engine_run_cmd(&scenario, seconds),
        Command::TriggerWaveform { rpm, fault } => trigger_waveform_cmd(rpm, fault),
        Command::EcuDemo { rpm, fault } => ecu_demo_cmd(rpm, fault),
        Command::SensorDemo { case } => sensor_demo_cmd(&case),
        Command::DiagnoseDemo { case } => diagnose_demo_cmd(&case),
        Command::Warmup { seconds } => warmup_cmd(seconds),
        Command::ExportTrace {
            scenario,
            seconds,
            out,
        } => export_trace_cmd(&scenario, seconds, &out),
        Command::LubricationTest { case } => lubrication_test_cmd(&case),
        Command::LeakTest { scenario } => leak_test_cmd(&scenario),
        Command::MechDemo { scenario } => mech_demo_cmd(&scenario),
        Command::CalibrationDemo => calibration_demo_cmd(),
        Command::WearDemo { hours } => wear_demo_cmd(hours),
        Command::DamageDemo { case } => damage_demo_cmd(&case),
        Command::CrankDynamics { rpm0, seconds } => crank_dynamics_cmd(rpm0, seconds),
        Command::ColdCrank => cold_crank_cmd(),
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

/// Demostración S05: N secuencias controladas por dos caminos
/// (directo vs snapshot+recarga a mitad) sin divergencia estructural.
/// Escribe `events.jsonl` + un snapshot de ejemplo. Sin reloj del sistema.
fn history_demo(out: &std::path::Path, sequences: u64) -> Result<(), EngineError> {
    use simulation::{generate_sequence, replay_history, SnapshotFile};

    std::fs::create_dir_all(out).map_err(|e| EngineError::Io {
        path: out.display().to_string(),
        source: e,
    })?;
    let mut divergences = 0u64;
    let mut first_hash = String::new();
    let mut last_hash = String::new();
    let n = sequences.min(10_000);
    for seed in 0..n {
        let events = generate_sequence(seed, 40, 0.01);
        let mid = events.len() / 2;
        let mut direct = assembly::d16y7::bare_block_with_catalog("history-demo");
        let outcome = replay_history(&mut direct, &events)
            .map_err(|e| EngineError::SchemaValidation(format!("seed {seed} direct: {e}")))?;
        let mut via = assembly::d16y7::bare_block_with_catalog("history-demo");
        replay_history(&mut via, &events[..mid])
            .map_err(|e| EngineError::SchemaValidation(format!("seed {seed} head: {e}")))?;
        let snap = SnapshotFile::capture(&via);
        let restored = snap
            .restore_graph()
            .map_err(|e| EngineError::SchemaValidation(format!("seed {seed} restore: {e}")))?;
        let mut restored = restored;
        let tail = replay_history(&mut restored, &events[mid..])
            .map_err(|e| EngineError::SchemaValidation(format!("seed {seed} tail: {e}")))?;
        if direct.state_hash() != tail.final_hash {
            divergences += 1;
        }
        if seed == 0 {
            first_hash = outcome.final_hash.clone();
            // Artefactos de ejemplo de la seed 0.
            let mut jsonl = String::new();
            for e in &events {
                jsonl.push_str(&canonical_json(e));
                jsonl.push('\n');
            }
            std::fs::write(out.join("events.jsonl"), jsonl).map_err(|e| EngineError::Io {
                path: out.join("events.jsonl").display().to_string(),
                source: e,
            })?;
            snap.save_new(&out.join("snapshot-example.json"))?;
        }
        last_hash = outcome.final_hash;
    }
    println!("history-demo:");
    println!("  sequences     : {n}");
    println!("  divergences   : {divergences}");
    println!("  first hash    : {first_hash}");
    println!("  last hash     : {last_hash}");
    if divergences != 0 {
        return Err(EngineError::ReplayMismatch {
            expected: "zero divergences".to_owned(),
            actual: divergences.to_string(),
        });
    }
    Ok(())
}

/// Comando S06: serie cinemática `x,v,a,V,dVdθ` por cilindro.
/// `--sweep N` imprime N filas sobre 720°; si es 0, una fila al ángulo dado.
/// Preset ilustrativo (no OEM): ver `GeometryPack::illustrative_d16_like`.
fn kinematics_cmd(theta720_deg: f64, rpm: f64, sweep: u32) -> Result<(), EngineError> {
    use kinematics::{cylinder_local_angle, phases::CylinderId, GeometryPack};
    use units::mechanics::Rpm;

    let geom = GeometryPack::illustrative_d16_like()
        .map_err(|e| EngineError::SchemaValidation(e.to_string()))?;
    let rpm_v = Rpm::from_rpm(rpm).map_err(|e| EngineError::SchemaValidation(e.to_string()))?;
    let omega = rpm_v.to_rad_per_s().as_rad_per_s();
    if !(0.0..=720.0).contains(&theta720_deg) {
        return Err(EngineError::SchemaValidation(format!(
            "theta720_deg {theta720_deg} outside [0, 720]"
        )));
    }
    let cyls = [
        CylinderId::Cyl1,
        CylinderId::Cyl2,
        CylinderId::Cyl3,
        CylinderId::Cyl4,
    ];
    println!("# illustrative preset ({})", geom.source_note());
    println!("# global_deg,cyl,local_deg,stroke,x_mm,v_m_s,a_m_s2,V_cm3,dVdtheta_cm3_per_deg");
    let rows: Vec<f64> = if sweep == 0 {
        vec![theta720_deg]
    } else {
        (0..sweep)
            .map(|i| f64::from(i) * 720.0 / f64::from(sweep))
            .collect()
    };
    for g in rows {
        for cyl in cyls {
            let local = cylinder_local_angle(g, cyl);
            let crank = kinematics::phases::crank_angle_for_piston(local);
            let st = geom.state(crank.to_radians(), omega);
            let stroke = kinematics::phases::stroke_at(local);
            println!(
                "{g:.3},{cyl:?},{local:.3},{stroke:?},{:.6},{:.6},{:.3},{:.4},{:.6}",
                st.x_m * 1_000.0,
                st.v_m_s,
                st.a_m_s2,
                st.v_m3 * 1e6,
                st.dvdtheta_m3_per_rad * 1e6 * std::f64::consts::PI / 180.0,
            );
        }
    }
    Ok(())
}

/// Comando S07: huelgo pistón–válvula del cilindro 1 al ángulo global dado,
/// con alzeda impuesta (el perfil real es S11). Todo en mm de demostración.
fn contact_check_cmd(
    theta720_deg: f64,
    lift_mm: f64,
    closed_gap_mm: f64,
) -> Result<(), EngineError> {
    use contacts::{check_piston_valve, PistonValveInput};
    use kinematics::{cylinder_local_angle, phases::CylinderId, GeometryPack};

    let geom = GeometryPack::illustrative_d16_like()
        .map_err(|e| EngineError::SchemaValidation(e.to_string()))?;
    if !(0.0..=720.0).contains(&theta720_deg) {
        return Err(EngineError::SchemaValidation(format!(
            "theta720_deg {theta720_deg} outside [0, 720]"
        )));
    }
    let local = cylinder_local_angle(theta720_deg, CylinderId::Cyl1);
    let crank = kinematics::phases::crank_angle_for_piston(local);
    let x = geom.displacement(crank.to_radians());
    let report = check_piston_valve(PistonValveInput {
        closed_gap_m: closed_gap_mm / 1_000.0,
        lift_m: lift_mm / 1_000.0,
        cant_rad: 0.0,
        valve_dia_m: 0.030,
        piston_x_m: x,
    })
    .map_err(|e| EngineError::SchemaValidation(e.to_string()))?;
    println!("contact-check (cyl1, illustrative geometry, imposed lift):");
    println!("  global_deg      : {theta720_deg:.3}");
    println!("  clearance_mm    : {:.4}", report.clearance_m * 1_000.0);
    println!("  overlap         : {}", report.overlap);
    println!("  penetration_mm  : {:.4}", report.penetration_m * 1_000.0);
    println!("  needs_material  : {}", report.needs_material_model);
    println!("  damage          : {:?}", report.damage);
    Ok(())
}

/// Comando S08: 10 pernos al torque de demostración (valor ARBITRARIO, no
/// OEM) con K ilustrativo por lubricante + campo de junta uniforme.
/// Demuestra el gate: mismo torque + distinto lubricante ⇒ distinta F.
fn fastener_seal_test_cmd(torque_nm: f64, dia_mm: f64) -> Result<(), EngineError> {
    use fastener::{
        distribute_clamp, preload_of_step, LubricantCondition, SealSegment, TorqueStep,
    };

    let dia_m = dia_mm / 1_000.0;
    println!("fastener-seal-test (ARBITRARY demo torque, ILLUSTRATIVE K; not Honda):");
    let mut total_nom = 0.0;
    for (name, cond) in [
        ("dry", LubricantCondition::DryAsReceived),
        ("oiled", LubricantCondition::OiledGeneric),
        ("antiseize", LubricantCondition::AntiSeizeGeneric),
    ] {
        let est = preload_of_step(
            TorqueStep {
                torque_nm: Some(torque_nm),
                angle_rad: 0.0,
                lubricant: cond,
            },
            dia_m,
            None,
            None,
        )
        .map_err(|e| EngineError::SchemaValidation(e.to_string()))?;
        println!(
            "  {name:9} F_lo={:.0} F_nom={:.0} F_hi={:.0} N",
            est.force_lo_n, est.force_nom_n, est.force_hi_n
        );
        if name == "oiled" {
            total_nom = est.force_nom_n * 10.0;
        }
    }
    let segments = vec![
        SealSegment {
            area_m2: 400e-6,
            stiffness_weight: 1.0,
        };
        4
    ];
    let field = distribute_clamp(total_nom, &segments)
        .map_err(|e| EngineError::SchemaValidation(e.to_string()))?;
    println!("  total_clamp_N       : {total_nom:.0}");
    println!(
        "  ring_MPa            : {}",
        field
            .contact_pressure_pa
            .iter()
            .map(|p| format!("{:.2}", p / 1e6))
            .collect::<Vec<_>>()
            .join(" ")
    );
    println!("  non_uniformity      : {:.4}", field.non_uniformity);
    println!("  seal_status         : {:?}", field.status[0]);
    println!("  conductance         : Unsupported (S23)");
    Ok(())
}

/// Comando S09: ciclo cerrado ideal con auditoría y convergencia.
/// Entradas ilustrativas (no OEM): p/T en BDC, γ constante, paso angular.
fn closed_cycle_cmd(
    p_bdc_kpa: f64,
    t_bdc_k: f64,
    gamma: f64,
    step_deg: f64,
) -> Result<(), EngineError> {
    use kinematics::GeometryPack;
    use thermo::{ClosedCycle, Mixture};

    let err = |e: String| EngineError::SchemaValidation(e);
    let geom = GeometryPack::illustrative_d16_like().map_err(|e| err(e.to_string()))?;
    let mix = Mixture::new(287.0, gamma).map_err(|e| err(e.to_string()))?;
    let cycle = ClosedCycle::new(geom, mix, p_bdc_kpa * 1_000.0, t_bdc_k)
        .map_err(|e| err(e.to_string()))?;
    if !(step_deg > 0.0 && step_deg <= 2.0) {
        return Err(err(format!("step_deg {step_deg} outside (0, 2]")));
    }
    let (points, mut audit) = cycle.run(step_deg);
    let (_, order) = cycle.convergence(&[1.0, 0.5, 0.25, 0.125]);
    audit.conv_order = order;
    let ptdc = points
        .iter()
        .max_by(|a, b| a.p_pa.total_cmp(&b.p_pa))
        .map(|p| (p.theta_deg, p.p_pa))
        .unwrap_or((f64::NAN, f64::NAN));
    println!("closed-cycle (ILLUSTRATIVE inputs; verification only, not D16 validation):");
    println!(
        "  trapped_mass_mg    : {:.3}",
        cycle.trapped_mass_kg() * 1e6
    );
    println!(
        "  pmax_kPa@deg       : {:.1} @ {:.2}",
        ptdc.1 / 1_000.0,
        ptdc.0
    );
    println!("  W_comp_J (num)     : {:.4}", audit.w_comp_j);
    println!("  W_comp_J (analytic): {:.4}", audit.w_comp_analytic_j);
    println!("  W_comp_rel_err     : {:.3e}", audit.w_comp_rel_err);
    println!("  W_net_J            : {:.4e}", audit.w_net_j);
    println!("  max_p_rel_err      : {:.3e}", audit.max_p_rel_err);
    println!("  conv_order         : {:.3}", audit.conv_order);
    Ok(())
}

/// Comando S10: ciclo con Wiebe y auditoría (IMEP ilustrativo, sin dyno).
fn closed_cycle_burn_cmd(
    start_deg: f64,
    duration_deg: f64,
    fuel_mg: f64,
    gamma: f64,
) -> Result<(), EngineError> {
    use combustion::{BurnCycle, BurnParams, WallHeat, Wiebe};
    use kinematics::GeometryPack;
    use thermo::Mixture;

    let err = |e: String| EngineError::SchemaValidation(e);
    let geom = GeometryPack::illustrative_d16_like().map_err(|e| err(e.to_string()))?;
    let mix = Mixture::new(287.0, gamma).map_err(|e| err(e.to_string()))?;
    let wiebe = Wiebe::new(5.0, 2.0, start_deg.to_radians(), duration_deg.to_radians())
        .map_err(|e| err(e.to_string()))?;
    let cycle = BurnCycle::new(
        geom,
        mix,
        101_325.0,
        350.0,
        wiebe,
        BurnParams {
            fuel_mass_kg: fuel_mg * 1e-6,
            lower_heating_value_j_per_kg: 44e6,
            combustion_efficiency: 1.0,
            wall: WallHeat::Adiabatic,
        },
    )
    .map_err(|e| err(e.to_string()))?;
    let (_, audit) = cycle.run(0.25);
    println!("closed-cycle-burn (ILLUSTRATIVE inputs; verification only, not D16 validation):");
    println!("  trapped_air_mg    : {:.2}", cycle.trapped_air_kg() * 1e6);
    println!("  fuel_mg           : {fuel_mg:.2}");
    println!("  W_net_J           : {:.3}", audit.w_net_j);
    println!("  IMEP_bar (ill.)   : {:.3}", audit.imep_pa / 1e5);
    println!(
        "  pmax_kPa@deg       : {:.0} @ {:.2}",
        audit.pmax_pa / 1_000.0,
        audit.pmax_theta_deg
    );
    println!("  Q_target_J        : {:.2}", audit.q_target_j);
    println!("  Q_integrated_J    : {:.2}", audit.q_integrated_j);
    println!("  Q_rel_err         : {:.3e}", audit.q_rel_err);
    println!("  W_motored_J       : {:.4e}", audit.w_motored_j);
    Ok(())
}

/// Comando S11: eventos IO/IC/EO/EC, volumen atrapado y huelgo mínimo por
/// cilindro desde un árbol y un cigüeñal compartidos. Proxy A6 (no Y7 OEM).
fn valvetrain_cmd(
    belt_teeth: f64,
    belt_offset_teeth: f64,
    closed_gap_mm: f64,
) -> Result<(), EngineError> {
    use kinematics::GeometryPack;
    use valvetrain::{cylinder_valve_events, BeltDrive, CamSpec};

    let err = |e: String| EngineError::SchemaValidation(e);
    let geom = GeometryPack::illustrative_d16_like().map_err(|e| err(e.to_string()))?;
    let intake = CamSpec::proxy_d16a6_intake().map_err(|e| err(e.to_string()))?;
    let exhaust = CamSpec::proxy_d16a6_exhaust().map_err(|e| err(e.to_string()))?;
    let belt = BeltDrive::new(belt_teeth, belt_offset_teeth).map_err(|e| err(e.to_string()))?;
    let ev = cylinder_valve_events(
        &geom,
        &intake,
        &exhaust,
        &belt,
        closed_gap_mm / 1_000.0,
        0.5,
    )
    .map_err(|e| err(e.to_string()))?;
    println!("valvetrain (proxy D16A6 ADAPTED_PROXY, illustrative gap; shared cam+crank):");
    println!("  belt_shift_crank_deg: {:.3}", belt.cam_shift_crank_deg());
    println!("# cyl,IO,IC,EO,EC,Vtrapped_cm3,min_clear_mm@deg,overlap");
    for c in &ev {
        println!(
            "  {:?},{:.1},{:.1},{:.1},{:.1},{:.2},{:.3}@{:.1},{}",
            c.cyl,
            c.io_deg,
            c.ic_deg,
            c.eo_deg,
            c.ec_deg,
            c.trapped_volume_m3 * 1e6,
            c.min_clearance_m * 1_000.0,
            c.min_clearance_at_deg,
            c.overlap,
        );
    }
    Ok(())
}

/// Comando S12: ciclo 720° no combustivo con gas exchange 0D y auditoría.
/// Sin VE impuesta: la masa atrapada emerge del caudal por válvula.
fn gas_exchange_cmd(
    rpm: f64,
    plenum_l: f64,
    exhaust_l: f64,
    throttle_cm2: f64,
    tailpipe_cm2: f64,
) -> Result<(), EngineError> {
    use flow::{run_cycle, GasExchangeInputs};
    use kinematics::GeometryPack;
    use thermo::Mixture;
    use valvetrain::{BeltDrive, CamSpec};

    let err = |e: String| EngineError::SchemaValidation(e);
    let geom = GeometryPack::illustrative_d16_like().map_err(|e| err(e.to_string()))?;
    let mix = Mixture::new(287.0, 1.4).map_err(|e| err(e.to_string()))?;
    let intake = CamSpec::proxy_d16a6_intake().map_err(|e| err(e.to_string()))?;
    let exhaust = CamSpec::proxy_d16a6_exhaust().map_err(|e| err(e.to_string()))?;
    let belt = BeltDrive::new(100.0, 0.0).map_err(|e| err(e.to_string()))?;
    let (trace, audit) = run_cycle(
        &geom,
        mix,
        &intake,
        &exhaust,
        &belt,
        GasExchangeInputs {
            rpm,
            p_amb_pa: 101_325.0,
            t_amb_k: 293.15,
            plenum_v_m3: plenum_l * 1e-3,
            exhaust_v_m3: exhaust_l * 1e-3,
            throttle_area_m2: throttle_cm2 * 1e-4,
            tailpipe_area_m2: tailpipe_cm2 * 1e-4,
            step_deg: 0.25,
            exhaust_init: None,
        },
    )
    .map_err(|e| err(e.to_string()))?;
    println!("gas-exchange (ILLUSTRATIVE volumes/areas/Cd; no VE set):");
    println!("  trapped_mg        : {:.3}", audit.trapped_kg * 1e6);
    println!("  residual_frac     : {:.4}", audit.residual_frac);
    println!("  mass_closure      : {:.3e}", audit.mass_closure);
    println!("  energy_closure    : {:.3e}", audit.energy_closure);
    println!(
        "  p_cyl_kPa_min/max : {:.1} / {:.1}",
        audit.p_min_pa / 1_000.0,
        audit.p_max_pa / 1_000.0
    );
    println!("  trace_points      : {}", trace.len());
    Ok(())
}

/// Comando S13: cadena ralentí (contraste research) + plena carga +
/// sensibilidad a tensión + casos de fallo. Sin AFR→rendimiento sin entrega.
fn fuel_supply_test_cmd() -> Result<(), EngineError> {
    use fuel::{resolve_rail, Fuel, Injector, PulseCommand, PumpState, Regulator};

    let err = |e: String| EngineError::SchemaValidation(e);
    let fuel = Fuel::illustrative_gasoline();
    let inj = Injector::p2a_working_hypothesis();
    let reg = Regulator::new(300_000.0).map_err(|e| err(e.to_string()))?;
    let pump = PumpState {
        on: true,
        max_flow_kg_s: None,
    };
    println!("fuel-supply-test (HYPOTHESIS 185cc/min@3bar + ANALOG deadtime; not P2A-000):");
    // Ralentí ilustrativo (research v0.1 §3.2): 0.113 g aire ⇒ AFR 14.7.
    let rail = resolve_rail(reg, pump, 35_000.0, 0.002).map_err(|e| err(e.to_string()))?;
    let m_idle = 0.113e-3 / 14.7;
    let (cmd, t_eff) = inj
        .pulse_for_mass(fuel, m_idle, rail.delta_p_pa, 14.0)
        .map_err(|e| err(e.to_string()))?;
    let (d, st) = inj.deliver(fuel, cmd, rail.delta_p_pa, 14.0, None);
    println!(
        "  idle: PW_eff={:.2}ms PW_tot={:.2}ms delivered={:.2}mg status={:?}",
        t_eff * 1e3,
        cmd.width_s * 1e3,
        d.mass_kg * 1e6,
        st
    );
    // Sensibilidad a tensión: mismo comandado a 12 V entrega menos.
    let (d12, _) = inj.deliver(fuel, cmd, rail.delta_p_pa, 12.0, None);
    println!(
        "  idle@12V (same cmd): delivered={:.2}mg ({:.1}% lean)",
        d12.mass_kg * 1e6,
        100.0 * (1.0 - d12.mass_kg / d.mass_kg)
    );
    // Plena carga ilustrativa: 25 mg con ΔP 3 bar.
    let rail_wot = resolve_rail(reg, pump, 100_000.0, 0.02).map_err(|e| err(e.to_string()))?;
    let (cmd_wot, _) = inj
        .pulse_for_mass(fuel, 25e-6, rail_wot.delta_p_pa, 14.0)
        .map_err(|e| err(e.to_string()))?;
    let (dw, stw) = inj.deliver(fuel, cmd_wot, rail_wot.delta_p_pa, 14.0, None);
    println!(
        "  wot: PW_tot={:.2}ms delivered={:.2}mg status={:?} nonlinear={}",
        cmd_wot.width_s * 1e3,
        dw.mass_kg * 1e6,
        stw,
        dw.nonlinear_region
    );
    // Fallos: bomba apagada, pulso nulo, capacidad insuficiente.
    let rail_off = resolve_rail(
        reg,
        PumpState {
            on: false,
            max_flow_kg_s: None,
        },
        100_000.0,
        0.02,
    )
    .map_err(|e| err(e.to_string()))?;
    let (_, st_off) = inj.deliver(fuel, cmd_wot, rail_off.delta_p_pa, 14.0, None);
    let (_, st_sat) = inj.deliver(fuel, cmd_wot, rail_wot.delta_p_pa, 14.0, Some(0.5e-3));
    println!("  pump-off: status={st_off:?} (state, no DTC)");
    println!("  weak-pump: status={st_sat:?} (partial delivery, no DTC)");
    let _ = PulseCommand { width_s: 0.0 };
    Ok(())
}

/// Comando S14: giro con traza motored S12 + fricción ilustrativa.
/// Sin combustión: demuestra desaceleración y auditoría energética.
fn crank_dynamics_cmd(rpm0: f64, seconds: f64) -> Result<(), EngineError> {
    let err = |e: String| EngineError::SchemaValidation(e);
    // Traza motored real del simulador (acoplamiento S12→S14).
    let (trace, _) = motored_trace_for_crank().map_err(err)?;
    let th: Vec<f64> = trace.iter().map(|p| p.theta_deg).collect();
    let pp: Vec<f64> = trace.iter().map(|p| p.p_cyl_pa).collect();
    run_crank_with_trace(&th, &pp, rpm0, seconds).map_err(err)
}

/// Traza motored S12 (720°, primer ciclo, preset ilustrativo).
fn motored_trace_for_crank() -> Result<(Vec<flow::TracePoint>, flow::CycleAudit), String> {
    use flow::{run_cycle, GasExchangeInputs};
    use kinematics::GeometryPack;
    use thermo::Mixture;
    use valvetrain::{BeltDrive, CamSpec};

    let geom = GeometryPack::illustrative_d16_like().map_err(|e| e.to_string())?;
    let mix = Mixture::new(287.0, 1.4).map_err(|e| e.to_string())?;
    let intake = CamSpec::proxy_d16a6_intake().map_err(|e| e.to_string())?;
    let exhaust = CamSpec::proxy_d16a6_exhaust().map_err(|e| e.to_string())?;
    let belt = BeltDrive::new(100.0, 0.0).map_err(|e| e.to_string())?;
    run_cycle(
        &geom,
        mix,
        &intake,
        &exhaust,
        &belt,
        GasExchangeInputs {
            rpm: 750.0,
            p_amb_pa: 101_325.0,
            t_amb_k: 293.15,
            plenum_v_m3: 2.0e-3,
            exhaust_v_m3: 1.5e-3,
            throttle_area_m2: 8.0e-4,
            tailpipe_area_m2: 8.0e-4,
            step_deg: 0.5,
            exhaust_init: None,
        },
    )
    .map_err(|e| e.to_string())
}

/// Integra el giro con la traza dada e imprime la auditoría.
fn run_crank_with_trace(th: &[f64], pp: &[f64], rpm0: f64, seconds: f64) -> Result<(), String> {
    use crank::{
        effective_inertia_mean, spin_down, DynoLaw, FrictionModel, IndicatedTorque, InertiaParams,
    };
    use kinematics::GeometryPack;

    let geom = GeometryPack::illustrative_d16_like().map_err(|e| e.to_string())?;
    let ind = IndicatedTorque::from_trace(th, pp, geom.clone()).map_err(|e| e.to_string())?;
    let params = InertiaParams::new(0.15, 0.5).map_err(|e| e.to_string())?;
    let j = effective_inertia_mean(&geom, params);
    let fric = FrictionModel::ChenFlynn {
        coeffs: crank::ChenFlynnCoeffs {
            c1_pa: 30_000.0,
            c2_dlless: 0.0,
            c3_pa_s_m: 8_000.0,
            c4_pa_s2_m2: 400.0,
        },
    };
    let omega0 = rpm0 * 2.0 * std::f64::consts::PI / 60.0;
    let r = spin_down(
        &ind,
        &geom,
        fric,
        DynoLaw::Free,
        j,
        0.0,
        omega0,
        seconds,
        1e-4,
    )
    .map_err(|e| e.to_string())?;
    let om_end = r.states.last().map(|s| s.omega_rad_s).unwrap_or(f64::NAN);
    println!("crank-dynamics (ILLUSTRATIVE J/friction; S12 motored trace):");
    println!("  J_eff_kg_m2       : {j:.5}");
    println!(
        "  rpm0 -> rpm_end   : {rpm0:.1} -> {:.1}",
        om_end * 60.0 / (2.0 * std::f64::consts::PI)
    );
    println!("  W_ind_J           : {:.3}", r.w_ind_j);
    println!("  W_fric_J          : {:.3}", r.w_fric_j);
    println!("  dKE_J             : {:.3}", r.delta_ke_j);
    println!("  energy_closure    : {:.3e}", r.energy_closure);
    Ok(())
}

/// Comando S15: arranque arrastrado con batería sana + traza motored S12.
/// Sin bootstrap: el resultado informa `started: false` por diseño.
fn cold_crank_cmd() -> Result<(), EngineError> {
    use electrical::{cold_crank, Battery, Coil, ColdCrankInputs, StarterDrive, StarterMotor};
    use kinematics::GeometryPack;

    let err = |e: String| EngineError::SchemaValidation(e);
    let geom = GeometryPack::illustrative_d16_like().map_err(|e| err(e.to_string()))?;
    // Traza motored S12 como carga resistiva (misma que crank-dynamics).
    let (trace, _) = motored_trace_for_crank().map_err(err)?;
    let th: Vec<f64> = trace.iter().map(|p| p.theta_deg).collect();
    let pp: Vec<f64> = trace.iter().map(|p| p.p_cyl_pa).collect();
    let r = cold_crank(
        &geom,
        &ColdCrankInputs {
            battery: Battery::illustrative_12v(),
            starter: StarterMotor::illustrative(),
            drive: StarterDrive::new(12.0).map_err(|e| err(e.to_string()))?,
            coil: {
                let mut c = Coil::illustrative();
                c.dwell_s = 3e-3;
                c
            },
            fuel_present_kg: 25e-6,
            inertia: crank::InertiaParams::new(0.15, 0.5).map_err(|e| err(e.to_string()))?,
            friction: crank::FrictionModel::ChenFlynn {
                coeffs: crank::ChenFlynnCoeffs {
                    c1_pa: 30_000.0,
                    c2_dlless: 0.0,
                    c3_pa_s_m: 8_000.0,
                    c4_pa_s2_m2: 400.0,
                },
            },
            trace_theta_deg: th,
            trace_p_pa: pp,
            t_end_s: 2.0,
            dt_s: 1e-4,
        },
    )
    .map_err(|e| err(e.to_string()))?;
    println!("cold-crank (ILLUSTRATIVE electrics; cranks, never starts in S15):");
    println!(
        "  started           : {} ({})",
        r.started, r.no_start_reason
    );
    println!("  mean_rpm_last_rev : {:.1}", r.mean_rpm_last_rev);
    println!("  ripple_rpm        : {:.1}", r.ripple_rpm);
    println!("  E_supplied_J      : {:.1}", r.e_supplied_j);
    println!("  E_copper_J        : {:.1}", r.e_copper_j);
    println!(
        "  spark_available   : {} ({:.1} mJ)",
        r.spark.available,
        r.spark.energy_j * 1e3
    );
    println!("  mech_closure      : {:.3e}", r.mech_closure);
    Ok(())
}

/// Comando S16: motor integrado end-to-end desde reposo. El régimen final
/// se reporta tal cual (sin clamp, sin torque mágico, sin bootstrap).
fn engine_run_cmd(scenario: &str, seconds: f64) -> Result<(), EngineError> {
    use engine::{
        healthy_inputs, high_friction_inputs, no_fuel_inputs, no_spark_inputs, run_engine,
    };

    let err = |e: String| EngineError::SchemaValidation(e);
    let mut inputs = match scenario {
        "healthy" => healthy_inputs(),
        "no_spark" => no_spark_inputs(),
        "no_fuel" => no_fuel_inputs(),
        "high_friction" => high_friction_inputs(),
        _ => return Err(err(format!("unknown scenario {scenario}"))),
    };
    if !(seconds.is_finite() && seconds > 0.0 && seconds <= 30.0) {
        return Err(err("seconds in (0, 30]".to_owned()));
    }
    inputs.t_end_s = seconds;
    if let Ok(dt) = std::env::var("D16SIM_DT") {
        if let Ok(v) = dt.parse::<f64>() {
            inputs.dt_s = v;
        }
    }
    // Overrides de experimento (barridos documentados, no calibración).
    if let Ok(v) = std::env::var("D16SIM_THR_CM2") {
        if let Ok(x) = v.parse::<f64>() {
            inputs.throttle_area_m2 = x * 1e-4;
        }
    }
    if let Ok(v) = std::env::var("D16SIM_BYP_CM2") {
        if let Ok(x) = v.parse::<f64>() {
            inputs.idle_bypass_area_m2 = x * 1e-4;
        }
    }
    let trace = run_engine(&inputs).map_err(|e| err(e.to_string()))?;
    let a = trace.audit;
    println!("engine-run scenario={scenario} (ILLUSTRATIVE all inputs; emergent rpm):");
    println!("  final_rpm         : {:.1}", a.final_rpm);
    println!("  W_ind_J           : {:.1}", a.w_ind_j);
    println!("  W_fric_J          : {:.1}", a.w_fric_j);
    println!("  W_starter_J       : {:.1}", a.w_starter_j);
    println!("  W_load_J          : {:.1}", a.w_load_j);
    println!("  fuel_inj_g        : {:.3}", a.fuel_injected_kg * 1e3);
    println!("  fuel_burned_g     : {:.3}", a.fuel_burned_kg * 1e3);
    println!("  Q_rel_J           : {:.1}", a.heat_released_j);
    println!("  mass_closure      : {:.3e}", a.mass_closure);
    println!("  energy_closure    : {:.3e}", a.energy_closure);
    println!(
        "  dU_J/KE_J/Hamb_J  : {:.1} / {:.1} / {:.1}",
        a.delta_u_j, a.delta_ke_j, a.h_amb_net_j
    );
    println!("  Hfuel_J           : {:.1}", a.h_fuel_j);
    println!("  samples           : {}", trace.samples.len());
    Ok(())
}

/// Comando S17: forma de onda + conteo de flancos por canal, con fallo
/// opcional (0 sano, 1 borde CKP menos, 2 TDC abierto, 3 desfase 10°).
/// Patrón GENERIC_TRIGGER (TRIG-2026-10-10), nunca OEM.
fn trigger_waveform_cmd(rpm: f64, fault: u8) -> Result<(), EngineError> {
    use trigger::{DistributorTrigger, TriggerConfig, TriggerFault};

    let err = |e: String| EngineError::SchemaValidation(e);
    let mut cfg = TriggerConfig::generic_family();
    cfg.fault = match fault {
        0 => TriggerFault::None,
        1 => TriggerFault::MissingCkpEdge { tooth_index: 5 },
        2 => TriggerFault::OpenChannel { channel: 1 },
        3 => TriggerFault::DistributorOffset {
            offset_crank_deg: 10.0,
        },
        _ => return Err(err(format!("unknown fault {fault}"))),
    };
    let trig = DistributorTrigger::new(cfg).map_err(|e| err(e.to_string()))?;
    println!(
        "trigger-waveform ({}):",
        trigger::TriggerProvenance::generic_family().label()
    );
    for (ch, name) in [(0u8, "CKP"), (1, "TDC"), (2, "CYP")] {
        let edges = trig
            .channel_edges(ch, rpm, 0.0, 1.0, 200_000.0, 100 + u64::from(ch))
            .map_err(|e| err(e.to_string()))?;
        let first = edges.first().copied().unwrap_or(f64::NAN);
        println!(
            "  {name}: {} edges/rev-cycle, first@{first:.6}s",
            edges.len()
        );
    }
    Ok(())
}

/// Comando S18: bordes del trigger (S17) → decoder + scheduler + DTC.
/// Fallos: 0 sano, 1 borde CKP menos, 2 CYP ausente (degradado 360°),
/// 3 silencio total (pérdida + DTC tras persistencia).
fn ecu_demo_cmd(rpm: f64, fault: u8) -> Result<(), EngineError> {
    use ecu::{CkpLossMonitor, Decoder, DtcSet, Ecu, ReferenceMaps};
    use trigger::{DistributorTrigger, TriggerConfig, TriggerFault};

    let err = |e: String| EngineError::SchemaValidation(e);
    let mut cfg = TriggerConfig::generic_family();
    cfg.fault = match fault {
        0 => TriggerFault::None,
        1 => TriggerFault::MissingCkpEdge { tooth_index: 5 },
        2 => TriggerFault::OpenChannel { channel: 2 },
        3 => TriggerFault::OpenChannel { channel: 0 },
        _ => return Err(err(format!("unknown fault {fault}"))),
    };
    let trig = DistributorTrigger::new(cfg).map_err(|e| err(e.to_string()))?;
    let ckp = trig
        .channel_edges(0, rpm, 0.0, 3.0, 200_000.0, 7)
        .map_err(|e| err(e.to_string()))?;
    let cyp = trig
        .channel_edges(2, rpm, 0.0, 3.0, 200_000.0, 8)
        .map_err(|e| err(e.to_string()))?;
    let now = ckp.last().copied().unwrap_or(1.0).max(0.01);
    let dec = Decoder::new(ckp, cyp, now).map_err(|e| err(e.to_string()))?;
    let est = dec.estimate(now);
    let ecu = Ecu::new(
        ReferenceMaps::illustrative().map_err(|e| err(e.to_string()))?,
        1e-3,
        13.5,
        300_000.0,
    )
    .map_err(|e| err(e.to_string()))?;
    let mut set = DtcSet::default();
    let mut mon = CkpLossMonitor::new(3, 2, 0.5).map_err(|e| err(e.to_string()))?;
    // Tres ciclos de evaluación del monitor (persistencia temporal).
    for _ in 0..3 {
        mon.evaluate(&mut set, est.confidence, false);
    }
    let plan = ecu.schedule(&est, 0.4, &set);
    println!("ecu-demo (P2E-like reference, maps+codes NOT Honda OEM):");
    println!("  state             : {:?}", plan.ecu_state);
    println!(
        "  rpm_est/conf      : {:.0} / {:.2}",
        est.rpm, est.confidence
    );
    println!("  injections        : {}", plan.injections.len());
    println!("  ignitions         : {}", plan.ignitions.len());
    if let Some(ig) = plan.ignitions.first() {
        println!(
            "  timing req/eff    : {:.2} / {:.2} BTDC",
            ig.requested_btdc_deg, ig.effective_btdc_deg
        );
    }
    println!(
        "  dtcs              : {:?}",
        set.active.iter().map(|d| &d.code).collect::<Vec<_>>()
    );
    for line in plan.log.iter().take(4) {
        println!("  log: {line}");
    }
    Ok(())
}

/// Transductor ECT puro NTC (duplica la transferencia del canal para poder
/// inyectar el cable en medio de la cadena: documentado).
fn ect_transducer_v(celsius: f64) -> f64 {
    let t_k = celsius + 273.15;
    let r = 2000.0 * (3435.0 * (1.0 / t_k - 1.0 / 298.15)).exp();
    5.0 * r / (2000.0 + r)
}

/// Transductor MAP puro (20–250 kPa → 0.2–4.8 V, ilustrativo).
fn map_transducer_v(kpa: f64) -> f64 {
    0.2 + (kpa - 20.0) * (4.8 - 0.2) / (250.0 - 20.0)
}

/// Comando S19: cadena sensorial con casos de fallo. La columna de acción
/// ECU (enriquecimiento por ECT, carga por MAP) cambia SIN tocar ningún
/// estado termodinámico (físicos fijos en el comando): el gate se demuestra
/// por construcción + tests.
fn sensor_demo_cmd(case: &str) -> Result<(), EngineError> {
    use electrical::{RangeMonitor, SensorChannel, Transfer, Wire, WireFault};

    let err = |e: String| EngineError::SchemaValidation(e);
    // Físicos reales (fijos, termodinámicos, NO tocados por este comando).
    let ect_real_c = 90.0;
    let map_real_kpa = 35.0;

    let mut ect = SensorChannel::new(
        "ECT",
        Transfer::Ntc {
            r25_ohm: 2000.0,
            beta_k: 3435.0,
            pullup_ohm: 2000.0,
        },
        5.0,
        0.5,
        0.0,
        0.0,
        0.0,
        0.0,
        5.0,
        10,
        11,
    )
    .map_err(|e| err(e.to_string()))?;
    let mut map = SensorChannel::new(
        "MAP",
        Transfer::Linear {
            a_v_per_unit: (4.8 - 0.2) / (250.0 - 20.0),
            b_v: 0.2 - 20.0 * (4.8 - 0.2) / (250.0 - 20.0),
        },
        5.0,
        0.005,
        0.0,
        0.0,
        0.0,
        0.0,
        5.0,
        10,
        12,
    )
    .map_err(|e| err(e.to_string()))?;
    // Fallo por estado del cable/transductor según el caso.
    let mut ect_bias_c = 0.0;
    let mut wire_ect = Wire::new(100.0, 1e6, 5.0, WireFault::Ok).map_err(|e| err(e.to_string()))?;
    let mut wire_map = Wire::new(100.0, 1e6, 5.0, WireFault::Ok).map_err(|e| err(e.to_string()))?;
    match case {
        "healthy" => {}
        // ECT miente frío: sesgo del transductor (−130 °C equivalentes).
        "ect_false_cold" => {
            ect_bias_c = -130.0;
        }
        "map_open" => {
            wire_map =
                Wire::new(100.0, 1e6, 5.0, WireFault::Open).map_err(|e| err(e.to_string()))?;
        }
        "map_shorted" => {
            wire_map = Wire::new(100.0, 1e6, 5.0, WireFault::ShortToSupply)
                .map_err(|e| err(e.to_string()))?;
        }
        _ => return Err(err(format!("unknown case {case}"))),
    }

    // Cadena completa por paso: físico → transductor → cable → ADC/filtro.
    let mut ect_r = 0.0;
    let mut map_r = 0.0;
    for _ in 0..100 {
        let v_ect = wire_ect.voltage_at_adc(ect_transducer_v(ect_real_c + ect_bias_c));
        ect_r = ect.step(0.01, 0.0, Some(v_ect));
        let v_map = wire_map.voltage_at_adc(map_transducer_v(map_real_kpa));
        map_r = map.step(0.01, 0.0, Some(v_map));
    }
    // Acción ECU ilustrativa (NO toca termodinámica): enriquecimiento por
    // ECT leída + carga por MAP leído (funciones del crate, testeadas).
    use electrical::{load_from_map, warmup_enrich_from_ect};
    let enrich = warmup_enrich_from_ect(ect_r);
    let load_est = load_from_map(map_r);
    let mut mon =
        RangeMonitor::new("REF-MAP-RANGE", 0.2, 4.8, 3, 2).map_err(|e| err(e.to_string()))?;
    for _ in 0..3 {
        mon.evaluate(map_r);
    }
    println!("sensor-demo case={case} (thermo untouched: ECT=90C MAP=35kPa):");
    println!("  ect_reading_V       : {ect_r:.3}");
    println!("  map_reading_V       : {map_r:.3}");
    println!("  ecu_enrich_factor   : {enrich:.2} (from ECT reading only)");
    println!("  ecu_load_est        : {load_est:.3} (from MAP reading only)");
    println!("  map_monitor_active  : {} (REF-MAP-RANGE)", mon.active);
    Ok(())
}

/// Comando S20: dos causas, una lectura. El harness monta los escenarios
/// (conoce la verdad SOLO para evaluar); los instrumentos no la ven. Sin
/// panel secreto: el veredicto exige la medición complementaria.
fn diagnose_demo_cmd(case: &str) -> Result<(), EngineError> {
    use diagnostics::{edge_interval_variance, AmbiguityCase};
    use trigger::{DistributorTrigger, TriggerConfig, TriggerFault};

    let err = |e: String| EngineError::SchemaValidation(e);
    match case {
        "map" => {
            let healthy_v = 0.2 + (35.0 - 20.0) * (4.8 - 0.2) / (250.0 - 20.0);
            let c = AmbiguityCase::MapBiasVsLeak {
                v_biased: healthy_v + 0.5,
                v_real_rise: 0.2 + (60.0 - 20.0) * (4.8 - 0.2) / (250.0 - 20.0),
                tolerance_v: 0.05,
                ref_biased_kpa: 35.0,
                ref_real_kpa: 60.0,
                ref_tolerance_kpa: 5.0,
            };
            println!("diagnose-demo case=map (bias +0.5V vs real rise to 60kPa):");
            println!("  without reference : {:?}", c.without_complementary_test());
            println!("  with reference    : {:?}", c.with_complementary_test());
        }
        "ckp" => {
            let mut cj = TriggerConfig::generic_family();
            cj.fault = TriggerFault::Intermittent {
                channel: 0,
                seed: 41,
                drop_probability: 0.15,
            };
            let mut co = TriggerConfig::generic_family();
            co.fault = TriggerFault::DistributorOffset {
                offset_crank_deg: 10.0,
            };
            let tj = DistributorTrigger::new(cj).map_err(|e| err(e.to_string()))?;
            let to = DistributorTrigger::new(co).map_err(|e| err(e.to_string()))?;
            let ej = tj
                .channel_edges(0, 750.0, 0.0, 2.0, 200_000.0, 5)
                .map_err(|e| err(e.to_string()))?;
            let eo = to
                .channel_edges(0, 750.0, 0.0, 2.0, 200_000.0, 5)
                .map_err(|e| err(e.to_string()))?;
            let c = AmbiguityCase::CkpJitterVsOffset {
                interval_variance_a: edge_interval_variance(&ej),
                interval_variance_b: edge_interval_variance(&eo),
                separation_ratio: 10.0,
            };
            println!("diagnose-demo case=ckp (intermittent vs 10° offset):");
            println!("  without shape     : {:?}", c.without_complementary_test());
            println!("  with shape        : {:?}", c.with_complementary_test());
        }
        _ => return Err(err(format!("unknown case {case}"))),
    }
    Ok(())
}

/// Comando S21: arranque en frío con red térmica (ECT, termostato,
/// enriquecimiento). Sin calibrar: parámetros ilustrativos etiquetados.
fn warmup_cmd(seconds: f64) -> Result<(), EngineError> {
    use engine::{healthy_inputs, run_engine};
    use thermal::ThermalParams;

    let err = |e: String| EngineError::SchemaValidation(e);
    if !(seconds.is_finite() && seconds > 0.0 && seconds <= 120.0) {
        return Err(err("seconds in (0, 120]".to_owned()));
    }
    let mut inputs = healthy_inputs();
    inputs.throttle_area_m2 = 0.05e-4;
    inputs.idle_bypass_area_m2 = 0.02e-4;
    inputs.thermal = Some(ThermalParams::illustrative());
    inputs.t_end_s = seconds;
    let trace = run_engine(&inputs).map_err(|e| err(e.to_string()))?;
    let a = trace.audit;
    // ECT final = último estado térmico vía re-ejecución corta? No: la
    // traza S16 no lleva nodos; se reporta auditoría + régimen (las
    // temperaturas nodales se verifican en tests con acceso al estado).
    println!("warmup (ILLUSTRATIVE thermal params; cold start 20C):");
    println!("  final_rpm         : {:.1}", a.final_rpm);
    println!("  fuel_inj_g        : {:.3}", a.fuel_injected_kg * 1e3);
    println!("  mass_closure      : {:.3e}", a.mass_closure);
    println!("  energy_closure    : {:.3e}", a.energy_closure);
    if let Some(t) = a.thermal {
        println!(
            "  T_wall/oil/cool/ECT : {:.2}/{:.2}/{:.2}/{:.2} C",
            t.t_wall_c, t.t_oil_c, t.t_cool_c, t.ect_c
        );
        println!("  thermostat_0_1    : {:.3}", t.thermostat_0_1);
    }
    Ok(())
}

/// Comando de exportación (contrato trace-bundle-v1, solo lectura): corre
/// el escenario de forma determinista y escribe el bundle versionado sin
/// mutar nada del core.
fn export_trace_cmd(scenario: &str, seconds: f64, out: &str) -> Result<(), EngineError> {
    use engine::{
        healthy_inputs, high_friction_inputs, no_fuel_inputs, no_spark_inputs, run_engine,
    };
    use engine_types::{canonical_json, sha256_hex};

    let err = |e: String| EngineError::SchemaValidation(e);
    let mut inputs = match scenario {
        "healthy" => healthy_inputs(),
        "no_spark" => no_spark_inputs(),
        "no_fuel" => no_fuel_inputs(),
        "high_friction" => high_friction_inputs(),
        _ => return Err(err(format!("unknown scenario {scenario}"))),
    };
    if !(seconds.is_finite() && seconds > 0.0 && seconds <= 30.0) {
        return Err(err("seconds in (0, 30]".to_owned()));
    }
    inputs.t_end_s = seconds;
    let trace = run_engine(&inputs).map_err(|e| err(e.to_string()))?;
    let step = (trace.samples.len() / 300).max(1);
    let dec: Vec<&engine::TraceSample> = trace.samples.iter().step_by(step).collect();
    let t_s: Vec<f64> = dec.iter().map(|s| s.t_s).collect();
    let rpm: Vec<f64> = dec.iter().map(|s| s.rpm).collect();
    let p_plen: Vec<f64> = dec.iter().map(|s| s.p_plen_kpa).collect();
    let p_exh: Vec<f64> = dec.iter().map(|s| s.p_exh_kpa).collect();
    let fr: Vec<f64> = dec.iter().map(|s| s.fuel_rate_g_s).collect();
    let manifest = serde_json::json!({
        "scenario": scenario,
        "seconds": seconds,
        "model_pack": "physics-pack-v0.1-illustrative",
    });
    let inputs_hash = sha256_hex(canonical_json(&manifest).as_bytes());
    let bundle = serde_json::json!({
        "schema_version": 1,
        "producer": "d16sim-cli export-trace",
        "source": {"sprint": "S16", "scenario": scenario, "inputs_hash": inputs_hash},
        "series": {
            "t_s": {"unit": "s", "values": t_s},
            "rpm": {"unit": "rev/min", "values": rpm},
            "p_plen_kpa": {"unit": "kPa", "values": p_plen},
            "p_exh_kpa": {"unit": "kPa", "values": p_exh},
            "fuel_rate_g_s": {"unit": "g/s", "values": fr}
        },
        "audit": {
            "mass_closure": trace.audit.mass_closure,
            "energy_closure": trace.audit.energy_closure,
            "final_rpm": trace.audit.final_rpm
        },
        "provenance": {"model_pack": "physics-pack-v0.1-illustrative", "labels": ["ILLUSTRATIVE", "VERIFIED_NUMERICALLY", "NOT_VALIDATED"]}
    });
    let text = format!("{}\n", canonical_json(&bundle));
    std::fs::write(out, &text).map_err(|e| EngineError::Io {
        path: out.to_owned(),
        source: e,
    })?;
    println!(
        "export-trace: schema=1 points={} sha256={} -> {out}",
        t_s.len(),
        sha256_hex(text.as_bytes())
    );
    Ok(())
}

/// Comando S22: red de galerías con casos de fallo por ruta física.
/// Sin bandera de daño: el riesgo vive en película/presión, no en veredicto.
fn lubrication_test_cmd(case: &str) -> Result<(), EngineError> {
    use lube::{solve_gallery, BearingGeom, FrictionLink, LubePumpState, OilGrade, Pump};

    let err = |e: String| EngineError::SchemaValidation(e);
    let walther = OilGrade::illustrative_10w30()
        .walther()
        .map_err(|e| err(e.to_string()))?;
    let mut pump = LubePumpState {
        pump: Pump::illustrative(),
        oil_level_01: 1.0,
    };
    let mut temp_c = 80.0;
    let mut main_clear = 0.030e-3;
    let rpm = 3000.0;
    match case {
        "healthy" => {}
        // Aceite insuficiente: nivel bajo ⇒ aireación.
        "low_oil" => pump.oil_level_01 = 0.15,
        // Viscosidad baja en caliente.
        "hot_thin" => temp_c = 130.0,
        // Holgura excesiva en bancada 1.
        "excess_clearance" => main_clear = 0.060e-3,
        // Bomba defectuosa: eficiencia al 50%.
        "weak_pump" => pump.pump.efficiency = 0.5,
        _ => return Err(err(format!("unknown case {case}"))),
    }
    let mu = walther.mu_pa_s(temp_c);
    let mains = BearingGeom::new(0.055, 0.020, main_clear).map_err(|e| err(e.to_string()))?;
    let rods = BearingGeom::new(0.045, 0.018, 0.029e-3).map_err(|e| err(e.to_string()))?;
    let mut bearings = Vec::new();
    for _ in 0..5 {
        bearings.push((mains, 5e6, rpm));
    }
    for _ in 0..4 {
        bearings.push((rods, 8e6, rpm));
    }
    let r = solve_gallery(&pump, rpm, mu, 101_325.0, &bearings, 1e5, 300e3)
        .map_err(|e| err(e.to_string()))?;
    let link = FrictionLink { k_starve: 2.0 };
    let worst_film = r.branches.iter().map(|b| b.film_0_1).fold(1.0f64, f64::min);
    println!("lubrication-test case={case} (ILLUSTRATIVE oil/pump/bearings):");
    println!(
        "  gallery_kPa_abs       : {:.1}",
        r.gallery_p_pa_abs / 1_000.0
    );
    println!("  relief_L_min          : {:.3}", r.relief_m3_s * 60_000.0);
    println!("  pump_heat_W           : {:.1}", r.pump_heat_w);
    println!("  worst_film_0_1        : {:.3}", worst_film);
    println!(
        "  friction_multiplier   : {:.3} (route, never a damage flag)",
        link.multiplier(worst_film)
    );
    Ok(())
}

/// Comando S23: banco de leak-down estático por escenario. Sin
/// multiplicadores de potencia: solo caudales, reparto y auditoría.
fn leak_test_cmd(scenario: &str) -> Result<(), EngineError> {
    use leak::{leak_scenario, pseal_sensitivity, run_leakdown, LeakScenario};

    let err = |e: String| EngineError::SchemaValidation(e);
    let s = match scenario {
        "missing_head_gasket" => LeakScenario::MissingHeadGasket,
        "underclamped_head" => LeakScenario::UnderclampedHead,
        "ring_leak_cyl3" => LeakScenario::RingLeakCyl3,
        _ => return Err(err(format!("unknown scenario {scenario}"))),
    };
    let net = leak_scenario(s).map_err(|e| err(e.to_string()))?;
    // Paso fino: las fugas grandes exigen dt pequeño (fail-closed si no).
    let (_, audit, totals) = run_leakdown(&net, 0, 10.0, 5e-4).map_err(|e| err(e.to_string()))?;
    let tot: f64 = totals.iter().sum();
    println!("leak-test scenario={scenario} (UNCALIBRATED law, NOT a prediction):");
    println!(
        "  steady_leak_g_min   : {:.3}",
        audit.steady_leak_g_s * 60.0
    );
    println!(
        "  path_split          : {}",
        totals
            .iter()
            .map(|t| format!("{:.2}", t / tot.max(1e-30)))
            .collect::<Vec<_>>()
            .join(" ")
    );
    println!("  mass_closure        : {:.3e}", audit.mass_closure);
    println!("  energy_closure      : {:.3e}", audit.energy_closure);
    println!("  pseal_sensitivity   : {:?}", pseal_sensitivity());
    Ok(())
}

/// Comando S24: protocolo `diagnose_low_compression` cableado a los
/// escenarios S23. El compresímetro usa el área efectiva del escenario +
/// modelo de cranking L1; el leak-down usa su reparto. Sin GUI, sin flags.
fn mech_demo_cmd(scenario: &str) -> Result<(), EngineError> {
    use diagnostics::{
        cranking_peak_kpa, diagnose_low_compression, CompressionGauge, LeakDownTester, LeakSplit,
        Provenance, C_CRANK_ILLUSTRATIVE,
    };
    use leak::{leak_scenario, run_leakdown, LeakScenario};

    let err = |e: String| EngineError::SchemaValidation(e);
    let s = match scenario {
        "healthy_baseline" => LeakScenario::HealthyBaseline,
        "missing_head_gasket" => LeakScenario::MissingHeadGasket,
        "underclamped_head" => LeakScenario::UnderclampedHead,
        "ring_leak_cyl3" => LeakScenario::RingLeakCyl3,
        _ => return Err(err(format!("unknown scenario {scenario}"))),
    };
    let net = leak_scenario(s).map_err(|e| err(e.to_string()))?;
    let (_, audit, totals) = run_leakdown(&net, 0, 10.0, 5e-4).map_err(|e| err(e.to_string()))?;
    // Área efectiva de la CÁMARA (solo puertos con from == 0: el
    // respiradero del cárter NO es fuga de la cámara; bug detectado en la
    // primera versión del cableado, que lo sumaba y colapsaba el pico).
    let tot: f64 = totals.iter().sum();
    let a_total_m2: f64 = net
        .ports
        .iter()
        .filter(|p| p.from == 0)
        .map(|p| p.conductance)
        .sum();
    // Reparto = OBSERVACIÓN del técnico (dónde se escucha/ve escapar el
    // aire: tapón de aceite / cola de escape / vaso de expansión), provista
    // por el harness como en S20 (los instrumentos nunca ven el ground
    // truth). Camino dominante físico por escenario, documentado.
    let split = match s {
        LeakScenario::HealthyBaseline | LeakScenario::RingLeakCyl3 => LeakSplit {
            to_crankcase_01: 1.0,
            to_exhaust_01: 0.0,
            to_coolant_01: 0.0,
        },
        // Subapretada a 500 kPa estáticos: silba por el tapón de aceite
        // (misma firma que anillos → ambigüedad documentada, exige 3er
        // test: reapriete + retest o química del refrigerante).
        LeakScenario::UnderclampedHead => LeakSplit {
            to_crankcase_01: 1.0,
            to_exhaust_01: 0.0,
            to_coolant_01: 0.0,
        },
        // Sin junta: paso abierto cámara→agua, burbujeo violento en vaso.
        LeakScenario::MissingHeadGasket => LeakSplit {
            to_crankcase_01: 0.30,
            to_exhaust_01: 0.10,
            to_coolant_01: 0.60,
        },
    };
    let peak_true = cranking_peak_kpa(
        1500.0,
        1.0,
        400e-6,
        8e-6,
        a_total_m2,
        200.0,
        4,
        C_CRANK_ILLUSTRATIVE,
    )
    .map_err(|e| err(e.to_string()))?;
    let gauge = CompressionGauge {
        dead_vol_m3: 8e-6,
        resolution_kpa: 5.0,
        zero_error_kpa: 0.0,
    };
    let prov = Provenance::new(
        "compression-gauge-A",
        "CAL-24-01",
        "op",
        "2026-10-10",
        "WOT cranking 200 rpm",
    )
    .map_err(|e| err(e.to_string()))?;
    let comp = gauge
        .read(peak_true, prov.clone())
        .map_err(|e| err(e.to_string()))?;
    let tester = LeakDownTester {
        supply_kpa: 500.0,
        orifice_m2: 1e-6,
        position: "TDC compression".to_owned(),
    };
    let (_, leak_pct) = tester
        .read(a_total_m2, prov.clone())
        .map_err(|e| err(e.to_string()))?;
    let verdict =
        diagnose_low_compression(Some(comp.value), 900.0, Some(leak_pct.value), Some(split));
    println!("mech-demo scenario={scenario} (L1 instruments, UNCALIBRATED):");
    println!(
        "  steady_leak_g_min   : {:.3}",
        audit.steady_leak_g_s * 60.0
    );
    println!("  compression_kPa     : {:.0}", comp.value);
    println!("  leak_pct            : {:.1}", leak_pct.value);
    println!(
        "  split crank/exh/cool: {:.2}/{:.2}/{:.2}",
        split.to_crankcase_01, split.to_exhaust_01, split.to_coolant_01
    );
    println!("  verdict             : {verdict:?}");
    let _ = tot;
    Ok(())
}

/// Comando S25: verifica la maquinaria de calibración con datos
/// SYNTHETIC_SELFTEST (recupera C y p_ideal del cranking + ranking de
/// Morris) y muestra el estado del manifiesto. NO valida nada: sin banco
/// real el cierre es NUMERICALLY_VERIFIED_ONLY.
fn calibration_demo_cmd() -> Result<(), EngineError> {
    use calibration::{calibration_set, least_squares, local_elasticities, morris_effects};

    let err = |e: String| EngineError::SchemaValidation(e);
    let c_true = 8.0e5;
    let p_true = 1500.0;
    let areas = [1e-9, 0.5e-6, 1.0e-6, 2.0e-6, 4.0e-6, 8.0e-6];
    let model = |p: &[f64], a: f64| p[1] * 400e-6 / (400e-6 + 8e-6) / (1.0 + p[0] * a * 0.6 * 4.0);
    let y: Vec<f64> = areas.iter().map(|a| model(&[c_true, p_true], *a)).collect();
    let w = vec![1.0; y.len()];
    let f = |p: &[f64]| areas.iter().map(|a| model(p, *a)).collect::<Vec<_>>();
    let r = least_squares(f, &y, &w, &[2.4e6, 1800.0], 1e-9).map_err(|e| err(e.to_string()))?;
    let f1 = |p: &[f64]| model(p, 2e-6);
    let el = local_elasticities(f1, &[c_true, p_true], &["crank_c_leak", "crank_p_ideal"]);
    let mo = morris_effects(
        |p: &[f64]| model(p, 2e-6),
        &["crank_c_leak", "crank_p_ideal"],
        &[(1e5, 5e6), (800.0, 2200.0)],
        4,
    );
    let set = calibration_set();
    let nid = set.iter().filter(|p| p.identifiable).count();
    println!("calibration-demo (SYNTHETIC_SELFTEST: verifies machinery, validates NOTHING):");
    println!(
        "  recovered C         : {:.3e} (true {:.3e})",
        r.params[0], c_true
    );
    println!(
        "  recovered p_ideal   : {:.1} kPa (true {p_true:.1})",
        r.params[1]
    );
    println!(
        "  std_k1              : {:.3e} {:.3e}",
        r.std_k1[0], r.std_k1[1]
    );
    println!("  max_corr            : {:.3}", r.max_abs_correlation);
    println!("  elasticities        : {el:.3?}");
    println!("  morris_rank         : {} > {}", mo[0].0, mo[1].0);
    println!("  identifiable/blocked: {nid}/{}", set.len() - nid);
    println!("  dataset status      : NO FIELD DATA — NUMERICALLY_VERIFIED_ONLY (P1 blocked)");
    Ok(())
}

/// Comando S26: 50 h sintéticas etiquetadas + acople desgaste→fuga.
/// Tendencias condicionadas, jamás vida útil; lo no soportado se lista.
fn wear_demo_cmd(hours: f64) -> Result<(), EngineError> {
    use wear::{OperatingPoint, ServiceLog, WearLaw, WearState};

    let err = |e: String| EngineError::SchemaValidation(e);
    if !(hours.is_finite() && hours > 0.0 && hours <= 200.0) {
        return Err(err("hours in (0, 200]".to_owned()));
    }
    let op = OperatingPoint {
        rpm: 3000.0,
        load_01: 0.6,
        oil_temp_c: 100.0,
        coolant_temp_c: 88.0,
        film_0_1: 0.8,
        ring_pressure_pa: 5e6,
        bearing_pressure_pa: 5e6,
    };
    let (rl, bl, ol) = (
        WearLaw::archard_ring(),
        WearLaw::bearing_starvation(),
        WearLaw::oil_aging(),
    );
    let mut s = WearState::new_seeded(7);
    let steps = (hours * 2.0).ceil() as usize;
    let dt = hours / steps as f64;
    for _ in 0..steps {
        s.advance(dt, &op, &rl, &bl, &ol)
            .map_err(|e| err(e.to_string()))?;
    }
    let a0 = s.ring_leak_area_m2(0, 3e-3, 0.03);
    println!("wear-demo {hours:.0}h SYNTHETIC (TREND_CONDITIONED, not life):");
    println!("  ring_gap_um[0]      : {:.1}", s.ring_gap_m[0] * 1e6);
    println!("  liner_wear_um[0]    : {:.2}", s.liner_wear_m[0] * 1e6);
    println!("  bearing_extra_nm[0] : {:.1}", s.bearing_extra_m[0] * 1e9);
    println!("  oil_oxidation       : {:.3}", s.oil_oxidation_01);
    println!("  gasket_relax        : {:.4}", s.gasket_relax_01);
    println!(
        "  ring_leak_mm2[0]    : {:.4} (feeds S23 annulus law, no flag)",
        a0 * 1e6
    );
    println!("  state_hash          : {}", s.hash());
    let mut log = ServiceLog::new();
    log.apply(&mut s, "oil_change", "10W30", "op")
        .map_err(err)?;
    println!(
        "  post_service_oil    : {:.3} (reset logged, parts untouched)",
        s.oil_oxidation_01
    );
    println!("  UNSUPPORTED         : scuffing, contact fatigue, thermal fretting (no law)");
    Ok(())
}

/// Comando S27: mecanismos con dominio. Riesgo ≠ ruptura; la trayectoria
/// nunca cambia (chequeos puros).
fn damage_demo_cmd(case: &str) -> Result<(), EngineError> {
    use damage::{bearing_starvation_risk, bolt_yield_state, piston_valve_state, thread_state};
    use fastener::preload::LubricantCondition;

    let err = |e: String| EngineError::SchemaValidation(e);
    let out = match case {
        "pv_clear" => {
            let (_, s) = piston_valve_state(contacts::interference::PistonValveInput {
                closed_gap_m: 3e-3,
                lift_m: 8e-3,
                cant_rad: 0.0,
                valve_dia_m: 30e-3,
                piston_x_m: 6e-3,
            })
            .map_err(|e| err(e.to_string()))?;
            format!("{s:?}")
        }
        "pv_contact" => {
            let (c, s) = piston_valve_state(contacts::interference::PistonValveInput {
                closed_gap_m: 3e-3,
                lift_m: 8e-3,
                cant_rad: 0.0,
                valve_dia_m: 30e-3,
                piston_x_m: 0.5e-3,
            })
            .map_err(|e| err(e.to_string()))?;
            format!("overlap={} {s:?}", c.overlap)
        }
        "bolt_ok" => {
            let (st, _) = bolt_yield_state(60.0, 10e-3, 58e-6, LubricantCondition::OiledGeneric)
                .map_err(|e| err(e.to_string()))?;
            format!("{st:?}")
        }
        "bolt_plastic" => {
            let (st, _) = bolt_yield_state(400.0, 10e-3, 58e-6, LubricantCondition::OiledGeneric)
                .map_err(|e| err(e.to_string()))?;
            format!("{st:?} (GENERIC material, not D16Y7)")
        }
        "bearing_risk" => format!(
            "{:?}",
            bearing_starvation_risk(0.2, 8e6).map_err(|e| err(e.to_string()))?
        ),
        "thread" => {
            let (m, s) = thread_state();
            format!("{:?} {s:?}", m.support)
        }
        _ => return Err(err(format!("unknown case {case}"))),
    };
    println!("damage-demo case={case}: {out}");
    Ok(())
}
