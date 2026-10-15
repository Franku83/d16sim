# Reproducción independiente C4

Objetivo fijo: **8c51b5d9c705745b1a558b00e8bc41d73aad0d7f**. Nuevas fuentes en `verification/c4_review`, dentro de la rama revisora; su producción sigue8a. No ejecutar probes directamente contra esos crates pensando que son C4. Los runners crean/usan un worktree detached exacto C4 y extraen sus fuentes de auditoría desde Git.

## Ejecución completa

Requisitos: objetos Git históricos, Python3 con tarfile filter=data, Rust/Cargo y dependencias bloqueadas disponibles para --offline. Los productores se reconstruyen con git archive; no requieren fuentes/fixtures ignorados previos. Ver environment.json.

Desde la rama revisora al commit de entrega:

```bash
python3 verification/c4_review/run_c4.py \
  --source-ref verification/r01-r02-2026-10-10 \
  --worktree ../d16sim-c4-proof-nuevo \
  --output /tmp/d16-c4-literal-nuevo

python3 verification/c4_review/run_extra.py \
  --source-ref verification/r01-r02-2026-10-10 \
  --worktree ../d16sim-c4-proof-nuevo \
  --output /tmp/d16-c4-c2-nuevo

python3 verification/c4_review/run_directed.py \
  --source-ref verification/r01-r02-2026-10-10 \
  --worktree ../d16sim-c4-proof-nuevo \
  --output /tmp/d16-c4-c3-nuevo

python3 verification/c4_review/run_extended.py \
  --source-ref verification/r01-r02-2026-10-10 \
  --worktree ../d16sim-c4-proof-nuevo \
  --literal-results /tmp/d16-c4-literal-nuevo \
  --output /tmp/d16-c4-independiente-nuevo
```

Los cuatro runners esperan exit0 al completar sus baterías con los resultados declarados. **Los dos checkers S30 internos devuelven exit1**, con11 fallos literal y10 adaptados. El wrapper no convierte esos rojos en PASS. Ningún runner integra, modifica producción ni concede gates. Las rutas de salida deben ser nuevas y vacías; no se sobrescriben ejecuciones históricas.

El primer runner ejecuta exactamente:

```bash
cargo test --locked --workspace
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D clippy::unwrap_used
```

También S30 referencia163/0 y9/32, tres modos originales **literales**, adversarial/C1, cinco repeticiones deterministas, cuatro CLI y productores históricos. `--skip-baseline` omite la referenciaS30, sin certificarla por sí solo. Segundo: productores genuinos25 estados y checker C2 intacto. Tercero: seis modos/checker C3 intactos y contrafactuales C2 archivados. Cuarto: ejecución adaptada separada, comparación completa literal/adaptado y cuatro modos nuevos/8 casos C4.

## Integridad de fuentes y opción A

`integrity.json` identifica los cinco checkers y sus fuentes congeladas desde36919b0. Los originales se contrastan además con sus commits de origen:

- check_contracts.py SHA256 b027d9e3ab697b1974e87980765b7b4872e24e3792153ec3e02b598bd0756b36.
- check_adversarial.py SHA256 d67f74f3c560a4bee17d703734a2b48ac5e88d0624351e51dfef5356e59f5b9b.
- check_c1_controls.py SHA256 b021e634c12c47074cb65d138eed083e544c2323eabcd19f5b372df34c914033.
- check_c2.py y check_c3.py: hashes completos en integrity.json; byte-idénticos a36919b0.

Main/adversarial/common/c1_controls/c2_boundaries/c3_directed permanecen byte-idénticos. Helper observacional de CLI se verifica por extracción exacta del SHA candidato. Main literal conserva las cuatro inicializaciones neutrales R01 ya auditadas; ninguna adaptación adicional física.

El nuevo `src/main_option_a.rs` es copia del source literal con **solo** `v1.model = None;` inmediatamente después de schema1. No se sobreescribe main.rs. Son dos bins separados: `d16sim-independent-probes` y `s30-option-a`, con el mismo checker y salidas separadas.

`final-from-git/adapter/` entrega ambas versiones, diff, hashes y comparación de resultados. OriginalSHA256 b28adcc0b93950aadf73c950e47b4679f6b90f746dba688ad4cad09c37607982; derivadoSHA256 c194a4625139f4bb53ef74a8a5cb1b60bb3212bfb1af586d5615eb0603de4e5b. La única observación distinta es v1_hash_migration.accepted, manteniendo old_bytes/hash. El modo adapter independiente comprueba además estado/hash/provenance iguales al genuino2541266, rechazo literal y aceptación genuina/adaptada.

**Caso14 literal: FAIL preservado.** No se cambian expectations/tolerances ni se aplica una excepción al Core.

## Nuevos probes puntuales

Dentro del worktree C4 preparado:

```bash
cargo run --locked --offline --release \
  --manifest-path verification/c4_review/Cargo.toml --bin c4-directed -- nodes
cargo run --locked --offline --release \
  --manifest-path verification/c4_review/Cargo.toml --bin c4-directed -- provenance
cargo run --locked --offline --release \
  --manifest-path verification/c4_review/Cargo.toml --bin c4-directed -- adapter
cargo run --locked --offline --release \
  --manifest-path verification/c4_review/Cargo.toml --bin c4-directed -- battery
```

Nuevo checker: `D16_C4_RESULTS=/ruta/output python3 verification/c4_review/check_c4.py`. Este oráculo nuevo está separado de los cinco históricos. Sobre logs rastreados usar una copia temporal, porque escribe summaries.

Pruebas de nodos usan presiones/densidades positivas finitas o inválidas, error atómico y estado/hash correcto. Para run_engine se comprueba su estado inicial propio: no existe parámetro para inyectarle un WorkState externo.

La referencia eléctrica Decimal2000 evita redondear f64MAX antes de evaluar la identidad MAX×1+0. Incluye combinaciones de SOC/frío que vuelven inválido un R0 individual finito; no asume que MAX/16 siempre sea válido. No modifica umbrales del Core. La primera precisión100 generó un falso negativo; se conservan7/8, log y demostración de la corrección en new-checker-reference-correction.json. Los checkers originales nunca se editaron.

Procedencia se comprueba con ocho eventos reales del experimento numérico, alternando pérdida de pmax y del marcador de accounting. Se conserva cada registro completo, orden/hash/origen, más320 relecturas/recapturas JSON. La hoja sin prior coincide con el registro C3 histórico y se deserializa en cada cadena; legacy-metadata-compatibility.json registra la comparación. No se interpreta metadata como prueba criptográfica ni como medición experimental.

## Resultados y trazabilidad

- `literal-and-regressions/`: workspace215/0, fmt/clippy0, literal21/32, adversarial19/19, C1 15/15, referenciaS30 y CLI. Árbol de fuentes4244d65c50ab57bc6d92003f596c37d2c251a8a2.
- `c2-controls/`:11/11,1442 predicados,752 filas/15 familias. `c3-controls/`:11/11,360 predicados,333 filas/11 familias. Árbol5c053fb8812c41819cf18f6a116b9703651463ca.
- `option-a/run/`: primera corrida adaptada separada, conservada.
- `independent-final/`: primera ampliación con referencia decimal100 y7/8, conservada sin sobrescribir.
- **`final-from-git/`**: fuenteGit **f1f0af78c84ee11ebb0032212ceed4beef61dbfe**, adaptado22/32, ampliación **8/8,285 predicados,183 filas/8 familias**, con comparación literal/adaptado. Sus fuentes coinciden con el subtree de entrega; ver final-source-proof.json. Los árboles de preparación no son dependencia permanente: reproducir con la rama/commit final.
- `commands.json` de cada runner, source-index, identities, preservation, implementation.diff, numerical-metrics y estadosGit registran argv/cwd/env/exit/duración, fuentes y límites.
- Constancia de integración y smoke posteriores, cuando se ejecuten, en integration-receipt.json y post-ff-smoke/. Estos no sustituyen la verificación previa.

Se conserva todo resultado anterior C1/C2/C3 y los seis originales de auditoría. Los únicos cambios nuevos del revisor son docs/evidencia/probes en su rama; no se incorporan a producción al hacer ff-only del candidato.

## Límite científico

Host único, dominios finitos, sin fuzzing universal, profundidad arbitraria de metadata o validación experimental. Atomicidad se compara con Debug de WorkState completo(incluidos ledgers), hashes/snapshots/trazas según garantía; no prueba layout binario ni payloads arbitrarios de NaN. Priors/model no autenticados, fuentes externas fiables necesarias. Se preservan K01–K03,G01–G07,AUD-R01 y los IDs R03/R04. No se ejecutan nuevos sprints ni se declara D16Y7 predictivo.
