# Matriz de aceptación R01–R02-C3

- **Fecha:** 2026-10-10
- **Base C2:** `a0040c58174a6cf6a2ac84e38605eafd2f584159`
- **Verificación C2:** `verification/r01-r02-2026-10-10` @ `40e75d54ba5b09d33e358b5777314ea2c657740a`
- **Criterio:** cada hallazgo C2-01…C2-06 se cierra por causa raíz con positivos, negativos y contrafactuales. Los oráculos no se alteran.

## C2-01 — Parámetros públicos de leva e inyector

| Tipo | Caso | Resultado exigido |
|---|---|---|
| Negativo | `injector_q/p`, `deadtime`, `deadtime_unc`, `cam_lift/duration/flank/center` con NaN/±Inf (8×3×2 intervalos) | `run/restore/resume` Err, `unchanged`, sin panic |
| Negativo | `injector_q = −3e-6` (activo 0.5 s) | Err (no inyectado=0 silencioso) |
| Negativo | `cam_center = NaN` (activo 0.5 s) | Err (no alzada=0 silenciosa) |
| Negativo | `q_ref=0`, `lift=0`, `flank=0`, `duration=0`, `at050>adv` | Err |
| Positivo | Inputs nominales (levas proxy D16A6 + inyector P2A) | `run_engine` Ok |

**Estado: `CLOSED`** (validate_inputs + `validate_cam`).

## C2-02 — Magnitudes derivadas del cilindro

| Tipo | Caso | Resultado exigido |
|---|---|---|
| Negativo | `m=MIN_POSITIVE`, `u=MAX` (T=Inf), hash válido | `restore` Err, `resume` no-op Err, `unchanged` |
| Negativo | Mismo estado con avance 1 ms | Err atómico (ya cubierto) |
| Contrafactual | Estado nominal | `restore`/`resume` Ok |
| Derivadas | `validate_derived_thermo`: T, p, ρ de cilindros y nodos en restore/resume/run_engine | finitas y positivas |

**Estado: `CLOSED`** (T en `validate_physical_state`; p/ρ en `validate_derived_thermo`).

## C2-03 — Energía térmica dependiente del estado real

| Tipo | Caso | Resultado exigido |
|---|---|---|
| Positivo | `c_wall∈{1e307, MAX}`, estado real a 0 °C (energía 0 J) | `resume` Ok, `restore` Ok, step Ok, stored finito |
| Negativo | Misma capacidad, estado a 20 °C (energía T·C=Inf) | `resume`/`restore`/`run` Err, `unchanged` |
| Contrafactual | `cold(20)` de fábrica en `run_engine` | sigue rechazando overflow real |
| Positivo | `thermal_extreme_valid`: capacidad 1e-308/1e300 a 0 °C | step Ok, stored finito |

**Estado: `CLOSED`** (check de `cold(20)` retirado de `validate_inputs`; estado real en `validate_physical_state`/`initial_state`).

## C2-04 — Identidad semántica en todos los esquemas

| Tipo | Caso | Resultado exigido |
|---|---|---|
| Negativo | v1/v2 + `model{schema, 999}` (contabilidad desconocida) | rechazo con hash válido |
| Negativo | v1/v2 + `model{999, 1}` (esquema desconocido) | rechazo con hash válido |
| Positivo | v1/v2 sin `model` (histórico genuino) | lectura/migración según contrato |
| Positivo | `model{schema∈{1,2}, 1}` conocido | aceptado (tolerancia documentada; fixture v1 del oráculo) |
| Contrafactual | `model{2,1}` coherente nativo | continuación exacta |

**Estado: `CLOSED`** (validación de `model` fuera de la rama exclusiva schema2).

## C2-05 — Historia de presión incompleta

| Tipo | Caso | Resultado exigido |
|---|---|---|
| Negativo | Nativo C2 con `pmax=None`, Chen–Flynn `c2=0.05` | `provenance=approximate_hot`, `not same_trajectory`, hash válido |
| Control c2=0 | Misma pérdida de `pmax` con fricción independiente de presión | continuación exacta (pmax no entra en el par) |
| Positivo | Nativo completo C2 (pmax presente) | replay bit-exacto, `provenance=None` |
| Contrafactual | R02 mid-burn genuino (model ausente) | `approximate_hot` (preservado) |

**Estado: `CLOSED`** (regla `native_exact` con historia requerida según fricción).

## C2-06 — Temperatura negativa legítima de batería

| Tipo | Caso | Resultado exigido |
|---|---|---|
| Positivo | `temp_c ∈ {−20, −1, 0, 20, 100}` | `validated` Ok, resistencia finita > 0, R(−20) > R(20) |
| Negativo | `temp_c ∈ {−273.15, −274, NaN, +Inf}` | `validated`/`resistance` Err |
| Contrafactual | Arranque nominal 20 °C | sin cambio de comportamiento |

**Estado: `CLOSED`** (dominio de temperatura separado del de resistencias).

## Criterios de cierre del batch

1. Los 11 casos del checker C2 pasan (antes 6/11).
2. Oráculos intactos: S30 22/32, adversarial 19/19, C1 15/15.
3. Workspace Rust sin fallos; fmt/clippy exigidos en verde.
4. Ralentí nominal sin cambio (solo validación).
5. F01/F02/F05/F07/F08/F09/F10 y RV07–RV09 conservados.
6. Diez fallos R03/R04 siguen identificados.
