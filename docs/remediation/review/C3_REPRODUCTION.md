# Reproducción independiente C3

Objetivo fijo: **3aef5f6ccdc076dad6e8be627a5eb836b26ee998**. Revisor: rama `verification/r01-r02-2026-10-10`, con producción8a sin modificar; no ejecutar directamente los probes allí como si sus crates fueran C3. Los runners crean/usan un worktree detached exacto C3 y extraen solamente fuentes de auditoría rastreadas en Git.

## Reconstrucción completa desde Git

Requisitos: Git con los commits históricos, Python3 con tarfile filter=data, Rust/Cargo y las dependencias de los lockfiles disponibles para --offline. Plataforma/toolchain/fecha real de revisión en [environment.json](c3-results/environment.json). No hay fixtures ni fuentes de producción ignoradas preexistentes necesarias; las copias históricas se reconstruyen con git archive.

Desde la rama revisora al commit final de entrega:

```bash
python3 verification/c3_review/run_c3.py \
  --source-ref verification/r01-r02-2026-10-10 \
  --worktree ../d16sim-c3-proof-nuevo \
  --output /tmp/d16-c3-previos-nuevos

python3 verification/c3_review/run_extra.py \
  --source-ref verification/r01-r02-2026-10-10 \
  --worktree ../d16sim-c3-proof-nuevo \
  --output /tmp/d16-c3-extension-c2-nueva

python3 verification/c3_review/run_directed.py \
  --source-ref verification/r01-r02-2026-10-10 \
  --worktree ../d16sim-c3-proof-nuevo \
  --output /tmp/d16-c3-dirigidos-nuevos
```

Exits esperados: **0,0,1**. El primer wrapper conserva el exit1 del oráculo original(10 rojos R03/R04) y concluye0 al reproducirlo junto a19/19 y15/15. El segundo reproduce C2 11/11. El tercero completa11 casos dirigidos con4 fallos confirmados y0 errores de ejecución. Ningún wrapper integra, autoriza otro batch o modifica producción. Las salidas deben ser rutas nuevas vacías; no se sobrescribe ninguna corrida.

El primer runner ejecuta los comandos exigidos íntegros:

```bash
cargo test --locked --workspace
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D clippy::unwrap_used
```

También S30 Rust/referencia original, todos los modos de los tres harnesses previos, cinco repeticiones deterministas, cuatro CLI y productores genuinos. `--skip-baseline` solo omite la referencia ya documentada; no la certifica. La extensión C2 reconstruye los cinco productores por SHA y25 snapshots sin alterar sus expectativas. run_directed ejecuta seis modos nuevos y contrafactuales con **idéntico source** compilado contra crates C2 archivados desde Git.

## Oráculos, adaptadores e integridad

[oracle-integrity.json](c3-results/oracle-integrity.json) y [full-harness-integrity.json](c3-results/full-harness-integrity.json) contrastan bytes y SHA256:

| Oráculo | Referencia original | SHA256 |
|---|---|---|
| check_contracts.py | 25d528e:audit/independent/check_contracts.py | b027d9e3ab697b1974e87980765b7b4872e24e3792153ec3e02b598bd0756b36 |
| check_adversarial.py | 1d61437:verification/remediation_review/check_adversarial.py | d67f74f3c560a4bee17d703734a2b48ac5e88d0624351e51dfef5356e59f5b9b |
| check_c1_controls.py | a0088ff:verification/c1_review/check_c1_controls.py | b021e634c12c47074cb65d138eed083e544c2323eabcd19f5b372df34c914033 |
| check_c2.py | 40e75d5:verification/c2_review/check_c2.py | ver hash completo en oracle-integrity.json |

Los sources main/adversarial/common/c1_controls/c2_boundaries son byte-idénticos a40e75d5. Main original conserva solo los cuatro campos neutros de inicialización R01 ya auditados, sin cambios C3. Helper observacional de CLI se extrae byte-exacto desde C3, incluido su newline original. La alerta de blank line final del helper en git diff --check no autoriza recortarlo; cargo fmt productivo pasó.

Cambios nuevos: configuración de target C3/rutas separadas, nuevo bin c3-directed y checker **separado**, runner dirigido y control C2 adicional. En producer_c2 se agregan exclusivamente dependencias directas ya presentes transitivamente y un bin que compila el mismo source dirigido contra C2; su productor histórico original no se cambia. Lockfiles se entregan rastreados. No se modifican los cuatro oráculos ni los informes y resultados de40e75d5.

El fixture `fixtures/genuine-v1-cold.json` viene byte-exacto de40e75d5:c2-results/run-01/historical-v1-cold.json, generado por2541266. Se compara con el mismo productor genuino regenerado durante esta revisión. No es un JSON v1 reconstruido manualmente ni una etiqueta de procedencia forjada.

## Reproducciones puntuales dentro del worktree C3 preparado

```bash
cargo run --locked --offline --release \
  --manifest-path verification/c3_review/Cargo.toml --bin c3-directed -- thermo
cargo run --locked --offline --release \
  --manifest-path verification/c3_review/Cargo.toml --bin c3-directed -- replay
cargo run --locked --offline --release \
  --manifest-path verification/c3_review/Cargo.toml --bin c3-directed -- fields
cargo run --locked --offline --release \
  --manifest-path verification/c3_review/Cargo.toml --bin c3-directed -- thermal
cargo run --locked --offline --release \
  --manifest-path verification/c3_review/Cargo.toml --bin c3-directed -- battery
cargo run --locked --offline --release \
  --manifest-path verification/c3_review/Cargo.toml --bin c3-directed -- semantics
```

Controles C2, sin cambiar una línea del probe:

```bash
cargo run --locked --offline --release \
  --manifest-path verification/c3_review/producer_c2/Cargo.toml \
  --bin c2-directed-control -- thermo
```

Cambiar modo a battery/semantics/replay para los restantes antes/después. Los crates de ese manifest salen de `git archive a0040c5`, no del C3 ni de un worktree modificado del implementador.

El checker nuevo usa `D16_C3_RESULTS=/ruta/output` y espera seis JSONL(fields,thermo,thermal,battery,replay,semantics). Sus cuatro fallos corresponden a requisitos específicos; el caso10 aplica la coherencia modelo/estado del contrato autorizado, aunque el implementador haya relajado esa regla. No se disfraza esta discrepancia como nueva funcionalidad ni se toca el checker anterior.

## Corridas conservadas y conteos

- [previous-suites/commands.json](c3-results/previous-suites/commands.json) y additional-commands: gates209/0, referencia163/0, oráculos22/32/19/19/15/15, CLI y productores. Fuentes materializadas desde árbolGit6ee6e72450145f8388b8d59cb190b107aa47ba1f.
- [c2-extension-final/commands.json](c3-results/c2-extension-final/commands.json):25 snapshots y oráculo C2 11/11,1442 predicados,752 filas. FuenteGit e087a0d99e5fff739054e3e66655a095bf1f3221. La primera adaptación fallida por ruta de run_history queda en c2-extension/ sin borrar logs.
- [final-from-git/commands.json](c3-results/final-from-git/commands.json): seis modos C3 y cuatro controles C2; fuenteGit **999ccd20e9454149d8f81da041a9bd3f993b58cf**. Probes y runners de entrega coinciden con su subtree; prueba en final-source-proof.json. Los árboles de preparación no son dependencia permanente: reproducir con la rama/commit final de entrega.
- [final-from-git/c3-summary.json](c3-results/final-from-git/c3-summary.json): **11 casos,7 pasan,4 fallan,0 errores,323 predicados,333 filas/11 familias**. Los casos se detienen al primer predicado fallido; todas las observaciones se generaron.
- [numerical-metrics.json](c3-results/numerical-metrics.json): ralentí, balances térmicos y snapshot R02 con su discrepancia histórica intacta.
- [source-index.json](c3-results/source-index.json), diffs, identities/preservation y estados Git: referencias reales y límites de aislamiento.
- Corridas iniciales y ampliaciones(directed-initial,directed-from-git,directed-final,directed-complete) conservadas; las nuevas pruebas de memoria/procedencia no sobrescriben evidencia previa.

Para reejecutar checkers sobre logs rastreados, copiar primero cada directorio a una ruta temporal nueva: los checkers escriben summaries. Los cuatro anteriores esperan respectivamente env D16_AUDIT_RESULTS, D16_REVIEW_RESULTS, D16_REVIEW_RESULTS y D16_C2_RESULTS. El nuevo usa D16_C3_RESULTS. No alterar tolerancias ni eliminar filas para conseguir verde.

## Referencias numéricas y límites de prueba

Presión ideal del nodo se contrasta con límites de representabilidad independientes, sin máximo OEM. Temperatura/masa/energía/densidad finitas no implican presión representable. En la batería, MAX×factor1.2 a10°C excede MAX; el positivo MAX/4 no. Atomicidad compara WorkState completo mediante Debug, incluidos ledgers; donde el contrato lo pide, se contrastan snapshots/hashes/trazas. Esto no es una prueba de layout binario ni de payloads arbitrarios de NaN.

Replay compara samples completos y estado, con alcances explícitos. El control c2=0 excluye solamente el campo de presión perdido al valorar equivalencia dinámica, y **registra** si pmax difiere; no oculta ese resultado ni certifica hash completo. Omitir memoria requerida de combustión/eventos/combustible se prueba aparte y da error de parseo. El experimento de pérdida posterior a migración preserva la metadata original, sin falsificarla.

No hay smoke posterior a integración porque Gate A falla. Los gates y los cuatro CLI se ejecutaron sobre el SHA exacto examinado. No se fusionó, corrigió producción, actualizó gate histórico, adquirió evidencia experimental ni inició R03/R04/R05/S31.
