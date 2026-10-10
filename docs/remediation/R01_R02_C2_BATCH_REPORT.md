# Reporte de batch — Remediación R01–R02-C2 (D16SIM)

- **Fecha:** 2026-10-10
- **Base de implementación:** `remediation/r01-r02-c1` @ `caad1cbf31d01531e0fbd81be6190f52b2d1c2d7`
- **Segunda verificación independiente:** `verification/r01-r02-2026-10-10` @ `a0088ff0301c07aa89f4ca4f5fab32b53eba9960`
- **Veredicto vigente antes de C2:** `C1_REMEDIATION_REQUIRED`
- **Rama/worktree C2:** `remediation/r01-r02-c2` en `/home/franku/projects/d16sim-remediation-c2` (desde `caad1cb`)
- **Estado del implementador:** `IMPLEMENTED_PENDING_INDEPENDENT_REVIEW`
- **Toolchain:** rustc/cargo 1.99.0, Linux x86_64
- **Destino NO tocado:** `remediation/r01-r05` @ `8a7e868` (sin merge/ff/rebase/squash)

> No se declara `VERIFIED_FIXED` (corresponde al revisor). No se inician
> R03/R04/R05 ni S31. No se usan datos del D16Y7. No se ajustan parámetros
> físicos nominales ni tolerancias de oráculos.

---

## 1. Estado de C1-01 a C1-05 y RV/F-ID

| Hallazgo | RV/F-ID | Sev. | Estado implementador | Evidencia |
|---|---|---|---|---|
| **C1-01** | RV05/F15 | P1 | `IMPLEMENTED_PENDING_INDEPENDENT_REVIEW` | Matriz de inputs/estado/nodos/coherencia; 16 configs inválidas rechazadas en run/resume/restore sin mutar. |
| **C1-02** | RV06/F15 | P1 | `IMPLEMENTED_PENDING_INDEPENDENT_REVIEW` | `restore` rechaza `c_wall=0`, `wall_frac=NaN`, capacidad MAX (3/3). |
| **C1-03** | RV02/F02 | P2 | `IMPLEMENTED_PENDING_INDEPENDENT_REVIEW` | Observaciones inválidas/parciales ⇒ `Inconclusive` (4/4). |
| **C1-04** | RV06/F15 | P2 | `IMPLEMENTED_PENDING_INDEPENDENT_REVIEW` | Energía derivada T·C validada; capacidad MAX ⇒ rechazo atómico. |
| **C1-05** | RV04/F06 | P1 | `IMPLEMENTED_PENDING_INDEPENDENT_REVIEW` | `model` semántico; R02 mid-burn ⇒ `approximate_hot`; nativos C2 exactos. |
| **C1-G01** | — | P2 | `DOCUMENTED` | Metadata `model`/`provenance` declarada no autenticada por `state_hash`. |
| **C1-D01** | — | P2 | `RESTRICTED` | Claim de ADR-039 acotado por la matriz C2 (nota append-only). |

| Contrato | Estado C2 |
|---|---|
| **RV02/F02** | Cerrado en el dominio público del protocolo (frontera validada); AUD-R01 multicause sigue abierto. |
| **RV04/F06** | Lectura/hash históricos + migración pre-R02 + identidad semántica pre-C2/C2. |
| **RV05/F15** | Matriz integral ejecutada; rollback en todos los negativos. |
| **RV06/F15** | Params + estado + energía derivada + restore. |
| **RV01/F01, RV03/F05, RV07/F15, RV08/F15, RV09/F15** | Conservados (regresiones verdes). |
| **F07/F08/F09/F10** | Conservados (replay, especie fresca, líquidos, balance térmico). |

---

## 2. Reproducciones antes/después (controles C2)

| Caso | Antes (`a0088ff`) | Después (C2) |
|---|---|---|
| `control_afr_nan/zero`, `eta_nan`, `spark_adv_nan`, `lhv_nan`, `density_negative`, `charge_nan`, `dwell_nan`, `starter_cut_nan`, `starter_bad`, `inertia_nan/zero`, `wiebe_duration_inf` | aceptados en run/resume/restore | **rechazados, estado intacto, sin panic** |
| `thermal_capacity_zero/wall_nan` | run/restore rechazaban; **resume aceptaba** | **rechazados (3/3)** |
| `thermal_capacity_overflow` | run Ok con cierre **no finito** | **rechazado** |
| `node_v_zero`, `node_cv_nan`, `node_v_nan`, `thermal_presence_mismatch`, `thermal_stored_overflow`, `thermal_params_bad` | aceptados y estado mutado | **rechazados, `unchanged=true` (6/6)** |
| `thermal_restore_config` capacity_zero/wall_nan/overflow | aceptados (`stored_finite=false` en overflow) | **rechazados (3/3)** |
| `extended_thermal capacity_large` | `Ok`, `stored_j=Inf`, estado mutado | **rechazado, `unchanged=true`** |
| `diagnostic_invalid` compresión NaN / leak Inf / split [2,−1,0] / [.1,0,0] | `Rings` (4/4) | **`Inconclusive` (4/4)** |
| `r02_midburn_migration` | `provenance=None` (nativo sin etiqueta) | **`approximate_hot`**, `missing_fields=["combustion_accounting"]` |
| `friction_overflow` (estrés legítimo) | aceptado, energía finita | aceptado, energía finita (excluido del gate por diseño) |

`hash_valid=true` en los tres `thermal_restore_config`: el rechazo es por
configuración física, no por hash (separación integridad/validez).

---

## 3. Resultados originales y nuevos

| Batería | Antes | Después |
|---|---|---|
| `cargo test --locked --workspace` | 198/0 | **204/0** (+5 C2, +1 diagnóstico) |
| `cargo fmt --all -- --check` | OK | OK |
| `cargo clippy … -D clippy::unwrap_used` | exit 0 | exit 0 |
| Controles C1/C2 nuevos | 10/15 (121 predicados) | **15/15 (150 predicados)** |
| Harness adversarial (19) | 19/19 | **19/19** (196 predicados) |
| Harness original S30 (32) | 22/32 | **22/32** (89 predicados) |

Los 10 fallos del harness original son exactamente los de R03/R04:
17/18 (F03/F04), 19/20 (F11/F12), 21/22 (F13), 23 (F14), 28 (F16), 31/32
(F18/F19). No hay fallos nuevos ni ocultos.

---

## 4. Matriz ejecutada y cobertura por API

La matriz completa está en `R01_R02_C2_BOUNDARY_MATRIX.md`. Cobertura ejecutada:

- **Engine `EngineInputs`**: control (AFR, ηv, avance), combustible (LHV,
  densidad, stoich), eléctrico/arranque (tensión, dwell, cutout, motor,
  batería), inercia (J, masa), Wiebe, áreas, volúmenes, fricción, térmico.
- **Engine `WorkState`/nodos**: m/u/fresh/fuel, v/cv, t, historia de eventos,
  contadores/ledgers, `th_u0_j`, coherencia de red térmica, energía derivada.
- **Engine resultados**: `run_engine` exige estado y auditoría finitos;
  `resume` valida el candidato antes de confirmar (atomicidad).
- **Thermal**: params, estado de entrada, ambiente, candidato y energía
  almacenada derivada; overflow tiny/MAX.
- **Diagnostics**: compresión, leak-down, umbral, fracciones y cierre.
- **Snapshots**: v1 histórico, v2 S30, v2 R02 (nativo y mid-burn), C2 nativo;
  modelo desconocido/incoherente; metadata manipulada (documentada).

**Dominios no cubiertos**: fuzzing exhaustivo, callback malicioso, segundo
host/arquitectura, horizontes > 120 s, gradientes espaciales, reconstrucción
exacta de trayectorias legacy calientes.

---

## 5. Compatibilidad de snapshots por productor

| Productor | Estructura | Clasificación C2 | Continuación |
|---|---|---|---|
| v1 `2541266` | schema 1, sin `thermal`/R01 | legacy v1 | `exact_neutral` (frío) / `approximate_hot` (caliente) |
| v2 S30 `ff09a0d` | schema 2, sin `pmax`/fuel nodos | legacy v2 pre-R02 | idem |
| v2 R02 `8a7e868` nativo | schema 2, `pmax` presente, sin `model` | **pre-C2 ambiguo** | `approximate_hot` / `exact_neutral`; nunca nativo exacto |
| v2 R02 mid-burn `8a7e868` | `burn_accounted=false`, exceso 1157 J | **pre-C2 ambiguo** | `approximate_hot` (no se resta el exceso) |
| C1 `caad1cb` | schema 2, `pmax`, sin `model` | pre-C2 ambiguo (conservador) | aproximado (estado intacto, bit-exacto) |
| C2 nativo | schema 2 + `model{2,1}` | nativo exacto | bit-exacto |

Lectura/hash históricos se preservan en todos los casos. El exceso químico
heredado (~1157 J) **no se corrige artificialmente**: se etiqueta la
procedencia y se limita la continuación. No se infiere el productor por
valores (`pmax`/`burn_accounted`).

---

## 6. Decisiones de procedencia y confianza (C1-G01)

- `state_hash` autentica **solo el estado canónico**.
- `model` y `provenance` son **metadata no autenticada**; reserializar no las
  autentica. Un consumidor que necesite confiar en la procedencia debe
  verificarla contra una fuente externa fiable.
- No se añade firma criptográfica ni se reescriben hashes históricos.
- Control de manipulación (`provenance_tamper_control`): el hash del estado
  sigue válido y la metadata manipulada se conserva (comportamiento
  documentado, no un defecto del contrato de hash).

---

## 7. Cambios numéricos significativos

**Ninguno.** El ralentí nominal 8 s/dt 50 µs es idéntico a C1:
**1047.3038737395177 rpm**, `Q=46146.847889 J`, `W_ind=13286.519774 J`,
residuo energético `6.95678e-5`. C2 solo añade validación/rechazo; no toca
ecuaciones, parámetros ni la trayectoria nominal. Los fixtures S24/S29 de
diagnóstico se normalizan (Σ=1) sin cambiar su causa esperada.

---

## 8. Limitaciones y riesgos abiertos

- **AUD-R01** sigue abierto: la indistinguibilidad multicause no se resuelve
  con validación de frontera.
- **F03/F04/F11–F14/F16/F18/F19** (R03/R04) siguen fuera de alcance y rojos en
  el oráculo original.
- **K01–K03, G01–G07** vigentes; sin calibración ni validación experimental.
- La continuación de snapshots pre-C2 calientes es **aproximada por diseño**;
  no se reconstruye historia no persistida.
- Metadata de procedencia no autenticada (limitación de confianza).
- Cobertura numérica finita, no fuzzing exhaustivo; host único.
- `friction_overflow` (c finito extremo) se mantiene como estrés legítimo, no
  se rechaza.

---

## 9. Commits y estado Git

| Commit | Contenido |
|---|---|
| `698b2e3` | C2: validación integral engine/thermal/eléctrico + identidad semántica de snapshots. |
| `9021d7d` | C2: dominio diagnóstico y rechazo de observaciones inválidas/parciales. |
| *(docs)* | Este reporte, matriz de fronteras, ADR-040, snapshot-contract, PROGRESS, nota ADR-039. |

- Cadena: `8a7e868 → 23e48eb → edb36cc → caad1cb → 698b2e3 → 9021d7d → (docs)`.
- `remediation/r01-r05` @ `8a7e868` **sin tocar**; `master` y worktrees de
  auditoría/verificación intactos. Sin merge/ff/rebase/squash.
- Working tree limpio tras el commit de documentación.

---

## 10. Recomendación para verificación independiente

Solicitar verificación de C2 con los tres oráculos intactos:

1. Re-ejecutar los 15 controles C2 (esperado 15/15) y confirmar que los cinco
   bloqueos C1-01…C1-05 dejan de reproducirse.
2. Confirmar atomicidad con comparación de `WorkState` completo (incluidos
   ledgers) en los negativos de engine/thermal.
3. Verificar la clasificación semántica: R02 mid-burn ⇒ `approximate_hot`;
   C2 nativo ⇒ exacto; modelo desconocido ⇒ rechazo.
4. Confirmar que el harness adversarial (19/19) y el original (22/32, 10
   R03/R04) permanecen sin cambios, y que el ralentí nominal no cambió.
5. Revisar el alcance de confianza del metadata (C1-G01) y la restricción del
   claim de ADR-039 (C1-D01).

**STOP.** No se inicia R03/R04/R05 ni S31; la integración a `remediation/r01-r05`
requiere autorización independiente del revisor.
