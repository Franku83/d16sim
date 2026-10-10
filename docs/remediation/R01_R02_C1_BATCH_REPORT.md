# Reporte de batch — Remediación R01–R02-C1 (D16SIM)

- **Fecha:** 2026-10-10
- **Base S30 original:** `ff09a0d`
- **Implementación R01–R02 examinada:** `8a7e8686e4e8831fb30d1a71b815e67e7d7139ef`
- **Auditoría original:** `25d528e` (worktree `/home/franku/projects/d16sim-audit`, intacto)
- **Verificación independiente:** `verification/r01-r02-2026-10-10` (commit `1d61437`, worktree `/home/franku/projects/d16sim-verification`, intacto)
- **Rama/worktree C1:** `remediation/r01-r02-c1` en `/home/franku/projects/d16sim-remediation-c1` (desde `8a7e868`)
- **Veredicto vigente antes de C1:** `REMEDIATION_REQUIRED_BEFORE_R03_R04`
- **Estado del implementador:** `IMPLEMENTED_PENDING_INDEPENDENT_REVIEW`
- **Toolchain:** rustc/cargo 1.99.0, Linux x86_64

> Este batch **no** declara `VERIFIED_FIXED` (corresponde al revisor independiente),
> **no** inicia R03/R04/R05 ni S31, **no** usa datos del D16Y7 y **no** ajusta
> parámetros físicos nominales ni tolerancias para obtener verdes.

---

## 1. Estado individual RV01–RV09 y F01/F02/F05/F06/F15

| RV | F-ID | Sev. | Estado implementador | Evidencia |
|---|---|---|---|---|
| **RV01** | F01 | P0 | `IMPLEMENTED_PENDING_INDEPENDENT_REVIEW` | Consumo químico en la misma convención temporal; exceso 1156.15→1.5e-11 J. |
| **RV02** | F02 | P0 | `IMPLEMENTED_PENDING_INDEPENDENT_REVIEW` | Escape instrumentado contabilizado (0.1/0.7/0.2); reparto nulo ⇒ Inconclusive. |
| **RV03** | F05 | P2 | `IMPLEMENTED_PENDING_INDEPENDENT_REVIEW` | `-0.0` conserva bits en los tres inventarios; hash/restore consistentes. |
| **RV04** | F06 | P1 | `IMPLEMENTED_PENDING_INDEPENDENT_REVIEW` | `MigrationProvenance` con esquema/hash/transformación/campos ausentes y continuación etiquetada. |
| **RV05** | F15 | P1 | `IMPLEMENTED_PENDING_INDEPENDENT_REVIEW` | `resume` valida y avanza sobre copia; errores dejan el estado intacto. |
| **RV06** | F15 | P1 | `IMPLEMENTED_PENDING_INDEPENDENT_REVIEW` | `ThermalState::step` valida Kelvin/estado/ambiente y confirma candidato finito. |
| **RV07** | F15 | P1 | `IMPLEMENTED_PENDING_INDEPENDENT_REVIEW` | Decoder/ECU no propagan NaN/Inf; `schedule` inhibe actuación. |
| **RV08** | F15 | P1 | `IMPLEMENTED_PENDING_INDEPENDENT_REVIEW` | Fitter valida dimensión/finitud del callback; sin panic. |
| **RV09** | F15 | P2 | `IMPLEMENTED_PENDING_INDEPENDENT_REVIEW` | Dyno rechaza factor de corrección no finito. |
| **RV-D01** | — | P3 | `FIXED_IN_DOCS` | Erratum de conteo en el reporte R01–R02 (sección 5). |

Los cinco contratos parciales (F01, F02, F05, F06, F15) quedan cubiertos en el
dominio ensayado por los nuevos contratos adversariales; ninguno se declara
verificado de forma independiente.

---

## 2. Reproducciones antes/después

### 2.1 Contratos adversariales (oráculo byte-idéntico, `check_adversarial.py`)

| Prueba | Antes (8a7e868) | Después (C1) |
|---|---|---|
| `test_02` exceso químico (spark, dt 100/50/25 µs) | 1156.152 / 1156.655 / 1156.557 J | **1.46e-11 / 3.64e-12 / 1.18e-11 J** |
| `test_03` escape instrumentado | reported [0.333, 0.000, 0.667] (escape 70 % descartado) | **reported [0.100, 0.700, 0.200]** |
| `test_04` reparto vacío | `Rings` | **`Inconclusive { have_n: 1, need_n: 2 }`** |
| `test_10` cero firmado | bits `9223372036854775808` → `0` | **bits preservados** |
| `test_13` atomicidad engine | `unchanged=false` (5/5) | **`unchanged=true` (5/5), sin panic** |
| `test_14` restore inválido | 7/7 aceptados | **7/7 rechazados** |
| `test_15` thermal inválido/overflow | Inf y −300 °C mutaban; capacidad extrema mutaba | **rechazo sin mutación (unchanged=true)** |
| `test_16` dyno overflow | `Ok` con factor `Inf` | **`Err`** |
| `test_17` decoder/ECU | tiempo `Inf`, fase `NaN`, 4+4 comandos no finitos | **tiempo/fase finitos, 0 comandos** |
| `test_18` fitter dimensión | **panic** (index out of bounds) | **error tipado, sin panic** |

**Resultado global del oráculo adversarial:** antes 9/19 pasan (174 predicados);
después **19/19 pasan (196 predicados)**, 198 filas, 31 familias, exit 0.

### 2.2 Harness original de auditoría (`check_contracts.py`, 32 casos)

- Antes de C1 (8a7e868): 22 pasan / 10 fallan (89 predicados, 138 filas).
- Después de C1: **22 pasan / 10 fallan (idéntico)**, exit 1.
- Los 10 fallos son exactamente los de R03/R04: 17, 18, 19, 20, 21, 22, 23, 28,
  31, 32. **No hay fallos nuevos ni ocultos.**

### 2.3 F07/F08/F09/F10 (preservación)

| Contrato | Antes | Después |
|---|---|---|
| F07 replay multicorte (test_11 + `native_replay`/`native_multiple_cuts`) | verde | **verde** (36 cortes/horizontes + 18 trayectorias de 3 cortes) |
| F08 especie fresca (test_15 + `fresh_transport`/`fresh_regulator`) | verde | **verde** (18+4 controles) |
| F09 unidades de líquidos (test_16 + `liquid_units`) | verde | **verde** (8 controles Oil/Coolant) |
| F10 balance térmico (test_07/08 + `thermal_ledger`) | verde | **verde** (24 balances + 6 nulos + 3 refinamientos) |

---

## 3. Tests y predicados ejecutados

| Batería | Resultado |
|---|---|
| `cargo test --locked --workspace` | **198 passed / 0 failed** (183 previos + 15 nuevos C1) |
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --locked --workspace --all-targets -- -D clippy::unwrap_used` | exit 0 |
| Harness original (32 casos) | 22 pasan / 10 fallan (R03/R04) |
| Harness adversarial (19 casos) | **19/19 pasan**, 196 predicados, 198 filas |
| Repetición de los 5 modos adversariales | JSONL byte-idéntico (determinismo local) |

Nuevos tests de regresión C1 (15): presupuesto químico por paso, corte de
snapshot en ventana de combustión, combustible insuficiente, cero firmado,
procedencia legacy/nativa, atomicidad de `resume`, restore inválido
(historia/contadores/térmico), atomicidad térmica (estado/ambiente/overflow),
decoder/ECU no finitos, dimensión del fitter, overflow dyno, reparto nulo y
ruta de escape instrumentada.

Conteo honesto: un target no es un caso; una fila no es una assertion. El
workspace tiene 59 bloques de resultado, **29 con casos** en S30 y **30** tras
R01–R02 (RV-D01: la frase «59 targets con casos» del reporte anterior es un
error editorial, corregido en la sección 5).

---

## 4. Cambios numéricos significativos

1. **Combustión (RV01):** la energía se libera igual que antes, pero la masa
   quemada se contabiliza en el mismo paso. El ralentí nominal 8 s (dt 50 µs)
   pasa de **1047.3173 a 1047.3039 rpm** (Δ ≈ 0.013 rpm, 1.3e-5 relativo) por
   el orden de las operaciones; el cierre de energía no cambia (6.9568e-5).
   No se reajustó ningún parámetro.
2. **Migración legacy (RV04):** la continuación de un v1 caliente auténtico
   pasa de 3920.2995 a 3920.2944 rpm por la misma razón (Δ 0.005 rpm). El
   control de pérdida de historia (`missing_history_control`) sigue mostrando
   ~3.671 MPa y ~1.43e-7 kg no recuperables: la continuación legacy se declara
   **aproximada**, no equivalente.
3. **Diagnóstico (RV02):** el reparto de junta ausente en el CLI pasa de
   0.14/0.00/0.86 (rutas cárter/refrigerante) a incluir el escape cuando existe;
   los cuatro escenarios nominales no tienen puerto de escape, por lo que su
   salida no cambia.
4. **Térmico (RV06):** sin cambio en estados válidos; solo se añade rechazo.

---

## 5. Decisiones sobre snapshots y compatibilidad histórica

- **Cero firmado (RV03):** el contrato exige identidad de bits; se preserva el
  bit de signo serializando `-0.0` (el `skip_serializing_if` solo omite el cero
  **positivo**). Los snapshots v1/v2 sin el campo llegan como `+0.0` y siguen
  omitiéndose, por lo que **conservan su hash**. No se canonicaliza ni se
  rechaza el dominio: la garantía documental y la implementación coinciden.
- **Procedencia (RV04):** `EngineSnapshot` incorpora `provenance` opcional
  (`from_schema`, `original_hash`, `transformation`, `missing_fields`,
  `continuation`). Un snapshot nativo completo no lleva procedencia; uno
  legacy registra qué se leyó y qué no existía. Política:
  - estado inicial neutro (t=0, sin combustible/movimiento) ⇒ `exact_neutral`;
  - estado caliente ⇒ `approximate_hot` (la historia ausente no se reconstruye).
  No se presenta un default como medición histórica. La continuidad exacta de
  snapshots nativos completos queda intacta (verificado por `native_replay`).
- **Erratum RV-D01:** «163 passed / 0 failed (59 targets con casos)» debe leerse
  como **163 casos, 59 bloques de resultado, 29 con casos** (30 tras R01–R02).
  Se conserva el texto histórico del reporte anterior y se añade este erratum.

---

## 6. Errores corregidos y posibles regresiones

- Se corrigió un error propio durante la implementación: la validación de
  `q_amb` (energía) se comparaba contra el cero absoluto (temperatura). Lo
  detectó el modo `thermal` del harness adversarial (panic del probe); se
  corrigió y se documentó (los J pueden ser negativos).
- **Regresiones introducidas:** ninguna detectada. Los 183 tests previos siguen
  verdes; el harness original conserva exactamente sus 10 fallos de R03/R04;
  los contratos F07/F08/F09/F10 siguen verdes.
- **Riesgo residual:** el cambio de contabilidad química altera mínimamente el
  orden de operaciones en la ventana de combustión (Δ ~1e-5 relativo en rpm);
  no hay evidencia de inestabilidad, pero el dominio ensayado es acotado.

---

## 7. Hallazgos o límites todavía abiertos

- **AUD-R01:** el caso de vector vacío/escape exclusivo queda corregido, pero la
  sospecha multicause no se declara resuelta; faltan modelos físicos de fuga de
  válvula y una campaña de indistinguibilidad.
- **F13/F18/F19** (R03) y **F03/F04** (R04) permanecen sin tocar.
- **Ledger por tramos:** `th_q_amb_j` se reinicia en `restore` (no está en
  `EngineState`); el replay verificado es del estado dinámico y trazas, no del
  historial contable completo de fronteras. Limitación declarada, sin cambio.
- **Densidad configurable:** `Fluid::Oil/Coolant` mantienen densidades fijas;
  no se añade API nueva.
- K01–K03 y G01–G07 siguen vigentes; no hay calibración ni validación
  experimental.
- La reproducción independiente del revisor está fijada al commit `8a7e868`;
  para re-verificar C1 deberá actualizar su commit objetivo y re-extraer el
  helper de producción (cambio mecánico, oráculo intacto).

---

## 8. Commits y estado del working tree

| Commit | Contenido |
|---|---|
| `23e48eb` | R01-C1: RV01 (presupuesto químico) + RV02 (observación completa) + cambios compartidos de engine para RV04/RV05. |
| `edb36cc` | R02-C1: RV03 (cero firmado), RV04 (procedencia), RV05–RV09 (fronteras atómicas en engine/thermal/ecu/calibration/dyno). |
| *(documentación)* | Este reporte, ADR-039, PROGRESS y actualizaciones técnicas. |

Working tree limpio tras el commit de documentación. Sin merge a `master`;
worktrees de auditoría y verificación intactos.

---

## 9. Recomendación de reevaluación independiente

Solicitar una **nueva verificación independiente** del HEAD C1, con el mismo
oráculo adversarial (byte-idéntico) y el harness original, actualizando el
commit objetivo y la extracción del helper. Puntos a revisar:

1. `test_02` en ventana completa (no solo muestreo cada 0.5 ms) y con
   interrupciones/reflujo.
2. Procedencia de migración: que un v1 caliente quede etiquetado
   `approximate_hot` y que no se anuncie replay físico histórico.
3. Atomicidad de `resume`/`step` con `Debug`/bits antes-después, incluidos
   ledgers.
4. Que el cero firmado no altere hashes de snapshots previos (regresión v1/v2).

**STOP**: no iniciar R03/R04/R05 ni S31 sin autorización explícita.
