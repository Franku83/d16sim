# Preflight C4 reproducible

Este preflight reproduce el candidato C3 sin corregirlo. Requiere objetos Git de C3 y del commit revisor36919b0, Python con tarfile filter=data, Rust/Cargo y dependencias bloqueadas en cache para --offline.

Desde un checkout de la rama C4 que contenga estos scripts:

```bash
python3 tools/remediation_c4/preflight.py \
  --output docs/remediation/c4/evidence/preflight-nuevo \
  --archive target/c4-preflight-nuevo
```

Usar destinos nuevos y vacíos: el runner se niega a sobrescribir logs o archivos históricos. Archiva producción exacta C3 y extrae el harness exacto del commit revisor; los cuatro oráculos y el checker C3 se contrastan byte a byte. No depende de directorios de build ni fixtures ignorados previos.

Ejecuta seis modos del probe C3, guarda todas las333 observaciones y el checker. El **checker** debe devolver exit1 con7/11 y cuatro fallos; el **runner de reproducción** devuelve0 solo si reproduce exactamente ese baseline y completa el probe adicional de conflicto. No interpretar exit0 del runner como remediación o gate aprobado.

Resultados observados en `evidence/preflight-01/`:

- `c3-summary.json`:11 casos,7 pasan,4 fallan,0 errores,323 predicados,333 filas.
- `fields/thermo/thermal/battery/replay/semantics.jsonl`: observaciones intactas.
- `schema-conflict.jsonl`: versiones/hash/restore y comparación física entre fixture genuino y reetiquetado.
- `integrity.json`: hashes y referencias Git de todos los oráculos/sources preservados.
- `commands.json`: argumentos/cwd/exit/duración de cada ejecución.
- `environment.json`, `completion.json`: plataforma y estado explícito sin fixes.

El source adicional `schema_conflict.rs` no modifica Engine ni actúa como reemplazo de un oráculo. Se compila como otro bin del manifest de auditoría extraído, en el archivo generado dentro de target. `strict_pair_consistent` describe la regla propuesta; no se ha aplicado a producción.

No hay todavía resultados postcorrección ni suites de aceptación C4. No se reclasifican los cuatro fallos como resueltos, no se suprimen assertions, no se alteran gates ni se presenta calibración/validación experimental. Ver C4_SCHEMA_FIXTURE_DECISION.md para la decisión requerida antes de continuar.

# Verificación C4 posterior a la autorización formal A

Estado de entrega: **IMPLEMENTED_PENDING_INDEPENDENT_REVIEW**. El preflight anterior permanece histórico; la decisión pendiente fue resuelta por el responsable en C4_OPTION_A_AUTHORIZATION.md. Esto no concede gate de integración.

## Reproducir el candidato desde Git

Desde el commit final de `remediation/r01-r02-c4`, con todos los objetos históricos y dependencias bloqueadas disponibles:

```bash
python3 tools/remediation_c4/verify.py \
  --source-ref HEAD \
  --output docs/remediation/c4/evidence/verificacion-nueva \
  --archive target/c4-verificacion-nueva
```

Usar directorios nuevos y vacíos. El runner archiva el **source-ref exacto**, reconstruye el harness congelado de36919b0455e8eaf741a72ff4a74ae85f40e116ba y los productores2541266/ff09/8a/caad/a004 desde Git. No requiere archivos ignorados previos. El cache de build se puede regenerar; no es evidencia ni fuente de snapshots. No hace merge, checkout de ramas ajenas, push, reset o cambios en el revisor.

El driver comprueba cuatro oráculos y checker C3 byte a byte, además de sus sources principales. Crea una **nueva** copia del source main para el bin adaptado; inserta únicamente `v1.model = None` después del cambio de schema a1. No escribe sobre el original. Materializa fuente original/derivada, diff exacto, SHA256 y origen Git, y ejecuta ambos bins con los mismos parámetros/modos.

Los siguientes comandos se ejecutan literalmente en el código archivado:

```bash
cargo test --locked --workspace
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D clippy::unwrap_used
```

Los probes/productores usan cargo run --locked --offline --release. Se ejecutan:

- Tres modos literales originales + checker S30: **21/32**, once fallos, exit1.
- Tres modos de copia derivada autorizada + **mismo checker**: **22/32**, diez fallos, exit1.
- Cinco modos adversariales: **19/19**.
- Nueve modos de controles C1: **15/15**.
- Cinco productores genuinos×cinco estados y cinco modos C2: **11/11**.
- Seis modos dirigidos C3: **11/11**.
- Probe adicional que exige rechazo literal incoherente y aceptación de v1 adaptado/genuino con iguales estado/hash.
- Cuatro CLI mecánicos smoke.

El runner devuelve0 al completar las baterías con sus exits esperados; su completion indica solo implementación pendiente de revisión. **No** significa que el literal original pase ni que los diez pendientes R03/R04 estén resueltos. No clasifica automáticamente el gate científico o de integración.

## Ejecución conservada y trazabilidad

La corrida final de esta implementación está en `evidence/verification-02`, desde árbolGit **90d51d9a538034dc8ff9d6243cc4c6b1d13904c1**. La corrida verification-01 permanece como iteración previa: se reforzó posteriormente `Battery::validated` para rechazar también combinaciones de resistencia no representable antes de aceptar restore/no-op. Ambas se conservan; verification-02 es la referencia del candidato final.

- `commands.json`: argv/cwd/env/exit/duración.
- `integrity.json` y `oracles/`: fuentes congeladas/hashes; ningún checker editado.
- `literal/` y `adapted/`: ejecuciones completas separadas, outputs y fallos originales.
- `adapter/original_main.rs`, `derived_main.rs`, `fixture.diff`, `transformation.json`: **única línea autorizada** y cadena Git/SHA256.
- `adapter/result_equivalence.json`: única observación distinta `v1_hash_migration.accepted`; todas las demás idénticas. No se modifica ningún JSONL para conseguir la equivalencia.
- `adapter/behavior.jsonl`: literal con hash válido rechazado, adaptado/genuino aceptados.
- `shared/`, `c2/`, `c3/`: todos los resultados y snapshots de las suites preservadas.
- `numerical-metrics.json`, `reported-failures.json`: nominal/R02/procedencia y nombres exactos de los rojos.
- `source-proof.json` y `final-git-state.json` en evidence: identidad del código ejecutado frente a entrega y preservación de worktrees/referencias.

Resultados negativos previos: preflight-01(7/11), unit-red.log, engine-unit-red.log. Resultados unitarios positivos intermedios: unit-green*.log. Todos se conservan.

## Reproducir las regresiones nuevas contra C3 sin aplicar fixes

El patch `evidence/red-regressions-only.patch` añade exclusivamente tests C4 y el fixture genuino al source C3; no contiene cambios de producción. Su ejecución preservada es red-proof-engine/electrical.log. En un directorio nuevo:

```bash
mkdir /tmp/d16-c4-red-nuevo
git archive 3aef5f6ccdc076dad6e8be627a5eb836b26ee998 | tar -x -C /tmp/d16-c4-red-nuevo
cd /tmp/d16-c4-red-nuevo
patch -p1 < /ruta/a/d16sim-remediation-c4/docs/remediation/c4/evidence/red-regressions-only.patch
cargo test --locked -p engine r02_c4_tests
cargo test --locked -p electrical r02_c4_tests
```

Exits esperados101: Engine1 pasa/4 fallan, Electrical0 pasa/1 falla. El sexto caso conserva una limitación previa y ya pasa en C3; los cinco fallos reproducen las causas corregidas. El test final además refuerza la frontera validated/restore/no-op de batería. No se omite una prueba fallida ni se cambia su expectativa para obtener verde.

Para rerun de checkers sobre logs ya rastreados, copiar antes cada directorio a un destino temporal nuevo: escriben summaries. Usar D16_AUDIT_RESULTS para literal/adapted, D16_REVIEW_RESULTS para shared (adversarial/C1), D16_C2_RESULTS para c2 y D16_C3_RESULTS para c3. Exits respectivos1/1/0/0/0/0.

## Límites

La extensión `prior` es metadata opcional no autenticada; no cambia EngineState ni state_hash. No valida autoría o profundidad arbitraria de metadata maliciosa. Los hashes y replay cubren las rutas/domain ensayados, no todas las entradas posibles ni otro host. Ningún resultado aquí es una nueva validación experimental del D16Y7. Los dos resultados S30 deben permanecer separados ante el revisor original; la autorización del dueño no es una autoaprobación del adaptador por el auditor.

Los logs se guardan byte-exactos: git diff --check puede señalar espacios de sugerencias de Clippy o líneas vacías finales de stdout. No se normalizan esas salidas; los sources Rust sí pasan cargo fmt. Esto no convierte warnings de herramientas en fallos de los contratos ni altera resultados.
