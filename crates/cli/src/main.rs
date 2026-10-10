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
