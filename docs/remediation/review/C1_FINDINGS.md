# Hallazgos independientes C1

Producción: `caad1cbf31d01531e0fbd81be6190f52b2d1c2d7`. Categorías de auditoría, sin sustituir estados científicos del Core. No se corrige código. Cada comando se ejecuta en el worktree C1 preparado según [C1_REPRODUCTION.md](C1_REPRODUCTION.md); `D16_REVIEW_RESULTS` apunta a una ejecución con fixtures históricos regenerados.

## C1-01 — Validación incompleta de Engine y confirmación de estados incompatibles

- **Categoría/severidad:** `CONFIRMED_DEFECT`, P1. **RV/contrato:** RV05/F15, parcialmente corregidos.
- **Subsistema/archivo/símbolos/líneas:** `crates/engine/src/run.rs`, `validate_inputs`1246–1316, `validate_physical_state`609–721, `resume`1324–1354, `drive`1107–1117; `NodeW` contiene v/cv públicos. `run_engine`1230–1240 confirma auditoría sin verificar finitud del resultado.
- **Afirmación comprometida:** validación completa antes de mutaciones, errores de configuración e incompatibilidades de estado. Spec F15, ADR-039 punto5. No compromete por sí solo el balance nominal ni demuestra una nueva regresión P0.
- **Reproducción:** `cargo run --locked --offline --release --manifest-path verification/c1_review/Cargo.toml --bin c1-controls -- boundary`. Familias `extended_engine_config`, `extended_work_state`. Comparación anterior: `cargo run --locked --offline --release --manifest-path verification/c1_review/r02_producer/Cargo.toml --bin r02-boundary-control` (crates archivados desde8a).
- **Esperado:** NaN/Inf/configuración físicamente inválida y estado incompatible producen error tipado antes de rebasar ledgers; WorkState completo permanece idéntico. Fallo de starter no se presenta como torque físico cero.
- **Observado:** AFR/ηv/avance/LHV/tensión/dwell/cutout NaN, densidad negativa y starter con torque NaN se aceptan en corrida1ms; `resume` altera estado/ledgers. `inertia.rotating_j_kg_m2=NaN` acepta run/resume/restore; run devuelve cierre energético no finito. `resume(t_end=time)` acepta v=0, v/cv NaN y ausencia térmica incompatible, rebasa ledgers; cvNaN produce `u_sys_0_j=NaN`. Una configuración térmica en inputs se ignora en resume si WorkState no tiene red.
- **Causa confirmada:** el validador omite control/propiedades de fuel/inercia/starter/estado de nodos y coherencia térmica; `.unwrap_or(0.0)` permanece en llamadas de starter; el candidato no se valida completo al confirmar. `f64::max` puede ocultar NaN en velocidad. Validar únicamente los campos de los cinco negativos antiguos no cumple el contrato ampliado.
- **Impacto:** aceptaciones inválidas, síntomas artificiales de falta de arranque, auditorías no finitas e inconsistencias de configuración. El caso de no-op no demuestra avance físico corrupto, pero sí omisión de la validación pública y mutación del ledger sobre entrada inválida. No se exige rechazar torque accesorio negativo sin contrato.
- **Recomendación:** completar matriz por campo/API, validar configuraciones y coherencia WorkState/inputs antes del rebase, propagar errores de starter, confirmar candidato y auditoría finitos. Mantener el rollback demostrado en `runtime_error_atomic`; no ajustar parámetros.
- **Regresión propuesta:** matriz NaN/±Inf/cero/negativos para esos campos, estado térmico on/off incompatible, v/cv inválidos, snapshot histórico válido y errores surgidos después de trabajo parcial. Comparar estado completo con ledgers antes/después.
- **Estado de evidencia:** reproducible C1 y8a; `c1-results/controls-boundary*.jsonl`. Defecto **residual**, no introducido por C1. Casos originales de fricción NaN y errores dinámicos sí quedan atómicos.

## C1-02 — Restore acepta parámetros térmicos inválidos

- **Categoría/severidad:** `CONFIRMED_DEFECT`, P1. **RV/contrato:** RV06 y RV05/F15.
- **Archivo/símbolos/líneas:** `crates/engine/src/run.rs`, `WorkState::restore`438–550, validadores609–721/1246–1316; `ThermalParams::validated`, `crates/thermal/src/network.rs`74–114.
- **Gate:** separación entre hash válido y estado/configuración válida; capacidad cero y no finitos deben rechazarse antes de división/solver. No es un nuevo requisito de datos externos.
- **Reproducción:** modo `boundary`, filas `thermal_restore_config`. Snapshot frío genuino del fixture, con `thermal:Some` en estado e inputs, hash válido; sustituir solo c_wall por0 o wall_frac porNaN. La presencia térmica coincide: el experimento descarta un simple mismatch como causa del rechazo.
- **Esperado:** `WorkState::restore` devuelve Err, nunca un objeto configurado para evolucionar con capacidad nula o fracción NaN.
- **Observado:** las dos restauraciones devuelven Ok. Una tercera con c_wall=MAX también se acepta con `th_u0_j=Inf`. Mismo resultado sobre8a. Las temperaturas NaN/Inf/≤cero absoluto y los contadores inválidos anteriores sí se rechazan en C1.
- **Causa confirmada:** `validate_inputs` no llama al validador térmico; `restore` construye `(ThermalState,ThermalParams)` y calcula stored_j antes de validar las capacidades/fracción. El validador de WorkState comprueba temperaturas, pero no parámetros ni th_u0_j.
- **Impacto:** frontera restore no fail-closed; snapshot autenticado puede quedar asociado con un modelo inválido. Aunque `step` posterior rechace c_wall0, no corrige la aceptación previa ni garantiza integridad de energía inicial.
- **Acción:** validar ThermalParams y cantidades derivadas en restore y resume antes de devolver/cambiar estado. Definir límites representables de cálculo sin inventar cotas físicas de material.
- **Regresión:** snapshots de ambas versiones, red on/off, capacidades0/NaN/Inf/tiny/MAX, wall_fracNaN/fuera[0,1], validaciones previas a construcción y casos válidos cold/hot.
- **Evidencia:** `controls-boundary.jsonl`, `controls-boundary-r02.jsonl`, checker caso07 rojo. Residual F15 demostrado, no regresión C1.

## C1-03 — Protocolo diagnóstico discrimina con observaciones inválidas o incompletas

- **Categoría/severidad:** `CONFIRMED_DEFECT`, P2. **RV/contrato:** RV02/F02 parcialmente corregido en la interfaz pública del protocolo.
- **Archivo/símbolo/líneas:** `crates/diagnostics/src/mech.rs`, `diagnose_low_compression`319–371; `LeakSplit`281–289. La construcción física del CLI (`observable_chamber_split`, `crates/cli/src/main.rs`1663–1703) está corregida.
- **Afirmación/gate:** ausencia/insuficiencia de observación debe producir salida limitada; mediciones incompletas no deben asignar causa única. No se ha reintroducido `LeakScenario`, ni se afirma un defecto del reparto .1/.7/.2.
- **Repro:** `cargo run --locked --offline --release --manifest-path verification/c1_review/Cargo.toml --bin c1-controls -- diagnostic`. Comparar `diagnostic_incomplete` y `diagnostic_invalid`.
- **Esperado:** None/vector vacío/inventario incompleto/observaciones no finitas no cuentan como evidencia suficiente; Inconclusive o error de dominio.
- **Observado:** None y vector vacío están corregidos. `compression=NaN`, `leak_pct=Inf`, split[2,−1,0] y split[.1,0,0] devuelven `Discriminated { cause:Rings }`. La última firma solo representa10% del total: no cierra físicamente. El protocolo no posee una política declarada para validar cobertura/fracciones, y cuenta esa firma como segundo instrumento suficiente.
- **Causa:** contar `Option::Some` y una suma positiva no verifica finitud, [0,1], normalización ni cobertura; las comparaciones NaN no disparan el rechazo por umbral. C1 añadió únicamente filtro para vector vacío. Ausencia de validación ya visible en8a por comparación de fuente; no se afirma una ejecución histórica de este bin diagnóstico nuevo sobre8a.
- **Alcance:** llamadas directas públicas inválidas/incompletas. Los cuatro CLI nominales suministran lecturas validadas y un reparto físico completo; pasan. Una firma completa válida no garantiza identificabilidad multicause (AUD-R01 sigue sospecha independiente).
- **Recomendación:** validar observaciones y cierre de reparto; hacer explícita cobertura de canales o declarar Inconclusive con observaciones parciales. No incorporar verdad del escenario ni otra firma heurística.
- **Regresión:** vectores0/parciales/negativos/NaN/Inf, compresión/leakPct/umbral no finitos, observación completa70%escape y canales ausentes; causas diferentes con observaciones idénticas deben seguir ambiguas cuando el dominio las admita.
- **Evidencia:** `controls-diagnostic.jsonl`, caso13 rojo; no es regresión material P0 introducida. Bloquea la afirmación completa RV02 exigida por esta misión, aunque el defecto original de acceso a verdad esté corregido.

## C1-04 — Temperaturas finitas con energía almacenada no representable

- **Categoría/severidad:** `CONFIRMED_DEFECT`, P2. **RV/contrato:** RV06/F15; balance finito de frontera, no fallo térmico nominal F10.
- **Archivo/símbolos/líneas:** `crates/thermal/src/network.rs`, `ThermalParams::validated`74–114, `stored_j`141–145, `step`193–262; `engine/run.rs`, `initial_state` y `audit_of`.
- **Repro:** modo `thermal`, caso `capacity_large`; temperatura20°C, c_wall=`f64::MAX` (finito>0), dt.1, Q10J, Pfric3W, ambiente20. Modo `boundary`, `thermal_capacity_overflow`, ejecuta run_engine con la misma capacidad extrema.
- **Esperado:** rechazo tipado/atómico ante overflow de cantidades necesarias para el ledger, o dominio representable explícitamente restringido antes de mutar. No se exige que MAX sea una capacidad de motor real.
- **Observado:** `step` retorna Ok y modifica estado; temperaturas y Qamb finitos, `stored_j=Inf`. `run_engine` devuelve Ok con cierre energético no finito. El retorno no falla por el overflow que sí ensaya el caso antiguo de capacidad diminuta.
- **Causa:** validar temperaturas candidatas y Qamb no valida el producto T·C ni la suma del ledger. Parámetros finitos individualmente no garantizan cantidades derivadas finitas.
- **Impacto:** dominio numérico público permite un ledger imposible; escenario extremo, sin evidencia de daño/ruptura ni regresión en parámetros nominales. El mecanismo existía en8a; la corrección parcial no alcanza este overflow.
- **Acción/regresión:** verificar stored_j inicial/candidato y audit_of o limitar formalmente el dominio representable; test tiny y huge, rollback y control de balances de temperaturas físicamente válidas. No subir tolerancias energéticas para aceptar Inf.
- **Estado:** `controls-thermal.jsonl`, `controls-boundary*.jsonl`, checker08 rojo, reproducción ejecutable y evidencia técnica suficiente.

## C1-G01 — Integridad de metadatos de procedencia no definida

- **Categoría/severidad:** `EVIDENCE_GAP`, P2; observación técnica confirmada, requisito adicional pendiente de definición. La parte de RV04/F06 relativa a generación/persistencia pre-R02 está verificada desde una fuente confiable; el contrato completo queda parcial por C1-05.
- **Fuente:** `EngineSnapshot::verified`, `crates/engine/src/state.rs`146–148, verifica **state**, no `provenance`; restore copia metadata en `engine/run.rs`562–605.
- **Repro:** modo `migration`, `provenance_tamper_control`. Tras migrar un v1 caliente auténtico, cambiar solo original_hash→`forged` y continuation→`exact_neutral`.
- **Esperado por contrato actual de hash:** hash de state no cambia. **Riesgo a decidir:** un consumidor que interprete ese hash como autenticación de procedencia recibe clasificación manipulada.
- **Observado:** verified=true; restore acepta y conserva metadata falsa. La política de hash expresamente cubre state; no se inventa una promesa de firma criptográfica de todo el archivo para convertir este control en un defecto P0.
- **Causa/impacto:** metadata fuera de hash y sin validador semántico. No demuestra replay físico caliente exacto ni error de torque; sí limita confianza en la cadena de procedencia recibida de terceros. Es riesgo nuevo del metadata C1, no una nueva regresión material demostrada del solver.
- **Recomendación:** documentar metadata como no autenticado y definir qué consumidores deben verificar contra fuente externa; considerar validación estructural de SHA/esquema/clase y protección de metadata bajo versión compatible si se exige integridad. No reescribir hashes históricos.
- **Regresión propuesta:** metadata mal formado/manipulado, fichero original confiable, cadena cold→hot que conserva exact_neutral **de su origen**, segunda serialización y continuación. No rechazar arbitrariamente exact_neutral solo porque el estado evolucionó y ahora está caliente.
- **Estado de evidencia:** control informativo preservado; fuera de las assertions del hash de state. No bloquea por sí solo la migración acotada ni se concede autenticidad del metadata.

## C1-D01 — Afirmación transversal de validación antes de mutar demasiado amplia

- **Categoría/severidad:** `DOCUMENTATION_DRIFT`, P2. ADR-039 punto5 y `docs/PROGRESS.md` batchC1 describen APIs validadas antes de mutar sin acotar campos/configuraciones; C1-01/C1-02/C1-04 contradicen esa generalización.
- **Contraste:** spec F15 matriz por API/unidad y observaciones de los nuevos probes. Los reportes19/19 son correctos para sus fixtures; no se acusa falsificación de conteos.
- **Propuesta:** en futura fase autorizada, corregir mecanismos y aportar matriz completa; mientras tanto, añadir restricción explícita al claim de validación. No reemplazar historia ni marcar automáticamente gates antiguos FAIL. Este informe recomienda reevaluar cierre R02-F15.
- **Repro/esperado/observado:** modos boundary/thermal; se esperaba rechazo de entradas inválidas, se observa Ok. Raíz y regresiones propuestas en C1-01/02/04.
- **Estado:** evidencia ejecutable y contraste documental; no se efectúa corrección en esta revisión.

## Referencias que permanecen abiertas

K01–K03 y G01–G07 no son defectos nuevos. AUD-R01 sigue sospecha de identificabilidad multicause, no cerrado por corregir vector vacío. F03/F04/F11/F12/F13/F14/F16/F18/F19 siguen fuera de C1 y rojos en el oráculo original. F17 y deriva documental integral pertenecen a una fase posterior sin autorización. No se degrada automáticamente ningún gate histórico: se recomienda reevaluar requisitos concretos F02/F15.

## C1-05 — Snapshot genuino R02 durante combustión no recibe migración semántica

- **Categoría/severidad:** `CONFIRMED_DEFECT`, P1. **RV/contrato:** RV04/F06 `PARTIALLY_FIXED`; limita también la afirmación F01 para continuaciones de estados anteriores. No es una nueva liberación de energía P0 demostrada en una trayectoria nativa C1.
- **Archivo/símbolos/líneas:** `crates/engine/src/run.rs`, `WorkState::restore`562–605 (`legacy = schema==1 || pmax.is_none()`), `drive`1017–1051 (nuevo consumo por paso); `crates/engine/src/state.rs`, esquema2 y campos sin marcador de versión de contabilidad química.
- **Afirmación/gate:** snapshots completos nativos conservan continuidad exacta; una continuación legacy caliente no debe presentarse sin limitación de alcance. Lectura/hash histórico y continuidad física son garantías distintas. Un esquema/estructura completos no identifican automáticamente la semántica del productor.
- **Reproducción:** productor **real8a** en `verification/c1_review/r02_producer/src/mid_burn.rs`, bin `r02-mid-burn`, `engine::resume` hasta0.35955s pedido,0.3596s integrado. Genera `historical-r02-mid-burn.json`. Leer por API C1 y continuar20ms con modo `migration`; no se reconstruye un DTO ni se borran campos a mano.
- **Esperado:** conservar integridad/compatibilidad de lectura, pero marcar procedencia y alcance aproximado o exigir una transformación explícita de la historia química antigua. No tratar la ausencia de una distinción de solver como prueba de replay físico exacto C1.
- **Observado:** hash genuino `151f2e8b54d7d154bedce8ae3473f3ba4d38831a468d23c40ae22a46b0f150b8` verifica; el productor R02 ya define los tres inventarios nuevos (aquí cero y omitidos legítimamente por la política de serialización); pmax está presente. Cilindro2 tiene burn_accounted=false. `Q − burned·LHV·η =1157.151388065635J` antes y1157.1513880656353J después de continuar. `provenance=None` en restore y nueva captura. Se preserva así la contabilidad antigua sin declarar su semántica distinta.
- **Causa confirmada:** C1 cambia cuándo se consume masa del latch, sin distinguir el productor/semántica de un estado estructuralmente completo8a. Restore solo identifica legacy por esquema1 o historia de presión ausente. La presencia de pmax no identifica la versión del presupuesto químico.
- **Impacto:** procedencia/compatibilidad dinámica incompletas y presupuesto químico absoluto heredado incoherente bajo la lectura C1. No se atribuye a C1 crear el exceso original, ni se exige que el solver anterior defectuoso y el corregido coincidan. El defecto es aceptar esa continuación sin la restricción/migración solicitada. Los nuevos incrementos C1 y los snapshots generados por C1 sí conservan el presupuesto en las pruebas nativas.
- **Recomendación:** versionar la convención de contabilidad o el productor/modelo necesario para interpretar latch/fuel/burned; conservar hashes y lectura histórica. Si la historia por cilindro no permite transformación exacta, registrar continuidad aproximada explícita. No restar1157J arbitrariamente ni inferir material/calibración de un contador global.
- **Regresión:** snapshots genuinos8a antes/durante/después de quemado, controles sin chispa y v1/v2 pre-R02, migración serializada y continuación; ningún estado anterior ambiguo declarado nativo exacto por ausencia de metadata. Mantener replay nativo completoC1 bit-exacto.
- **Evidencia:** `controls-migration.jsonl`, `historical-r02-mid-burn.json`, checker nuevo15 rojo. Compatibilidad semántica **incompleta introducida al cambiar la convención en C1**; el desfase físico/contable de origen era RV01 y no una regresión P0 nueva del solver.
