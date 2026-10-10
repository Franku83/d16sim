# Reproducción independiente R01–R02

Producción objetivo: `8a7e8686e4e8831fb30d1a71b815e67e7d7139ef`. Los artefactos se entregan en `verification/r01-r02-2026-10-10`; el commit de revisión añade únicamente los dos directorios nuevos. Un checkout de producción solo no contiene estos probes.

## Ejecución completa

Desde un repositorio que conserve los objetos Git de S30, auditoría y S16 histórico:

```bash
git worktree add --detach ../d16sim-r01-r02-proof verification/r01-r02-2026-10-10
cd ../d16sim-r01-r02-proof
python3 verification/remediation_review/reproduce.py --output /tmp/d16sim-r01-r02-proof-results
```

**Exit esperado: 1**, porque se prueban contratos requeridos que siguen incumplidos. No es un fallo de compilación ni un PASS del batch. Examinar `execution-complete.json`, los dos summaries y los logs. Los probes pueden salir 0 y registrar un panic capturado/valor inválido; solo el oráculo decide si el contrato se cumple.

Requisitos: Git con objetos `ff09a0d`, `25d528e`, `2541266`, `8a7e868`; Python ≥3.12 (extracción segura tar con `filter='data'`), Rust/Cargo y dependencias disponibles en cache para las ejecuciones `--offline`. Entorno usado: Python 3.14, Rust/Cargo 1.99.0, Linux x86_64, glibc 2.44. No se prescribe el mismo kernel como requisito científico. Los lockfiles de cada harness/producer se entregan. Sin cache, descargar las dependencias bloqueadas antes de ejecutar; no cambiar los lockfiles.

El script:

1. Comprueba que `crates/`, Cargo.toml/Cargo.lock, Foundation y snapshot-contract coincidan con el commit de producción revisado. Comprueba el oráculo original contra Git, el main adaptado con exactamente cuatro campos neutros, la extracción exacta del helper y el inicializador nominal byte-idéntico.
2. Ejecuta `cargo test --locked --workspace`, `cargo fmt --all -- --check` y `cargo clippy --locked --workspace --all-targets -- -D clippy::unwrap_used`.
3. Materializa **desde Git en `target/`** los fuentes S30 y S16 histórico. No usa archivos ignorados preexistentes del implementador ni escribe en ramas/worktrees originales.
4. Compila el productor histórico con su tipo v1 verdadero, genera un snapshot caliente, ejecuta el harness contractual reconstruido, los cinco modos nuevos y sus repeticiones. Los JSONL repetidos deben ser byte-idénticos.
5. Ejecuta el nuevo oráculo, los cuatro demos mecánicos de producción y la comparación original sobre S30. Guarda argv/cwd/env/exit/duración/output.

`--skip-baseline` permite repetir los probes ampliados sin reconstruir la referencia ya ensayada; **no certifica baseline**. La evidencia entregada conserva una ejecución completa en `results/final/` y la ampliación con tres cortes/condiciones de combustible y balances nulos en `results/expanded/`. Los resultados originales de auditoría no se sobrescriben.

## Comandos puntuales

```bash
cargo test --locked --workspace
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D clippy::unwrap_used
cargo run --locked --offline --release --manifest-path verification/remediation_review/Cargo.toml --bin remediation-adversarial -- engine
cargo run --locked --offline --release --manifest-path verification/remediation_review/Cargo.toml --bin remediation-adversarial -- leak
cargo run --locked --offline --release --manifest-path verification/remediation_review/Cargo.toml --bin remediation-adversarial -- thermal
cargo run --locked --offline --release --manifest-path verification/remediation_review/Cargo.toml --bin remediation-adversarial -- snapshot
cargo run --locked --offline --release --manifest-path verification/remediation_review/Cargo.toml --bin remediation-adversarial -- boundary
```

El modo snapshot necesita `historical-v1-hot.json` en el directorio `D16_REVIEW_RESULTS`, o por defecto en `docs/remediation/review/results`; la ejecución completa lo regenera con el productor histórico. Si se ejecuta un modo aislado sin el fixture, las pruebas de autenticidad no están cubiertas; el checker falla por su ausencia.

Ejecutar los oráculos sobre resultados existentes:

```bash
D16_AUDIT_RESULTS="$PWD/docs/remediation/review/results/expanded" python3 verification/remediation_review/check_contracts.py
D16_REVIEW_RESULTS="$PWD/docs/remediation/review/results/expanded" python3 verification/remediation_review/check_adversarial.py
```

Ambos retornan 1: **10 fallos originales fuera de batch** y **10 casos nuevos incumplidos dentro de la revisión**. No se ha sustituido un resultado esperado por el valor del código corregido.

## Fuentes, fixtures y oráculos

| Artefacto | Procedencia / función |
|---|---|
| `src/main.rs` | Harness de `25d528e` + cuatro campos neutros; genera las 138 filas originales |
| `check_contracts.py` | Oráculo original byte-idéntico, incluyendo sus tolerancias |
| `reference/original_main.rs` | Bytes originales conservados sin adaptación |
| `src/common.rs` | Constructor frío idéntico al del main adaptado; no es un oráculo |
| `src/adversarial.rs` | Nuevos experimentos por APIs públicas; emite observaciones y captura panic cuando se ensaya frontera |
| `src/observed_cli.rs` | Función privada de producción extraída exacta; no se altera su lógica. El CLI real se prueba aparte |
| `check_adversarial.py` | 19 contratos independientes; invariantes, solución analítica y políticas de frontera. Fallos preservados |
| `legacy/` | Productor nuevo que importa **engine histórico** de `git archive 2541266`, con contrato/tipo originales; lockfile propio |
| `baseline/` | Main original sin adaptar y manifest con dependencias apuntando al `git archive ff09a0d` generado |
| `reproduce.py` | Reconstrucción, checks de integridad, ejecución/repetición y captura de resultados |

Invariante de especie cerrada: suma de masa fresca de todos los nodos constante cuando el regulador está aislado. La mezcla receptora se obtiene del inventario inicial y masa efectivamente transportada, no de copiar el update interno. La prueba de líquido distingue volumen analítico y masa mediante dos densidades. Los balances térmicos suman capacidades globales y fuentes/frontera; la referencia exponencial es la solución continua de un nodo aislado, no la actualización Euler del código. La química compara calor con consumo efectivo, sin asumir equivalencia a un dato experimental.

Tolerancias nuevas: 1e-12 kg para cierres de especie/masa de escenarios pequeños; 1e-12 absoluto para mezcla y relativo para líquido; 1e-7 J para balances térmicos, cuyo redondeo observado es ~1e-9 J; refinamiento térmico exige error fino <.6 del grueso; química concede 1 J pese a quemado cero. Los hashes/replay no admiten tolerancia. Las assertions paran al primer fallo por caso: el contador es **predicados ejecutados**, no assertions potenciales ni verificaciones por cada paso interno. Los máximos del inventario se observan en ventanas de .5 ms, no se afirma un registro de todos los pasos internos de esas corridas.

## Evidencia registrada

- [git-initial.json](results/git-initial.json): identidades/padres/estados iniciales.
- [production.diff](results/production.diff), [production-stat.log](results/production-stat.log): cambios reales del implementador.
- [harness-integrity.json](results/harness-integrity.json): hashes y diffs explícitos original/implementador/revisión.
- [original-audit-integrity.json](results/original-audit-integrity.json): seis documentos intactos.
- [final-integrity.json](results/final-integrity.json): ramas/estados originales preservados, SHA de sources y fixture histórico.
- [environment.json](results/environment.json): host/toolchain y ausencia de evidencia externa.
- [results/final/commands.json](results/final/commands.json): ejecución completa, incluido S30 original.
- [results/expanded/commands.json](results/expanded/commands.json): ejecución ampliada y repetición.
- [results/expanded/integrity.json](results/expanded/integrity.json): checks automáticos exactos.
- [results/expanded/independent-test-summary.json](results/expanded/independent-test-summary.json), [adversarial-test-summary.json](results/expanded/adversarial-test-summary.json): conteos ejecutados.
- Los JSONL de ambos harnesses, logs CLI y stderr del panic del fitter permanecen disponibles en esos directorios. Las rutas absolutas dentro de logs describen el host de ejecución, no una dependencia del reproductor.

Una invocación preliminar dio FileNotFoundError por usar nombres de salida diferentes de los esperados por el oráculo. Se corrigió **el runner**, conservando el checker; `original-harness-commands.json` lo registra. El resultado definitivo de 32 casos proviene de ejecuciones completas con los nombres originales `probes-*.jsonl`.

No se ha compilado un binario histórico preservado ni contrastado Windows/ARM; se usa el tipo/solver histórico con toolchain actual. No hay evidencia experimental D16Y7.
