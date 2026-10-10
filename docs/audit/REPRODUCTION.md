# Reproducción de la auditoría

Commit de producción auditado: `ff09a0dda82188e6bf27a80d17c902f8972a39d2`. Rama: `audit/independent-2026-10-10`. Worktree utilizado: `/home/franku/projects/d16sim-audit`. Fecha: 2026-10-10. Hardware/toolchain/OS: [environment.json](results/environment.json). Un host Linux x86_64, Rust/Cargo 1.99.0; no evidencia de Windows/ARM/segundo host.

Los cambios son adiciones en `audit/independent/` y `docs/audit/`. El crate de probes tiene su propio `[workspace]`, lockfile y `target`; importa APIs existentes por `path` sin parchearlas. No se cambiaron parámetros de producción, Foundation, modelos o gates. Las perturbaciones explícitas viven en los fixtures y no se presentan como calibración. Los fixtures son `SYNTHETIC_SELFTEST`.

## Ejecución mínima independiente

Desde la raíz del checkout de auditoría:

```bash
bash audit/independent/reproduce.sh
```

El script crea `target/independent-audit-results/`, compila el crate aislado con `--release --locked --offline`, ejecuta modos engine/subsystems/dyno y aplica los 32 contratos Python. **En la base auditada el exit final esperado es 1: 23 failures, 0 errors, 9 casos que pasan.** Un exit 0 del generador solo indica que produjo observaciones; no constituye PASS científico. No se invierte ni silencia el exit del checker. Si build/probe falla, el script se detiene antes del checker y sus logs permiten distinguir fallo de entorno/harness.

Cargo requiere dependencias del lock cacheadas para `--offline`. En entorno limpio puede hacerse prefetch con acceso a registro antes del ensayo:

```bash
cargo fetch --manifest-path audit/independent/Cargo.toml --locked
```

Para regenerar las observaciones en otra carpeta, dar un path absoluto como único argumento al script. Los resultados archivados de esta entrega están en `docs/audit/results/`; el directorio por defecto evita sobrescribirlos. El checker acepta `D16_AUDIT_RESULTS` para leer otro directorio. Ejemplo de un solo caso sobre resultados archivados:

```bash
python3 -m unittest audit.independent.check_contracts.Contracts.test_11_replay_includes_pressure_history
```

## Comandos explícitos y baseline de producción

Los siguientes comandos fueron ejecutados; logs archivados. Los comandos de probes equivalen al script anterior, pero escriben directamente en el archivo indicado:

```bash
cargo test --locked --workspace > docs/audit/results/workspace-test.log 2>&1
cargo fmt --all -- --check > docs/audit/results/format.log 2>&1
cargo clippy --locked --workspace --all-targets -- -D clippy::unwrap_used > docs/audit/results/clippy.log 2>&1
cargo metadata --locked --offline --no-deps --format-version 1 > docs/audit/results/cargo-metadata.json
cargo tree --locked -e features -i serde_json > docs/audit/results/serde-json-features.log
cargo build --manifest-path audit/independent/Cargo.toml --locked --offline --release
cargo run --manifest-path audit/independent/Cargo.toml --locked --offline --release -- engine > docs/audit/results/probes-engine.jsonl 2> docs/audit/results/probes-engine.stderr
cargo run --manifest-path audit/independent/Cargo.toml --locked --offline --release -- subsystems > docs/audit/results/probes-subsystems.jsonl
cargo run --manifest-path audit/independent/Cargo.toml --locked --offline --release -- dyno > docs/audit/results/probes-dyno.jsonl 2> docs/audit/results/probes-dyno.stderr
python3 audit/independent/check_contracts.py > docs/audit/results/independent-contract-tests.log 2>&1
python3 audit/independent/inventory.py
```

El baseline pasó 163 casos existentes, 30 targets unitarios (uno con cero casos) y 29 targets doc-test con cero casos. No se contaron assertions ejecutadas del baseline. El inventario identifica 582 sitios `assert!`/`assert_eq!`/`assert_ne!` estáticos: no son 582 assertions ejecutadas ni mecanismos físicos independientes. Clippy salió 0 con el lint exigido por el CI; contiene warnings, no se afirma «cero warnings».

El ejecutable independiente es un target binario, no una suite Rust de tests. Su ejecución produce 138 filas JSONL agrupadas en 42 familias; algunas contienen muchos pasos, ventanas o subcasos. La suite Python es un target de pruebas con 32 casos y 88 predicados `check` realmente alcanzados (algunos casos se interrumpen al fallar). Esas tres cifras no son sinónimos ni una medida de cobertura física. [Resumen ejecutado](results/independent-test-summary.json), [log contractual](results/independent-contract-tests.log), [inventario](results/inventory.json).

## Mapa de pruebas y hallazgos

| Familia y fixture | Archivo de resultados | Contrato / hallazgo |
|---|---|---|
| engine_refinement: idle/base ×dt200/100/50/25µs ×3/8/30s; idle_finer: 12.5/6.25µs a8s | probes-engine.jsonl | Masa total (caso3 pasa), sensibilidad G02; refinamiento de RPM final separado de media de cola |
| thermal_long: cold idle120s a100/50/25µs; thermal_boundary: 100°C, .1s | engine/subsystems | F10 balance incompleto; identidad standalone cuenta calor al ambiente; no es medida real de warmup |
| engine_negative: no_fuel/no_spark/high_friction; fuel_cut a3s hasta8s | engine | Caso5 pasa; causalidad negativa local, no estabilidad general |
| fuel_outflow homogéneo y fuel_inventory no chispa4s | engine | F01, caso10 rojo; expected independiente de mezcla homogénea y cota de especies |
| cam_integrated: −180/−30/−6/0/+6/+30/+180/+720°; cam_flow a.25/.125° | engine | Caso4 pasa, F16/caso28 rojo; standalone cam_flow no reemplaza integración |
| cam_event_chain: −6/0/+6° en ventana2–2.5s | engine | Cierres IVC, masa atrapada/latch/Q/work/RPM, K02/G04. Inyección también se desplaza; no aislamiento exclusivo del aire |
| snapshot_json; replay_split c2=0/.05; v1_hash_migration; restore invalid/tamper/schema | engine | F05/F06/F07/F15; casos11/12/14/29 rojos y9 pasa |
| leak_scenario: cuatro redes ×dt500/250µs,10s; leak_species; leak_liquid_units | subsystems | F02/F08/F09; casos15/16 rojos; comparar split físico con CLI |
| diagnosis_multicause | subsystems | R01 sospecha; clasificador reproducido, causa física múltiple incompleta |
| wear_grid: film0/.8/1 ×T40/100/130,50h contra5000×.01h; unsupported/irreversible/roundtrip | subsystems | Caso6 pasa; F05/F11/F12, casos13/19/20 rojos |
| damage_bolt60/80/400Nm; damage_geometry; damage_support | subsystems | Caso7 pasa, alcance K03; no rotura material inferida |
| decoder origin/recovery/CYP dropout/invalid; wire_seed1/2; scheduler | subsystems | F13/F14/F15 y G06; casos21/22/23/30 rojos,8 pasa |
| analytic_cylinder: sin calor, RK4 2/1/.5/.25°, invariant pV^γ; flow_metamorphic | subsystems | Casos1/2 pasan: referencia analítica/invariancia independiente, no repite solver |
| thermal_invalid/dyno_invalid/dataset_invalid | subsystems | F15 casos25–27 rojos |
| fit_analytic: f=p²,y4,w1,p0=3/6/2 | subsystems | F18/F19 casos31/32 rojos; referencia stdp=1/(2p), no replica algoritmo Gauss–Newton |
| dyno_public: load4/6/8; dyno_sampling: mismos inputs cada400/40/4/1pasos,dt50µs,6s | dyno | F03/F04 casos17/18 rojos; promedios y balances por ventanas explícitas |

`cam_flow.intake_signed_sum` es suma discreta de muestras de caudal, **no una integral temporal**; crece al duplicar muestras. No se utilizó como prueba de convergencia del caudal. `trapped_kg` de ese solver es observación standalone; `cam_event_chain.mean_trapped_total_kg` observa el estado del motor al cruce IVC. La inicialización `cold` replica condiciones declaradas para comparar escenarios; no constituye referencia independiente. La referencia independiente se utiliza en invariantes, unidades, identidades y soluciones analíticas.

## CLI, controles CI y tipos históricos

[cli-commands.json](results/cli-commands.json) conserva argv exactos, exit y log de 15 ejecuciones: validate-data, release-check, mech-demo dos escenarios, sensor-demo tres fallos, ecu-demo, wear-demo, damage-demo, dyno-demo, init/run-scenario/replay y export-trace. Todos salieron 0; ello no significa que sus inferencias estén verificadas. Para repetir los principales:

```bash
cargo run --locked -p d16sim -- release-check
cargo run --locked -p d16sim -- mech-demo --scenario missing_head_gasket
cargo run --locked -p d16sim -- sensor-demo --case map_open
cargo run --locked -p d16sim -- dyno-demo
```

Runner init/run/replay produce `runner_plumbing_only` y `physics-pack-v0.0-none`: no es replay del estado físico S16. Los artefactos originales runner/trace se preservan junto a logs. F17 se reproduce con las dos expresiones del workflow, registradas en docs-ci-controls.json; ambas salen 1, sin afirmar ejecución de GitHub Actions.

El tipo v1 histórico se recuperó de la versión anterior a S21:

```bash
git show 2541266:crates/engine/src/state.rs
```

[Tipo archivado](results/historical-engine-state-v1.rs). El probe usa el mismo orden/campos de v1 (sin thermal) y hash de esos bytes, evitando confundir F06 con la pérdida decimal F05. El replay F07 también usa snapshot en memoria, no JSON, y c2=0 como control.

47 filas comunes de subsystems son exactamente iguales entre debug y release: [comparación](results/debug-release-comparison.json). El control debug es una versión previa del harness sin los tres casos fit_analytic; se usa solo para esas familias comunes, no se cuenta como 47 tests nuevos. La repetición completa por el script y su comparación se registran en `results/reproduction-comparison.json`.

## Integridad y aislamiento

`python3 audit/independent/check_package.py` comprueba campos, referencias y aislamiento; no evalúa corrección física. `results/production-isolation.json` compara todos los paths ya versionados contra la base y registra estado de rama/worktree principal. `SHA256SUMS` incluye entregables, código de auditoría y resultados seleccionados; no prueba corrección científica, solo integridad de este paquete. Desde raíz:

```bash
sha256sum -c docs/audit/SHA256SUMS
```

El informe y su catálogo se regeneran con `python3 audit/independent/render_findings.py`; ese script solo escribe FINDINGS dentro de auditoría. Los números de líneas de producción corresponden al commit base. No se modificaron benchmarks/umbrales/gates para conseguir un PASS. No hubo merge a master, corrección de bugs ni inicio de S31.
