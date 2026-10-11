# Reproducción independiente C2

Objetivo exacto: **a0040c58174a6cf6a2ac84e38605eafd2f584159**. Las fuentes del revisor viven en `verification/c2_review`, dentro de la rama `verification/r01-r02-2026-10-10`; producción de esa rama permanece8a. No ejecutar directamente sus probes contra esos crates como si fueran C2. Los runners materializan fuentes desde Git en un checkout separado con HEAD exacto C2, y verifican ausencia de modificaciones productivas.

## Reproducción completa desde un checkout limpio del revisor

Requisitos: objetos Git de los SHAs históricos/ref de revisión, Python3 con tarfile filter=data, Rust/Cargo, dependencias bloqueadas accesibles en cache para --offline. Plataforma de ejecución en [environment.json](c2-results/environment.json). Los runners no necesitan fixtures ni fuentes ignoradas previas; reconstruyen archivos históricos mediante git archive. Cache de build no constituye evidencia.

```bash
python3 verification/c2_review/run_c2.py \
  --source-ref verification/r01-r02-2026-10-10 \
  --worktree ../d16sim-c2-proof-nuevo \
  --output /tmp/d16sim-c2-previos-nuevos

python3 verification/c2_review/run_extra.py \
  --source-ref verification/r01-r02-2026-10-10 \
  --worktree ../d16sim-c2-proof-nuevo \
  --output /tmp/d16sim-c2-ampliados-nuevos
```

Primer comando: exit esperado0 (oráculos previos22/32,19/19,15/15; el wrapper conserva el exit1 del oráculo original fuera de alcance). Segundo: **exit esperado1**, cinco incumplimientos de once casos nuevos. Los outputs deben ser rutas nuevas vacías; no se sobrescriben ejecuciones. Ningún runner autoriza/inicia otro batch ni llama merge. La aprobación formal requiere revisar todos los resultados, no el código de salida de una batería aislada.

El primer runner ejecuta:

```bash
cargo test --locked --workspace
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D clippy::unwrap_used
```

También referenciaS30 completa,3 modos originales,5 adversariales repetidos,9 controlesC1,4 CLI y productores anteriores. `--skip-baseline` puede omitir la referenciaS30 ya conservada; no la certifica por sí solo. Segundo runner archiva los5 productores auténticos, genera25 snapshots, ejecuta control comparativoC1 y cinco modos ampliados con nuevo checker. Los lockfiles de productores se entregan rastreados; no se resuelven nuevas versiones durante reproducción final.

## Integridad independiente

[Integrity-expanded.json](c2-results/integrity-expanded.json) registra comparación byte-exacta y hashes:

| Oráculo | Referencia Git | SHA256 |
|---|---|---|
| check_contracts.py | 25d528e:audit/independent/check_contracts.py | b027d9e3ab697b1974e87980765b7b4872e24e3792153ec3e02b598bd0756b36 |
| check_adversarial.py | 1d61437:verification/remediation_review/check_adversarial.py | d67f74f3c560a4bee17d703734a2b48ac5e88d0624351e51dfef5356e59f5b9b |
| check_c1_controls.py | a0088ff:verification/c1_review/check_c1_controls.py | b021e634c12c47074cb65d138eed083e544c2323eabcd19f5b372df34c914033 |

Main/adversarial/common/c1_controls anteriores también son byte-idénticos. Main original solo conserva los cuatro campos neutros necesarios añadidos en R01–R02: inventarios plenum/escape/emitido=+0 y pmax=None. No hubo nuevas adaptaciones físicas ni de expectativas C2. Helper `src/observed_cli.rs` extraído exactamente desde C2 entre `fn observable_chamber_split(` y siguiente `/// Comando S24:`; incluye su newline final original. `git diff --check` marca ese blank line intencional del helper; no se recorta porque se exige identidad de bytes. No es un fallo del cargo fmt productivo, que pasó.

Cambios exclusivamente en nuevas copias: configuración target/destino de logs, verificación del tercer oráculo, nuevo bin/runner/matriz y productores adicionales. `check_c2.py` es un oráculo nuevo, separado; nunca sustituye tolerancias anteriores. Producción C2 sí adapta fixtures sintéticos de sus unit tests de diagnóstico a firmas normalizadas; no equivale a alterar los tres oráculos independientes y no se presenta como medición externa.

## Probes puntuales en el worktree C2 preparado

```bash
cargo run --locked --offline --release \
  --manifest-path verification/c2_review/Cargo.toml --bin c2-boundaries -- matrix
cargo run --locked --offline --release \
  --manifest-path verification/c2_review/Cargo.toml --bin c2-boundaries -- focused
cargo run --locked --offline --release \
  --manifest-path verification/c2_review/Cargo.toml --bin c2-boundaries -- thermal
cargo run --locked --offline --release \
  --manifest-path verification/c2_review/Cargo.toml --bin c2-boundaries -- diagnostics
D16_C2_FIXTURES=/ruta/resultados/producer-matrix cargo run --locked --offline --release \
  --manifest-path verification/c2_review/Cargo.toml --bin c2-boundaries -- snapshots
```

Para generar exclusivamente el matrix histórico después de preparar fuentes Git:

```bash
python3 verification/c2_review/run_history.py \
  --worktree /ruta/c2-proof --output /ruta/nueva/producer-matrix
cargo run --locked --offline --release \
  --manifest-path verification/c2_review/producer_c1/Cargo.toml --bin previous-control
```

Los cinco manifests apuntan a `target/remediation-review-{legacy,s30,r02,c1,c2}`, generados por run_extra con archivos reales del SHA. Productorv1 usa2541266; no un tipo v1 emulado en C2. Los estados fríos se construyen con el mismo fixture declarado de temperatura/masa ambiente y se serializan por el tipo histórico real. Los calientes, pre/mid/post combustión y residuales se generan mediante resume del solver de su commit. El control C1 nativo despeja únicamente provenance auxiliar del montaje inicial después de reconstruir presión conocida; no cambia estado físico. Esta decisión está visible en el source, no en archivos externos.

## Reejecutar checkers sobre evidencia rastreada

Usar copia temporal de los logs, porque los checkers escriben summaries:

```bash
cp -a docs/remediation/review/c2-results/run-01 /tmp/c2-check-previos-nuevo
D16_AUDIT_RESULTS=/tmp/c2-check-previos-nuevo python3 verification/c2_review/check_contracts.py
D16_REVIEW_RESULTS=/tmp/c2-check-previos-nuevo python3 verification/c2_review/check_adversarial.py
D16_REVIEW_RESULTS=/tmp/c2-check-previos-nuevo python3 verification/c2_review/check_c1_controls.py
cp -a docs/remediation/review/c2-results/from-git-expanded /tmp/c2-check-ampliado-nuevo
D16_C2_RESULTS=/tmp/c2-check-ampliado-nuevo python3 verification/c2_review/check_c2.py
```

Exits1/0/0/1. El oráculo original conserva diez rojos R03/R04; el nuevo tiene cinco rojos C2 materialmente independientes.

## Ensayos y trazabilidad

- [run-01/commands.json](c2-results/run-01/commands.json), additional-commands.json: argv, cwd, env, exit y duración de gates/oráculos/CLI/productores/controles; reproducción driver completo.
- [run-01/integrity.json](c2-results/run-01/integrity.json): adaptación neutra, helper exacto y base_inputs byte-identical.
- [from-git-expanded/commands.json](c2-results/from-git-expanded/commands.json): ejecución nueva definitiva; snapshots genuinos por etapa y comandos de productor en producer-matrix/commands.txt.
- [from-git-expanded/c2-summary.json](c2-results/from-git-expanded/c2-summary.json): 11 casos,6 pasan,5 fallan,0 errores,1274 predicados,752 filas/15 familias.
- [focused.jsonl](c2-results/from-git-expanded/focused.jsonl), [previous-c1-controls.jsonl](c2-results/from-git-expanded/previous-c1-controls.jsonl): controles físico/semántico y antes/despuésC1.
- [SNAPSHOT_INVENTORY.md](c2-results/SNAPSHOT_INVENTORY.md): hashes/versión/procedencia y recapitulación de cada archivo.
- [preservation.json](c2-results/preservation.json), [git-state-before-verdict.json](c2-results/git-state-before-verdict.json), source-index y diffs: límites Git y fuentes de contrato.

Reproducción de oráculos previos se materializó desde árbolGit `d0b9f4b85f72d1b330815ed245981df7c0a3455b`. Extensión final desde árbolGit `4479672e53b6d1fd4972a5b55be098335cc9b47b`; sus archivos `verification/c2_review` son idénticos a la entrega. Estos árboles registran la preparación antes del commit de informes; el usuario reproduce con la rama/commit final, sin depender de permanencia de objetos no referenciados. La comparación del subtree final se registra en final-source-proof.json. Las corridas anteriores nuevas (`from-git-extra`, `from-git-final`) permanecen preservadas; `from-git-expanded` añade cuatro relaciones y entradas Engine para los98 parámetros térmicos, no altera expectativas de los tres oráculos anteriores.

Atomicidad se compara mediante Debug de WorkState completo, incluidos ledgers, y snapshots/hashes cuando aplica. No se confunde igualdad de Debug con prueba de payloads arbitrarios de NaN ni se afirma fuzzing exhaustivo. Replay nativo completo contrasta estado/hash y toda la traza, no solo RPM. Química usa inventario independiente de contador; Wiebe antiderivada y Newton cooling aportan referencias analíticas distintas de repetir únicamente el solver.

No modificar ningún oráculo para eliminar los cinco rojos. No ajustar el exceso histórico1157J ni parámetros nominales para imponer resultados. No hay smoke posterior a integración: el gate falla y la faseB no se ejecuta.
