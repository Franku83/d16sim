# Matriz de fronteras R01–R02-C2 (Fase A)

- **Fecha:** 2026-10-10
- **Base:** `remediation/r01-r02-c1` @ `caad1cbf31d01531e0fbd81be6190f52b2d1c2d7`
- **Hallazgos que la motivan:** C1-01, C1-02, C1-03, C1-04, C1-05, C1-G01, C1-D01.
- **Regla:** esta matriz se completa **antes** de implementar; distingue validación por campo, invariantes relacionales y validación de resultados derivados. No introduce cotas físicas OEM inventadas; solo dominio representable y coherencia del contrato.

Leyenda: **V** = validación existente; **A** = ausente (a añadir); **D** = derivada (producto/suma); **R** = relacional entre campos.

## 1. Engine — `EngineInputs` (`crates/engine/src/run.rs::validate_inputs`)

| Campo | Unidad / dominio | Invariante | V/A/D/R | Error | Atomicidad |
|---|---|---|---|---|---|
| `dt_s` | s, (0, 200 µs] | finito | V | `Invalid` | antes de mutar |
| `t_end_s` | s, ≥ 0 finito | finito | V | `Invalid` | antes de mutar |
| `record_every` | entero > 0 | > 0 | V | `Invalid` | antes de mutar |
| `combustion_efficiency` | 0–1 | finito ∈ [0,1] | V | `Invalid` | antes de mutar |
| `regulator_dp_pa` | Pa ≥ 0 | finito | V | `Invalid` | antes de mutar |
| `throttle/idle_bypass/tailpipe_area_m2` | m² ≥ 0 | finito ≥ 0 | V | `Invalid` | antes de mutar |
| `accessory_tau_n_m` | N·m finito (signo libre) | finito | V | `Invalid` | antes de mutar |
| `plenum_v_m3`, `exhaust_v_m3` | m³ > 0 | finito > 0 | V | `Invalid` | antes de mutar |
| `wiebe_a` | adimensional > 0 | finito > 0 | V | `Invalid` | antes de mutar |
| `wiebe_m` | ≥ 0 | finito ≥ 0 | V | `Invalid` | antes de mutar |
| `wiebe_duration_deg` | ° > 0 | finito > 0 | V | `Invalid` | antes de mutar |
| `friction` | params finitos ≥ 0 | finito ≥ 0 | V | `Invalid` | antes de mutar |
| `control.afr_target` | adimensional > 0 | finito > 0 | **A** | `Invalid` | antes de mutar |
| `control.eta_v_assumed` | (0, 1] | finito ∈ (0,1] | **A** | `Invalid` | antes de mutar |
| `control.spark_adv_btdc_deg` | ° finito (signo libre) | finito | **A** | `Invalid` | antes de mutar |
| `fuel.lhv_j_per_kg` | J/kg > 0 | finito > 0 | **A** | `Invalid` | antes de mutar |
| `fuel.density_kg_m3` | kg/m³ > 0 | finito > 0 | **A** | `Invalid` | antes de mutar |
| `fuel.stoich_afr` | adimensional > 0 | finito > 0 | **A** | `Invalid` | antes de mutar |
| `charge_volts` | V > 0 | finito > 0 | **A** | `Invalid` | antes de mutar |
| `dwell_s` | s ≥ 0 | finito ≥ 0 | **A** | `Invalid` | antes de mutar |
| `starter_cutout_rpm` | rpm ≥ 0 | finito ≥ 0 | **A** | `Invalid` | antes de mutar |
| `crank_timeout_s` | s ≥ 0 | finito ≥ 0 | **A** | `Invalid` | antes de mutar |
| `starter.0` (motor) | params > 0 + free<stall | finito > 0 | **A** (vía `torque_at(0)`) | `Invalid` | antes de mutar |
| `starter.1.ratio` | > 0 | finito > 0 | **A** | `Invalid` | antes de mutar |
| `starter.2` (batería) | soc∈[0,1], R/params ≥0 finitos | finito | **A** | `Invalid` | antes de mutar |
| `inertia.rotating_j_kg_m2` | kg·m² > 0 | finito > 0 | **A** | `Invalid` | antes de mutar |
| `inertia.recip_mass_kg` | kg ≥ 0 | finito ≥ 0 | **A** | `Invalid` | antes de mutar |
| `thermal` (si `Some`) | `ThermalParams::validated` + energía derivada finita | **D** | `Invalid` | antes de mutar |

## 2. Engine — `WorkState` / nodos (`validate_physical_state`, coherencia)

| Estado | Dominio | Invariante | V/A/D/R | Error | Atomicidad |
|---|---|---|---|---|---|
| `time_s`, `theta_rad` | finitos | finito | V | `Invalid` | antes de mutar |
| `omega_rad_s` | finito ≥ 0 | finito | V | `Invalid` | antes de mutar |
| `pmax_pa` | finito > 0 | finito | V | `Invalid` | antes de mutar |
| cámara `m`, `u`, `fresh`, `fuel`, `burn_latched` | finitos, dominios [0,1]/[0,m] | V | `Invalid` | antes de mutar |
| nodo `m`, `t`, `fresh`, `fuel` | finitos, t>0 | V | `Invalid` | antes de mutar |
| nodo `v` | m³ > 0 | finito > 0 | **A** | `Invalid` | antes de mutar |
| nodo `cv` | J/kg·K > 0 | finito > 0 | **A** | `Invalid` | antes de mutar |
| `prev_local_deg` | finitos | V | `Invalid` | antes de mutar |
| ledgers/contadores | finitos; ≥0 donde físico | V | `Invalid` | antes de mutar |
| `th_u0_j` | finito | **A** | `Invalid` | antes de mutar |
| `therm` (si `Some`) | `ThermalParams::validated` + `stored_j` finito + T > 0 K | **A/D** | `Invalid` | antes de mutar |
| coherencia `therm.is_some() == inputs.thermal.is_some()` | **R** | **A** | `Invalid` | antes de mutar |
| resultado `run_engine` (audit) | `mass/energy closure`, contadores finitos | **A/D** | `Invalid` | estado local (sin mutación externa) |
| resultado `resume` (candidato) | `validate_physical_state` + ledgers finitos | **A/D** | `Invalid` | copia no confirmada |
| starter en `drive` | errores propagados (sin `.unwrap_or(0.0)`) | **A** | `Invalid` | copia no confirmada |

## 3. Thermal — `ThermalParams`, `ThermalState`

| Campo | Dominio | Invariante | V/A/D/R | Error | Atomicidad |
|---|---|---|---|---|---|
| `c_wall/piston/oil/cool` | J/K finito > 0 | V | `Invalid` | antes de mutar |
| `ha_*` | W/K finito ≥ 0 | V | `Invalid` | antes de mutar |
| `wall_frac` | finito ∈ [0,1] | V | `Invalid` | antes de mutar |
| `tstat_lo < tstat_hi` | **R** | V | `Invalid` | antes de mutar |
| `ect_tau_s` | finito ≥ 0 | V | `Invalid` | antes de mutar |
| `stored_j` (derivada T·C) | finito | **A/D** | `Invalid` (consumidores) | — |
| `step`: estado de entrada | T finito > −273.15 °C | V | `Invalid` | antes de mutar |
| `step`: ambiente | finito > −273.15 °C | V | `Invalid` | antes de mutar |
| `step`: candidato (T y `q_amb`) | finito, T > 0 K | V | `Invalid` | candidato no confirmado |
| `step`: energía almacenada candidata | finito | **A/D** | `Invalid` | candidato no confirmado |

## 4. Diagnostics — `diagnose_low_compression` / `LeakSplit`

| Entrada | Dominio | Invariante | V/A/D/R | Política |
|---|---|---|---|---|
| `compression_kpa` | Pa finito ≥ 0 (si `Some`) | finito ≥ 0 | **A** | inválido ⇒ no cuenta como medición |
| `low_threshold_kpa` | Pa finito > 0 | finito > 0 | **A** | inválido ⇒ `Inconclusive` |
| `leak_pct` | % finito ∈ [0,100] (si `Some`) | finito | **A** | inválido ⇒ no cuenta |
| `LeakSplit` fracciones | finitas ∈ [0,1] | **A** | inválido ⇒ no cuenta |
| cierre de fracciones | Σ ≈ 1 (tol 1e-6) | **A/D** | parcial ⇒ no cuenta |
| `None`/vector vacío | — | V | `Inconclusive` |
| disponibilidad de canales | `LeakSplit` = descomposición **completa normalizada**; parcialidad se expresa con `None` | **A** (restricción de dominio) | documentada |

## 5. Snapshots — identidad semántica y procedencia

| Elemento | Dominio | Invariante | V/A/D/R | Política |
|---|---|---|---|---|
| `state.schema_version` | 1, 2 | verificación por esquema | V | v1/v2 históricos conservan hash |
| `state_hash` | SHA-256 del estado canónico | V | autentica **solo el estado** |
| `provenance` (wrapper) | metadata | **A** | no autenticado por `state_hash` (C1-G01) |
| `model` (wrapper) | `schema_version` + `combustion_accounting` | **A** | separa esquema de convención química |
| nativo C2 (model conocido) | continuación exacta | **A** | bit-exacto |
| pre-C2 con `pmax` presente (R02/C1) | ambiguo | **A** | `approximate_hot`/`exact_neutral`, nunca nativo exacto |
| v2 pre-R02 (`pmax` ausente) | legacy | V | procedencia existente |
| v1 (`schema==1`) | legacy | V | procedencia existente |
| modelo/accounting desconocido | — | **A** | rechazo fail-closed |
| metadata manipulada | — | **A** (documental) | no autenticada; verificar contra fuente externa |

## 6. Dependencias y derivadas susceptibles de overflow

- Engine: `j_eff` (inercia), `u_sys` (m·cv·T), `p_cyl` (m·R·T/V), `stored_j` (T·C), `audit` (sumas), `pmax`.
- Thermal: `d_*/c_*` (q/C), `T·C`, `q_amb`.
- Diagnostics: `Σ` fracciones, comparaciones con umbral (NaN no dispara).
- Snapshots: hash sobre JSON canónico; el metadata no entra en el hash.

## 7. Tests positivos/negativos necesarios

- Engine: matriz NaN/±Inf/cero/negativos por campo (solo inválidos), coherencia térmica on/off, no-op, estado imposible, snapshot histórico válido, errores tras trabajo parcial.
- Thermal: capacidades 0/NaN/Inf/tiny/MAX, `wall_frac` NaN/fuera de rango, T frío/caliente/absoluta, `dt` extremo, transferencia ±.
- Diagnostics: firmas completas/incompletas/vacías/negativas/NaN/Inf, etiquetas intercambiadas, ambigüedad.
- Snapshots: v1/v2 históricos, v2 S30, R02 nativo y mid-burn genuinos, C2 nativo, replay multicorte, round-trip, metadata manipulada.

## 8. Restricciones explícitas del modelo

- No se imponen rangos OEM de capacidades/inercia; solo finitud, positividad y coherencia.
- No se alteran parámetros nominales ni tolerancias de los oráculos.
- `friction_overflow` (c finito extremo) es caso de estrés legítimo, no se rechaza por diseño.
- La cobertura diagnóstica se restringe a descomposiciones completas normalizadas; no se inventan canales.
- La procedencia/modelo no están autenticados por el hash del estado (documentado, C1-G01).
