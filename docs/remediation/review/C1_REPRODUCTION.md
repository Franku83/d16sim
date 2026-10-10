# Reproducción independiente C1

Objetivo exacto: `caad1cbf31d01531e0fbd81be6190f52b2d1c2d7`. Fuentes y logs nuevos viven en la rama `verification/r01-r02-2026-10-10`, separados de producción. El código del revisor sigue en8a: **no ejecutar directamente el harness contra esos crates pensando que son C1**.

## Ejecución desde Git

Con los objetos históricos disponibles y la rama del revisor al commit de entrega:

```bash
cd /ruta/al/worktree/verification
python3 verification/c1_review/run_c1.py \
  --source-ref verification/r01-r02-2026-10-10 \
  --worktree ../d16sim-c1-proof-nuevo \
  --output /tmp/d16sim-c1-resultados-nuevos
```

**Exit esperado1.** Las 15 pruebas nuevas tienen cinco fallos reales; el oráculo S30 conserva diez fallos R03/R04. Un exit1 no significa compilación fallida: examinar execution-complete, summaries y logs. El script no integra, corrige ni autoriza etapas posteriores. Los directorios de salida deben estar vacíos: nunca se sobrescribe una ejecución anterior. Worktrees se conservan.

El runner crea un worktree detached en el SHA C1, extrae únicamente los artefactos rastreados por Git de la referencia del revisor y verifica igualdad de ambos oráculos y experimentos anteriores. No requiere `audit/` ignorado del implementador. Los sources históricos se regeneran con git archive, sin depender de copias ignoradas preexistentes. Los caches de compilación pueden reutilizarse; no son fixtures/oráculos.

Requisitos: Git con ff09a0d/25d528e/1d61437/2541266/8a7e868/caad1cb, Python con extracción segura `tarfile.filter=data`, Rust/Cargo y dependencias de los lockfiles. Host usado Linux x86_64, Rust/Cargo1.99.0; ambiente completo en [environment.json](c1-results/environment.json). Los producers y probes se ejecutan locked/offline; si falta cache, obtener primero las dependencias bloqueadas, sin modificar lockfiles. El runner permite `--skip-baseline` para omitir la referencia S30 ya documentada; ese flag no certifica163 casos históricos por sí solo.

Se ejecutan cargo test/fmt/clippy exactamente con los comandos exigidos, el harness original, el adversarial anterior, repetición de cinco modos, cuatro CLI mecánicos y productores v1/v2/R02. Después se ejecutan nueve modos nuevos y su checker. El wrapper valida hashes de los experimentos conservados; reproduce.py valida la adaptación neutra y extracción del helper. No se modifican los seis originales de auditoría ni los cinco documentos de la revisión anterior.

## Integridad de oráculos y adaptaciones

Las referencias exactas y SHA256 están en [independent-integrity.json](c1-results/independent-integrity.json).

- `check_contracts.py`: byte-idéntico a `25d528e:audit/independent/check_contracts.py`, SHA256 `b027d9e3ab697b1974e87980765b7b4872e24e3792153ec3e02b598bd0756b36`.
- `check_adversarial.py`: byte-idéntico a `1d61437:verification/remediation_review/check_adversarial.py`, SHA256 `d67f74f3c560a4bee17d703734a2b48ac5e88d0624351e51dfef5356e59f5b9b`; no alteración de las 19 expectativas/tolerancias.
- `src/main.rs`, `src/adversarial.rs`, `src/common.rs`: byte-idénticos a los adaptadores/experimentos rastreados en1d61437. Main original de25d + exactamente cuatro campos neutrales R01; sin cambio adicional C1.
- `src/observed_cli.rs`: bytes exactos de C1 entre `fn observable_chamber_split(` y el marcador siguiente `/// Comando S24:`; termina con el mismo newline. Es extracción de producción compilada para acceder al helper privado, no oráculo independiente. Los cuatro comandos CLI reales se ejecutan aparte.
- `legacy/src/main.rs`: productor anterior conservado. Nuevo `cold_or_hot.rs` solo añade selección de estado frío sobre el mismo tipo histórico.
- `legacy_v2`: productor compilado con los tipos/solver de S30, no un DTO v2 escrito a mano. `r02_producer`: tipos reales8a para snapshot nativo y controles históricos de frontera.
- `reproduce.py`: nuevo objetivo C1, directorio distinto y selección explícita del bin original al coexistir el productor auxiliar. Se conservan experimentos, tolerancias y resultados de8a en sus rutas originales.
- `src/c1_controls.rs` y `check_c1_controls.py`: batería adicional independiente, claramente separada; no sustituye ninguno de los dos oráculos anteriores.

La procedencia v1 se confirma generando otra vez el archivo caliente con el productor histórico `2541266`, y comparando sus bytes contra el archivo entregado en1d61437: identidad exacta. Es el solver/tipo histórico compilado con toolchain actual; no un binario histórico preservado.

## Comandos puntuales dentro del worktree C1 preparado

```bash
cargo test --locked --workspace
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D clippy::unwrap_used
cargo run --locked --offline --release \
  --manifest-path verification/c1_review/Cargo.toml --bin c1-controls -- boundary
cargo run --locked --offline --release \
  --manifest-path verification/c1_review/Cargo.toml --bin c1-controls -- chemistry
cargo run --locked --offline --release \
  --manifest-path verification/c1_review/Cargo.toml --bin c1-controls -- analytic
```

Los otros modos: diagnostic, migration, thermal, electronics, fit, dyno. Migration requiere `D16_REVIEW_RESULTS` con los seis fixtures históricos generados por el runner. Boundary no necesita archivo externo y reproduce C1-01/C1-02. Migration reproduce C1-05 con un snapshot R02 auténtico cortado en0.35955s solicitado (tiempo integrado0.3596s). Thermal reproduce C1-04. Diagnostic reproduce C1-03.

Para consultar solo los oráculos con resultados rastreados, desde el worktree del revisor:

```bash
D16_AUDIT_RESULTS="$PWD/docs/remediation/review/c1-results" \
  python3 verification/c1_review/check_contracts.py
D16_REVIEW_RESULTS="$PWD/docs/remediation/review/c1-results" \
  python3 verification/c1_review/check_adversarial.py
D16_REVIEW_RESULTS="$PWD/docs/remediation/review/c1-results" \
  python3 verification/c1_review/check_c1_controls.py
```

Exit respectivos1/0/1: 22/32,19/19,10/15. No reinterpretar outputs JSONL exitosos como aprobación del contrato: los probes emiten observaciones, los checkers evalúan expectativas.

## Referencias numéricas y tolerancias nuevas

Química por paso utiliza consumo obtenido de `fuel_before + injected_delta − fuel_after − emitted_delta`, independiente del contador de masa quemada. Ese consumo×LHV×η se contrasta con delta de calor; cierre acumulado incluye inventario inicial. Tres pasos temporales y controles de escasez/reflujo/corte de futuros encendidos, con cortes JSON durante reacción y comparación de continuación. Desactivar futuros encendidos no extingue una combustión ya iniciada: no se añade un modelo de quenching.

Referencia angular: integrar analíticamente la Wiebe hasta ventana completa, masa esperada `m0(1−exp(−a))`; el probe no llama a `burned_fraction` como oráculo. Inercia1e9 mantiene ω100rad/s casi constante (variación<1.4e-10rad/s); es un fixture numérico declarado, sin reajuste nominal. El checker usa una cota de cuadratura rectangular `h·TV(rate)`, TV≤2·pico, y reducción del error del paso100µs al6.25µs. No exige monotonicidad de cada subdivisión porque el borde cae en puntos distintos.

Tolerancias de esta extensión: cierre1e-12kg, error químico1e-7J frente al observado~1e-12J, térmico1e-7J frente~1e-9J; bits/hashes/trazas/atomicidad sin tolerancia. El oráculo anterior conserva todas sus tolerancias. Una matriz finita no implica fuzzing exhaustivo; assertions ejecutadas se detienen al primer fallo de cada caso.

## Evidencia conservada

- [commands.json](c1-results/commands.json): argv/cwd/env/exit/duración de los gates y harnesses.
- [workspace-test.log](c1-results/workspace-test.log), [format.log](c1-results/format.log), [clippy.log](c1-results/clippy.log).
- [independent-test-summary.json](c1-results/independent-test-summary.json), [adversarial-test-summary.json](c1-results/adversarial-test-summary.json), [c1-controls-summary.json](c1-results/c1-controls-summary.json).
- `probes-*.jsonl`, `adversarial-*.jsonl`, `repeat-*.jsonl`, `controls-*.jsonl`: observaciones completas y stderr.
- `historical-v1-{cold,hot}.json`, `historical-v2-{cold,hot}.json`, `historical-r02-{native,mid-burn}.json`: artefactos genuinos; hashes de origen en filas de migración.
- [controls-boundary-r02.jsonl](c1-results/controls-boundary-r02.jsonl): controles idénticos sobre8a. No ejecuta la interfaz provenance de C1 sobre un tipo anterior que no la tenía.
- [implementation.diff](c1-results/implementation.diff): diff real8a→C1, preservado sin normalizar espacios.
- [numerical-metrics.json](c1-results/numerical-metrics.json) y [git-review-state.json](c1-results/git-review-state.json).
- [final-reproduction-verification.json](c1-results/final-reproduction-verification.json) registra la reproducción completa final: sources de árbol Git `012aead94dddc9d3df7170e70ac0554eadb61d45`, C1/S30198/163 y15 controles. Los nueve modos nuevos son byte-idénticos entre ejecución directa y reconstruida. Logs/fixtures finales en `c1-results/from-git-final/`; esa referencia de árbol preserva los sources del harness, que permanecen idénticos en el commit de entrega.
- El runner adicional reproduce todas las fuentes desde una referencia Git, con logs nuevos separados. La primera reproducción completa desde Git (antes del nuevo control mid-burn) se conserva en `c1-results/from-git-initial/`:198/0, baselineS30 163/0,32 y19 contratos,14 controles. La ampliación15casos no sobrescribe esa ejecución. La ejecución histórica8a y S30 entregada en1d61437 sigue intacta; la ejecución C1 inicial se conserva incluso cuando se repite la reproducción completa.

Caché de compilación no se entrega; fixture/producer/lockfile sí. Las rutas absolutas de logs describen el host y no son dependencias del formato científico. No se usan datos sintéticos como evidencia experimental, no hay datos de calibración/holdout nuevos ni validación del D16Y7.
