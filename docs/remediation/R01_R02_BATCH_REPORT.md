# Reporte de batch — Remediación R01–R02 (D16SIM)

- **Fecha:** 2026-10-10
- **Base de producción auditada (S30):** `ff09a0dda82188e6bf27a80d17c902f8972a39d2`
- **Auditoría independiente:** rama `audit/independent-2026-10-10`, commit `25d528e` (worktree `/home/franku/projects/d16sim-audit`, **intacto**)
- **Rama de remediación:** `remediation/r01-r05`
- **Worktree:** `/home/franku/projects/d16sim-remediation`
- **Spec de referencia:** `docs/remediation/D16SIM_REMEDIATION_R01_R05_SPEC_v1.0.md`
- **Toolchain / plataforma:** rustc/cargo `1.99.0`, Linux x86_64 (cachyos, kernel 7.2.9)
- **Estado global:** `IMPLEMENTED_PENDING_INDEPENDENT_REVIEW` (sin revisor independiente disponible)

> Este batch **no** declara el Core predictivo, **no** usa datos del D16Y7, **no**
> ejecuta calibración experimental y **no** inicia R03/R04/R05 ni S31.

---

## 1. Commits y archivos por etapa

| Commit | Etapa | Resumen |
|---|---|---|
| `2a42df6` | R01 | Advección de especie fresca en la red de fugas (F08), unidades de líquidos (F09), frontera ambiente en el ledger térmico (F10), observación mecánica desde flujos físicos (F02). |
| `f4dd610` | R01–R02 | Motor: transporte de especie combustible (F01), historia de presión persistida (F07), JSON lossless (F05), verificación v1 antes de migrar (F06), fronteras fail-closed (F15). |
| *(documentación)* | R01–R02 | `docs/adr/ADR-038…`, `docs/adr/README.md`, `docs/PROGRESS.md`, `.gitignore`, este reporte. |

Archivos modificados por etapa:

- **R01:** `crates/leak/src/network.rs`, `crates/thermal/src/network.rs`, `crates/cli/src/main.rs`, `docs/physics/{leaks,thermal,diagnostic_tools}.md`.
- **R01–R02 (motor/fronteras):** `crates/engine/src/run.rs`, `crates/engine/src/state.rs`, `crates/engine-types/Cargo.toml`, `crates/dyno/src/brake.rs`, `crates/ecu/src/decoder.rs`, `crates/calibration/src/dataset.rs`, `docs/snapshot-contract.md`, `docs/physics/integrated_engine.md`.
- **Documentación:** ADR-038, índice ADR, PROGRESS, este reporte, `.gitignore`.

### Harness independiente — adaptación mínima documentada

El harness del auditor vive intacto en `/home/franku/projects/d16sim-audit/audit/independent/`.
Para ejecutarlo **contra esta rama** se copió a `<worktree>/audit/independent/`
(ignorado en git, no forma parte de producción). Diff frente al original:

- `check_contracts.py`: **byte-idéntico** (ninguna expectativa/umbral alterado).
- `src/main.rs`: **único** cambio, en el constructor del fixture `cold()`, al añadir
  los cuatro campos nuevos del estado con valores neutros:
  `plen_fuel_kg: 0.0, exh_fuel_kg: 0.0, fuel_emitted_kg: 0.0, pmax_pa: None`.
  Ninguna aserción, oráculo o valor de fixture cambia.

Reproducción: `bash audit/independent/reproduce.sh <dir>` (exit 1 = contratos rojos).

---

## 2. Estado individual de los nueve hallazgos del batch

| ID | Sev. | Estado | Evidencia |
|---|---|---|---|
| **F01** | P0 | `FIXED_PENDING_VERIFICATION` | Advección por especie; caso 10 verde; tests de conservación. |
| **F02** | P0 | `FIXED_PENDING_VERIFICATION` | `mech-demo` deriva el reparto de flujos; tests de invariancia/ambigüedad. |
| **F05** | P1 | `FIXED_PENDING_VERIFICATION` | `float_roundtrip`; casos 12–13 verdes; test de bits. |
| **F06** | P1 | `FIXED_PENDING_VERIFICATION` | Hash v1 verificado antes de migrar; caso 14 verde; test v1/tampered/esquema. |
| **F07** | P1 | `FIXED_PENDING_VERIFICATION` | `pmax_pa` persistido; caso 11 verde; replay multi-corte. |
| **F08** | P1 | `FIXED_PENDING_VERIFICATION` | Advección fresca + `fresh_closure`; caso 15 verde. |
| **F09** | P1 | `FIXED_PENDING_VERIFICATION` | Densidad de líquido; caso 16 verde; test dimensional. |
| **F10** | P1 | `FIXED_PENDING_VERIFICATION` | Calor al ambiente en el ledger; sin caso Python (nuevo test). |
| **F15** | P1 | `FIXED_PENDING_VERIFICATION` | Casos 24–27, 29–30 verdes; validación por API. |

Los nueve tienen reproducción antes/después y al menos un test de regresión nuevo.
**Ninguno** se declara `VERIFIED_BY_INDEPENDENT_REVIEW` (no hubo revisor).

---

## 3. Reproducción antes / después

### 3.1 Baseline (antes) — commit auditado `ff09a0d`

| Prueba | Resultado |
|---|---|
| `cargo test --locked --workspace` | 163 passed / 0 failed (59 targets con casos) |
| `cargo fmt --all -- --check` | OK |
| `cargo clippy … -D clippy::unwrap_used` | exit 0 (con warnings) |
| Harness independiente | exit 1 — **23 fallos / 9 aprobados** (138 filas / 42 familias) |

### 3.2 Después — HEAD de remediación

| Prueba | Resultado |
|---|---|
| `cargo test --locked --workspace` | **183 passed / 0 failed** (163 + 20 nuevos) |
| `cargo fmt --all -- --check` | OK |
| `cargo clippy … -D clippy::unwrap_used` | exit 0 (con warnings) |
| Harness independiente | exit 1 — **10 fallos / 22 aprobados** (89 predicados) |

Los **10 fallos restantes pertenecen todos a R03/R04** (fuera del batch autorizado):
`test_17`, `test_18` (dyno F03/F04), `test_19`, `test_20` (wear F11/F12),
`test_21`, `test_22` (decoder F13), `test_23` (wiring F14), `test_28` (F16),
`test_31`, `test_32` (fitter F18/F19). No hay fallos de R01/R02.

### 3.3 Valores antes/después por defecto (probes del auditor)

| Probe / caso | Antes | Después |
|---|---|---|
| `fuel_outflow` (caso 10) | `fuel_after = 6.6493908554e-5` (sin advección) | `fuel_after = 6.4635207550e-5` = `expected` (error < 1e-15) |
| `fuel_inventory` (caso 10) | `max_ratio_over_4s = 2.3657034193` | `0.0561209118` (≤ 1) |
| `snapshot_json` (casos 12–13) | θ `122.39085530309971 → …73` (bits perdidos, hash rojo) | round-trip exacto; `same_hash = true` |
| `v1_hash_migration` (caso 14) | `accepted = false` | `accepted = true` |
| `replay_split` c2=0.05 (caso 11) | rpm 175.608 vs 859.536, `hash_match=false` | rpm 286.323 vs 286.323, `hash_match=true` |
| `leak_species` (caso 15) | `receiver_fresh = 0` | `0.004002307770342` = `expected` |
| `leak_liquid_units` (caso 16) | `1.0000000000e-6 kg` (factor 870) | `8.6999999999e-4 kg` = `870·V` |
| `thermal_long` (F10, sin caso) | residuo energía −3.891e-3 / −3.687e-3 / −3.636e-3 (dt100/50/25 µs) | **+2.179e-5 / +1.119e-5 / +5.649e-6** |
| `engine_invalid` zero_record (caso 24) | **panic** | `returned_error=true`, sin panic |
| `engine_invalid` ηcomb (caso 24) | −1 y 2 aceptados | ambos rechazados |
| `thermal_invalid` (caso 25) | `zero_capacity_accepted=true`, estado no finito | `false`, estado finito |
| `dyno_invalid` (caso 26) | `below_absolute_zero_accepted=true` | `false` |
| `dataset_invalid` (caso 27) | aceptado | rechazado |
| `restore_invalid` (caso 29) | `negative_mass_accepted=true` | `false` |
| `decoder_invalid` (caso 30) | NaN/futuro aceptados | ambos rechazados |
| `mech-demo missing_head_gasket` (F02) | split 0.30/0.10/0.60 (por etiqueta) | **0.14/0.00/0.86** (por flujos; escape no instrumentado) |

---

## 4. Resultados de tests originales y nuevos

- **Originales:** 163/163 siguen verdes (ninguna expectativa relajada ni test
  eliminado; ninguna tolerancia cambiada).
- **Nuevos (20):**
  - engine (10): salida homogénea con combustible; inventario acotado y
    conservado sin chispa; energía química exige masa quemada; replay multi-corte
    con `c2≠0` + control `c2=0`; ledger térmico con/sin frontera ambiente;
    round-trip JSON con bits y hash; migración v1 auténtica + tampered + esquema
    desconocido; entradas inválidas fail-closed; `restore` rechaza estado
    imposible con hash válido; mismatch térmico.
  - leak (3): especie fresca + `fresh_closure`; volumen líquido→masa; rutas
    dimensionales gas/líquido.
  - thermal (2): ledger de frontera; capacidad nula rechazada antes de mutar.
  - dyno (1): ambiente bajo cero absoluto rechazado.
  - ecu (1): NaN/futuro rechazados.
  - calibration (1): observaciones no finitas / σ inválida / metadata incompleta.
  - cli (2): reparto desde flujos (no etiqueta) y preservación ante renombrado.
- **Conteo de asserts:** `assert!`/`assert_eq!` estáticos no equivalen a
  assertions ejecutadas; el harness reporta 89 predicados ejecutados y 138 filas
  de observación (no 138 tests). No se declaran cifras de cobertura física.

---

## 5. Cambios numéricos y explicación física

1. **Especies (F01):** el combustible ahora sale/refluye con la masa. En ralentí
   nominal 8 s (dt 50 µs) la RPM final pasó de 1025.82 a 1047.32 y `Q` de
   46373 J a 46147 J. El cierre de masa total sigue en ~7e-12 (relativo).
   **No** se reajustaron parámetros para preservar las cifras previas.
2. **Presión de fricción (F07):** `pmax_pa` ya no se reinicia en cada `resume`;
   con `c2≠0` la trayectoria cambia (de ahí 175.6→286.3 rpm en el corte de
   replay del probe). El efecto es el previsto por persistir el término.
3. **Ledger térmico (F10):** el residuo integrado cae de ~−0.0039 a ~+1e-5 y
   **decrece al refinar dt**, consistente con error de discretización, no con un
   término omitido.
4. **Líquidos (F09):** factor 870 corregido (masa ×870 mayor, dimensiones SI).
5. **Fugas (F08):** la fracción fresca receptora pasa de 0 a la masa fresca
   recibida / nueva masa; `fresh_closure ≈ 0`.

---

## 6. Modificaciones de esquema y compatibilidad

- **Esquema nativo v2 conservado**; no se hizo bump incompatible.
- Campos nuevos opcionales en `EngineState` (`plen_fuel_kg`, `exh_fuel_kg`,
  `fuel_emitted_kg`, `pmax_pa`) con `#[serde(default)]` y **omitidos en su valor
  neutro**. Los hashes de snapshots v1/v2 previos se preservan.
- **Verificación por esquema (F06):** el hash v1 se reconstruye sin `thermal`
  antes de migrar. Un hash v1 auténtico ya migra; uno alterado o de esquema
  desconocido se rechaza.
- **Parseo lossless (F05):** `serde_json/float_roundtrip` en `engine-types`
  (se unifica a todo el grafo de build).
- **Validación física fail-closed (F15):** hash válido ≠ estado válido.
- Detalle en `docs/snapshot-contract.md`; decisión en `ADR-038`.

---

## 7. Nuevas hipótesis, ADR y limitaciones

- **ADR-038** (especies + historia de presión + fronteras fail-closed), aceptado.
- **Nueva hipótesis operativa:** con advección de especies, el reflujo de
  combustible al plenum puede realimentar ciclos posteriores; su efecto en
  régimen/emisiones es una predicción **L1 no calibrada**, no un resultado.
- **Limitaciones vigentes (sin cambio):** K01–K03 (subsistemas no cierran motor
  runtime; combustión/llenado L1; contacto genérico ≠ fractura), G01–G07
  (sin datos experimentales, sin 2º host/experto, sin cadenas completas).
  `F10` no tiene caso Python dedicado en los 32 originales (nuevo test propio).
  El signo del par accesorio sigue sin contrato estricto (declarado, no defecto
  autónomo según la auditoría).

---

## 8. Regresiones, problemas pendientes y cobertura no examinada

- **Regresiones introducidas:** ninguna detectada (183/0 en workspace; sin
  fallos nuevos de harness; fmt/clippy OK).
- **Pendientes del batch:** ninguno `BLOCKED`. Los nueve hallazgos quedan
  `FIXED_PENDING_VERIFICATION` a la espera de revisión independiente.
- **Fuera del batch (siguen rojos, esperados):** F03, F04 (R04); F11, F12, F13,
  F14, F16, F18, F19 (R03); F17 y D01–D26 (R05).
- **Cobertura no examinada:** segundo host/ARM/Windows; horizonte > 120 s;
  validación experimental; acoplamiento ECU→combustión runtime; comportamiento
  de especie bajo gradientes espaciales (modelo homogéneo L1); fuzzing global.

---

## 9. Estado de revisión independiente

`IMPLEMENTED_PENDING_INDEPENDENT_REVIEW`. No hubo revisor independiente
disponible en esta ejecución; **no** se declara PASS independiente. La auditoría
original permanece inmutable como referencia. Recomendación: revisar los nueve
IDs contra los probes re-ejecutados y el diff de `audit/independent/src/main.rs`
(cambio neutro único).

---

## 10. Estado Git y recomendación

- Rama `remediation/r01-r05`; sin merge a `master`; auditoría separada e intacta.
- Working tree limpio salvo la documentación de este commit.
- **Recomendación:** solicitar **autorización explícita para el batch 2 (R03–R04)**.
  **STOP**: no iniciar R03/R04/R05 ni S31 sin nueva autorización.
