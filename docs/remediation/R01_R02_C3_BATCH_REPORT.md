# Reporte de batch — Remediación R01–R02-C3 (D16SIM)

- **Fecha:** 2026-10-10
- **Base de implementación:** `remediation/r01-r02-c2` @ `a0040c58174a6cf6a2ac84e38605eafd2f584159`
- **Verificación C2:** `verification/r01-r02-2026-10-10` @ `40e75d54ba5b09d33e358b5777314ea2c657740a`
- **Veredicto vigente antes de C3:** `C2_REMEDIATION_REQUIRED`
- **Rama/worktree C3:** `remediation/r01-r02-c3` en `/home/franku/projects/d16sim-remediation-c3` (desde `a0040c5`)
- **Estado del implementador:** `IMPLEMENTED_PENDING_INDEPENDENT_REVIEW`
- **Toolchain:** rustc/cargo 1.99.0, Linux x86_64
- **Destino NO tocado:** `remediation/r01-r05` @ `8a7e868` (sin merge/ff/rebase/squash)

> No se declara `VERIFIED_FIXED` (corresponde al revisor). No se inician
> R03/R04/R05 ni S31. No se usan datos del D16Y7. No se ajustan parámetros
> nominales ni tolerancias de oráculos.

---

## 1. Estado individual C2-01…C2-06

| ID | F-ID | Sev. | Estado implementador | Evidencia |
|---|---|---|---|---|
| **C2-01** | F15 | P1 | `IMPLEMENTED_PENDING_INDEPENDENT_REVIEW` | Inyector (q_ref, p_ref, deadtime, incertidumbre) y levas (duración, at050, alzada, flanco, centerline, LSA) validados; 46→0 aceptaciones no finitas; activos negativos rechazados. |
| **C2-02** | F15 | P1 | `IMPLEMENTED_PENDING_INDEPENDENT_REVIEW` | T=U/(m·cv) + p/ρ derivadas validadas en restore/resume (incl. no-op); estado imposible con hash válido rechazado. |
| **C2-03** | F15 | P2 (regresión C2) | `IMPLEMENTED_PENDING_INDEPENDENT_REVIEW` | Energía térmica evaluada con el estado real; capacidad 1e307/MAX a 0 °C aceptada, a 20 °C rechazada. |
| **C2-04** | F06 | P1 | `IMPLEMENTED_PENDING_INDEPENDENT_REVIEW` | `model` desconocido/incoherente rechazado en v1 y v2; sin `model` se conserva lectura histórica. |
| **C2-05** | F06/F07 | P1 (regresión C2) | `IMPLEMENTED_PENDING_INDEPENDENT_REVIEW` | Nativo sin `pmax` y c2≠0 ⇒ `approximate_hot`; c2=0 ⇒ exacto; R02 mid-burn sigue aproximado. |
| **C2-06** | F15 | P2 | `IMPLEMENTED_PENDING_INDEPENDENT_REVIEW` | −20 °C válida (R finita > 0); bajo cero absoluto rechazado. |

---

## 2. Reproducciones antes/después

| Caso | Antes (`40e75d5` sobre C2) | Después (C3) |
|---|---|---|
| Inyector/leva no finitos (8 campos × 3 × 2 intervalos) | **46 aceptados** | **0 aceptados**, `unchanged` |
| `negative_injector` (q_ref=−3e-6, 0.5 s) | Ok, inyectado=0, RPM 237.08 | **Err** |
| `nan_cam` (centerline NaN, 0.5 s) | Ok, inyectado=0, RPM 289.15 | **Err** |
| `derived_cylinder` (m=MIN_POSITIVE, u=MAX) | `restore_ok=true`, `noop_ok=true`, `unchanged=false` | **`restore_ok=false`, `noop_ok=false`, `unchanged=true`** |
| `finite_actual_thermal` (c=1e307/MAX, T=0 °C) | `resume_ok=false`, `restore_ok=false` | **`resume_ok=true`, `restore_ok=true`** |
| `semantic_negative` unknown/mismatch (v1 y v2) | aceptados (v1) / rechazados (v2) | **rechazados en todos los esquemas** |
| `native_missing_history` (pmax ausente, c2=0.05) | `provenance=null`, ω 215.941 vs 218.373 | **`approximate_hot`**, misma divergencia declarada |
| `cold_battery` (−20 °C, 0.5 s) | Err «battery voltage» | **Ok** |

Resultado global del checker C2: **6/11 → 11/11** (1442 predicados, 752 filas).

---

## 3. Matriz de aceptación y límites explícitos

`docs/remediation/R01_R02_C3_ACCEPTANCE_MATRIX.md` fija por hallazgo los
positivos, negativos y contrafactuales ejecutados.

Límites explícitos:

- No se exige igualdad exacta `model.schema_version ↔ state.schema_version`
  (se toleran valores **conocidos** distintos): el fixture de migración v1 del
  oráculo independiente captura bajo esquema 2 y reetiqueta el payload a 1.
  Desconocidos (999) se rechazan siempre.
- La historia de `pmax` se exige según la fricción activa: c2≠0 ⇒ requerida;
  c2=0 ⇒ irrelevante (control exacto). No se reconstruye presión histórica.
- No se imponen cotas OEM de capacidades, inercia, levas o inyector.
- `friction_overflow` (c finito extremo) sigue siendo estrés legítimo.

---

## 4. Resultados de todas las suites

| Batería | Baseline | C3 |
|---|---|---|
| `cargo test --locked --workspace` | 204/0 | **209/0** (+4 C3, +1 ajuste) |
| `cargo fmt --all -- --check` | OK | OK |
| `cargo clippy … -D clippy::unwrap_used` | exit 0 | exit 0 |
| Harness S30 original (32) | 22/32 | **22/32** |
| Harness adversarial R01–R02 (19) | 19/19 | **19/19** (196 predicados) |
| Controles C1 (15) | 15/15 | **15/15** (150 predicados) |
| Extensión C2 (11) | 6/11 | **11/11** (1442 predicados) |

Los 10 fallos del S30 son exactamente los de R03/R04: 17/18 (F03/F04), 19/20
(F11/F12), 21/22 (F13), 23 (F14), 28 (F16), 31/32 (F18/F19). No hay fallos
nuevos ni ocultos.

---

## 5. Cambios numéricos

**Ninguno.** Ralentí nominal 8 s/dt 50 µs idéntico a C1/C2:
**1047.3038737395177 rpm**, `Q=46146.847889 J`, `W_ind=13286.519774 J`,
residuo `6.95678e-5`. C3 solo añade validación y clasificación; no toca
ecuaciones, parámetros ni trayectorias válidas.

---

## 6. Compatibilidad de snapshots y procedencia

| Productor | model | pmax | Clasificación C3 | Continuación |
|---|---|---|---|---|
| v1 `2541266` | ausente | ausente | legacy | `exact_neutral` / `approximate_hot` |
| v2 S30 `ff09a0d` | ausente | ausente | legacy | idem |
| v2 R02 `8a7e868` | ausente | presente | pre-C2 | `approximate_hot` (mid-burn 1157 J sin maquillar) |
| v2 C1 `caad1cb` | ausente | presente | pre-C2 | `approximate_hot` |
| v2 C2 nativo | conocido | presente | nativo exacto | bit-exacto |
| v2 C2 sin pmax + c2≠0 | conocido | ausente | **aproximado** | `approximate_hot` |
| v2 C2 sin pmax + c2=0 | conocido | ausente | exacto | pmax no interviene |
| model desconocido (999) | presente | — | **rechazo** | — |

Los 25 snapshots genuinos de los 5 productores leen y verifican sus hashes;
`roundtrip_verified` y `continuation_ok` pasan. La metadata (`model`,
`provenance`) sigue **no autenticada** por `state_hash` (C2-L01, documentado).

---

## 7. Estado de los contratos ya verificados

| Contrato | Estado C3 |
|---|---|
| F01 (química por paso) | Conservado (12 corridas, 196008 pasos, cierre ≤2.2e-18 kg) |
| F02 (diagnóstico físico) | Conservado (firma normalizada; AUD-R01 abierto) |
| F05 (JSON/bits/cero firmado) | Conservado (test_04/09 de C1 y adversarial) |
| F07 (replay nativo multicorte) | Conservado (36+18 comparaciones; frontera pmax ahora etiquetada) |
| F08/F09/F10 | Conservados (22 mezclas, 8 líquidos, 33 térmicos) |
| RV07/RV08/RV09 | Conservados (15 ECU + 3 decoder, 10 fitter, 14 dyno) |

---

## 8. Hallazgos nuevos clasificados

- **Ningún defecto nuevo material** dentro del alcance autorizado.
- **Limitación declarada (no bloqueante):** metadata `model`/`provenance` no
  autenticada por el hash del estado; requiere fuente externa fiable.
- **Tolerancia documentada:** modelo conocido con `schema_version` distinto del
  payload (fixture de migración v1) se acepta; se declara explícitamente.
- **Fuera de alcance (R03/R04):** F03/F04/F11–F14/F16/F18/F19 sin tocar.
- K01–K03, G01–G07 y AUD-R01 siguen abiertos; sin calibración ni validación
  experimental; sin fuzzing exhaustivo; host único.

---

## 9. Commits y working tree

| Commit | Contenido |
|---|---|
| `8fc7a59` | C3: validación de leva/inyector, termodinámica derivada, energía térmica según estado real, identidad semántica y presión requerida, dominio de batería. |
| *(docs)* | Este reporte, matriz de aceptación, ADR-041, snapshot-contract, PROGRESS. |

Cadena lineal:
`8a7e868 → 23e48eb → edb36cc → caad1cb → 698b2e3 → 9021d7d → a0040c5 → 8fc7a59 → (docs)`.
`remediation/r01-r05` intacto en `8a7e868`; worktrees de auditoría/verificación
sin cambios; sin merge/ff/rebase/squash. Working tree limpio tras el commit de
documentación.

---

## 10. Recomendación para revisión e integración

Revisión dirigida con criterios fijados (antes de nuevas pruebas):

1. Re-ejecutar el checker C2 (esperado 11/11) y confirmar el cierre de
   C2-01…C2-06 por causa raíz.
2. Verificar atomicidad con `Debug` completo del WorkState en los negativos de
   engine/electrical.
3. Confirmar la política semántica: desconocido ⇒ rechazo; pmax requerido por
   c2≠0 ⇒ aproximado; nativo completo ⇒ exacto.
4. Confirmar que los tres oráculos previos permanecen 22/32, 19/19 y 15/15, y
   que el ralentí nominal no cambió.
5. Emitir la decisión separada:
   **A. `READY_FOR_DEVELOPMENT_INTEGRATION`** (los P0/P1 dentro de alcance
   cierran; residuales P2 registrados con aceptación explícita), y
   **B. `SCIENTIFIC_CONTRACT_CERTIFICATION`** (evaluación separada de F15;
   no se concede automáticamente).

**STOP.** No se inicia R03/R04/R05 ni S31; la integración a
`remediation/r01-r05` queda a decisión del revisor independiente.
