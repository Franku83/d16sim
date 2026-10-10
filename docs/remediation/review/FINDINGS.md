# Hallazgos de verificación R01–R02

Commit `8a7e8686e4e8831fb30d1a71b815e67e7d7139ef`, 2026-10-10. Las etiquetas de auditoría no sustituyen estados científicos del proyecto. Reproducción común: `python3 verification/remediation_review/reproduce.py --output /tmp/d16sim-r01-r02-review` desde el worktree de revisión; exit 1 conserva los contratos incumplidos. Evidencia final ampliada en `results/expanded/`; baseline completo en `results/final/`.

| ID | Categoría | Severidad | F-ID | Resumen |
|---|---|---|---|---|
| RV01 | CONFIRMED_DEFECT | P0 | F01 | Calor de reacción con combustible aún contabilizado como no quemado |
| RV02 | CONFIRMED_DEFECT | P0 | F02 | Escape físico instrumentado ignorado; ausencia de reparto se clasifica |
| RV03 | CONFIRMED_DEFECT | P2 | F05 | Cero firmado pierde bits en inventarios opcionales |
| RV04 | EVIDENCE_GAP + DOCUMENTATION_DRIFT | P1 | F06 | Migración caliente con historia ausente sin procedencia ni límite explícito |
| RV05 | CONFIRMED_DEFECT | P1 | F15 | Engine rechaza tras mutar o acepta configuración/estado inválidos |
| RV06 | CONFIRMED_DEFECT | P1 | F15 | Thermal y restore aceptan temperaturas imposibles o mutan antes de rechazar |
| RV07 | CONFIRMED_DEFECT | P1 | F15 | Decoder/ECU propagan observaciones no finitas hacia actuación |
| RV08 | CONFIRMED_DEFECT | P1 | F15 | Fitter hace panic ante tamaño inválido del modelo |
| RV09 | CONFIRMED_DEFECT | P2 | F15 | Dyno devuelve factor infinito en su dominio público declarado |
| RV-D01 | DOCUMENTATION_DRIFT | P3 | — | «59 targets con casos» contradice logs |

## RV01 — Presupuesto químico durante la reacción

- **Archivo/símbolo:** `crates/engine/src/run.rs:919–943`, `drive`; `CylState::fuel_kg` definido como combustible presente en `state.rs:15–18`.
- **Contrato:** R01-F01, spec:61–63 («ninguna energía química sin masa quemada»); Foundation conservación de especies/calor efectivo. **Veredicto F01: PARTIALLY_FIXED.**
- **Reproducción:** modo `engine`, filas `engine_species` con spark=true; `check_adversarial.py::test_02`. Arranque nominal 1 s, controles cada .5 ms y dt 100/50/25 µs. No se cambian parámetros nominales.
- **Esperado:** el calor liberado consume el inventario químico correspondiente; si todavía se difiere una variable acumulativa por diseño, debe existir un inventario no quemado efectivo y un término químico pendiente coherentes. No se puede sumar calor liberado y combustible aún intacto como energías independientes.
- **Observado:** dt 50 µs, t=.35955 s: Q=**1156.6545077 J**, quemado=**0 kg**, inyectado=almacenado=**5.2861907716e-5 kg**, emitido=0. El refinamiento a 25 µs mantiene **1156.5573194 J** de exceso. El cierre de masa del combustible pasa: precisamente por eso no demuestra cierre químico.
- **Causa confirmada:** Q se integra a partir de `burn_latched` durante la ventana; la resta de `c.fuel` y suma a `fuel_burned_kg` ocurren solo después de `fire_local+duration`. El transporte fue corregido, la coherencia temporal del latch/presupuesto no.
- **Alcance:** estados y ledgers durante combustión/transitorios y snapshots cortados allí. No demuestra por sí solo error de potencia final ni oxidación/AFR reales. El comportamiento de consumo diferido preexistía: es un **residual del contrato ampliado F01**, no una nueva regresión atribuida a la advección.
- **Recomendación/regresión:** consumir especie conforme a la reacción e integrar un presupuesto químico en la misma convención temporal, o definir explícitamente inventarios efectivo/pendiente que lo cierren; controles dentro/fuera de ventana, apagado/interrupción, escape y replay. No bajar tolerancias hasta aceptar ~1157 J como redondeo. El test concede 1 J y aun falla.
- **Evidencia:** ejecutable, tres pasos refinados; contrato no cerrado. La falta de AFR/oxígeno K02 no justifica duplicar inventario químico.

## RV02 — Canales observacionales incompletos

- **Archivo/símbolo:** `crates/cli/src/main.rs:1658–1696`, `observable_chamber_split`; caller `mech_demo_cmd:1730–1740,1765–1766`; `crates/diagnostics/src/mech.rs:317–366`, `diagnose_low_compression`.
- **Contrato:** R01-F02 spec:70–72: observaciones físicas, canal insuficiente explícito, ambigüedad. **Veredicto F02: PARTIALLY_FIXED.**
- **Reproducción:** modo `leak`, `diagnostic_physical_exhaust` y `diagnostic_channels`; casos nuevos 03/04. La función compilada está copiada byte-exacta del commit; `reproduce.py` comprueba su identidad. Red válida con puertos cámara→cárter/refrigerante/otro gas, conductancias 1:2:7; el solver, no una etiqueta, genera los totales.
- **Esperado:** fracciones observadas [cárter/escape/refrigerante] ≈[.1/.7/.2]. Ausencia de reparto instrumental suficiente → Inconclusive/Unsupported o error explícito.
- **Observado:** caudales integrados **[1.16678e-7,2.33356e-7,8.16746e-7] kg**. El helper declara `exhaust_flag=true`, pero reporta **[.33333333/0/.66666667]**. Normaliza solo cárter+refrigerante y descarta 70% de la salida. Caso todo a escape produce reparto cero; con compresión=500 kPa y leak=30% el protocolo declara **Rings**. El caso sin reparto también declara Rings.
- **Causa confirmada:** `match p.to` solo suma 1 y 2; `to_exhaust_01` es siempre 0. La detección posterior del canal no incorpora su masa ni se propaga como disponibilidad al clasificador. Este cuenta `Some(split)` como observación completa y aplica desempate `0>=0`.
- **Alcance:** montaje con escape instrumentado soportado por el propio helper/comentario; observaciones faltantes. Los cuatro escenarios CLI actuales no poseen ese puerto: sus valores nominales y eliminación del acceso a `LeakScenario` sí se verifican. No se atribuye acceso oculto a verdad al uso documentado de índices de roles de nodo.
- **Novedad:** el descarte del escape pertenece al nuevo helper R01; no existía ese helper en S30. La clasificación del vector cero es un defecto preexistente concreto; la sospecha original AUD-R01 sobre todas las combinaciones de causas **no queda completamente resuelta** por este caso. No se afirma una regresión A/B de un escenario CLI que antes no admitía esa topología.
- **Recomendación/regresión:** sumar cada ruta instrumentada con su denominador físico, modelar disponibilidad del canal y transmitirla al protocolo; misma red/caudal bajo nombres distintos, escape exclusivo, cero/incompleto, dos causas con observaciones indistinguibles. No contar dos modelos derivados de la misma área efectiva como evidencia independiente de dos causas.
- **Evidencia:** source y probe ejecutable con solver real, más smoke tests directos de CLI. Fixtures son numéricos; el puerto adicional es un **fixture de control**, no una afirmación de instrumentación experimental disponible.

## RV03 — Serialización de -0.0

- **Archivo/símbolo:** `crates/engine/src/state.rs:55–75`, atributos `skip_serializing_if=is_zero_f64`.
- **Contrato:** snapshot-contract:11–12 promete preservación de bits y hash; R02-F05. **Veredicto F05: PARTIALLY_FIXED.**
- **Reproducción:** modo `snapshot`, `snapshot_signed_zero`, nuevo caso 10; asignar -0.0 a cada uno de `plen_fuel_kg`, `exh_fuel_kg`, `fuel_emitted_kg` de un snapshot físico válido, capture→JSON→deserialize.
- **Esperado/observado:** bits **9223372036854775808**→**0**; restore acepta el original, hash sigue igual. Los positivos de alta precisión y el defecto decimal original sí pasan.
- **Causa confirmada:** `-0.0==0.0` causa omisión y `serde(default)` reconstruye +0.0. El dominio público no rechaza el cero firmado.
- **Alcance/novedad:** defecto de representación introducido por los nuevos campos opcionales; no se ha demostrado impacto dinámico. P2 por garantía bit a bit, sin degradar automáticamente toda la corrección F05 a defecto físico P1.
- **Recomendación/regresión:** preservar bits o definir/rechazar/canonicalizar cero firmado antes de prometer identidad. Ensayar ceros con ambos signos en cada campo opcional y hash/replay. No cambiar el oráculo original.
- **Evidencia:** ejecutable, tres campos; `hash_equal` no implica igualdad de bits.

## RV04 — Alcance de migración física y procedencia

- **Archivo/símbolo:** `crates/engine/src/state.rs:55–68,117–126`; `run.rs:420–537` (defaults, migración térmica a frío y presión como cota inferior); `docs/snapshot-contract.md:6–19,24–29`; ADR-038:25–32; spec R02-F06:118–121.
- **Contrato/veredicto:** **F06 PARTIALLY_FIXED**. El rechazo del hash v1 original está corregido. No se demuestra una migración físicamente equivalente general ni se conserva procedencia de la transformación como exige la spec.
- **Reproducción:** `legacy/` compila el engine y su **tipo v1 real** materializados con `git archive 2541266`; genera snapshot t=.50005 s. `snapshot` comprueba hash, restore, serialización posterior/continuación y cuatro negativos. `missing_history_control` elimina exclusivamente campos nuevos de un snapshot nativo para aislar el efecto de pérdida de información.
- **Esperado:** verificar identidad histórica; registrar versión/hash/procedencia/limitaciones de la migración; recuperar solo historia deducible y declarar desconocida la restante. Si se promete replay físico exacto, deben persistirse todos sus datos.
- **Observado:** v1 auténtico acepta/migra a schema 2. Los inventarios ausentes toman 0, pmax se sustituye por presión instantánea máxima; el nuevo snapshot no registra origen v1/hash previo ni la incertidumbre. El control v2 pierde **3.6710858 MPa** de memoria y **1.43637e-7 kg** de fuel en nodos; continuo 3488.4602 RPM vs continuación sin historia 3678.2635 RPM a 1.5 s.
- **Inferencia y límite:** ese control demuestra la sensibilidad a la información borrada, **no** que un v1 particular debiera coincidir con una trayectoria del solver nuevo. Las ecuaciones corregidas pueden cambiar legítimamente la trayectoria histórica. Los snapshots originales no contienen los nuevos inventarios de nodos; su cero no es una medición. La equivalencia para v1 caliente permanece **inconclusa**, no confirmada falsa por la comparación artificial. La ausencia de procedencia/restricción en el contrato es confirmada documentalmente.
- **Alcance:** snapshots legacy v1 y v2 sin campos R01, especialmente con combustible circulando o c2≠0. El cold-start sin combustible puede acotarse a valores neutros conocidos. El tipo/contrato productor es histórico; su crate de verificación tiene lockfile propio y toolchain actual, no se presume reproducción de un binario histórico archivado.
- **Recomendación/regresión:** contrato de migración histórica separado de replay nativo; conservar procedencia e indicar qué campos se infieren/no se conocen. Rechazar continuación exacta incompatible o devolver restart aproximado explícito con dominio. Probar v1 frío/caliente genuinos y v2 previo; no imponer que un solver defectuoso antiguo y uno corregido den la misma RPM.
- **Evidencia:** fixture auténtico ejecutable y hashes conservados, control informacional y contraste documental; gap crítico sin PASS por falta de evidencia. No es G01 experimental ni se resuelve con calibración.

## RV05 — Frontera engine y mutación parcial

- **Archivo/símbolo:** `run.rs:544–586` validación incompleta; `1140–1220` inputs/resume; `1016–1019` error de fricción convertido a 0; `state.rs` capture/verified.
- **Contrato/veredicto:** R02-F15, validación antes de mutar, errores tipados; **PARTIALLY_FIXED**.
- **Reproducción:** modo `boundary`, `engine_boundary_atomic` y `restore_boundary`; nuevos casos 13/14. Compara el WorkState completo por representación Debug antes/después; marca un ledger preexistente para detectar rebase.
- **Esperado:** rechazar área negativa/Wiebe inválido/NaN de fricción/estado inválido antes de modificar estado o ledger; restore valida también inputs e historia requerida.
- **Observado:** área de tailpipe negativa y Wiebe a=-1 retornan Err **con WorkState modificado**. Fricción viscosa NaN acepta; `heat_released_j=NaN` acepta y continúa. Restore acepta volumen plenum=0, `prev_local_deg[0]=NaN`, emitido=-1 kg y calor NaN con hash autocoherente; también configuración record_every=0, aunque posterior resume la rechaza.
- **Causa:** rebase del ledger precede validación completa; checks de dominio derivados se difieren al solver. Restore omite validate_inputs y campos de historia/contadores. `torque_mag(...).unwrap_or(0.0)` sustituye error por una fuerza diferente.
- **Alcance:** APIs en memoria y consumidores de estados públicos; aceptar NaN en memoria no implica que un JSON literal NaN sea parseable. Para áreas/Wiebe se demuestra mutación del estado completo, **no se infiere de esa comparación cuál componente físico cambió**. El rollback abarca el ledger según la spec.
- **Novedad/recomendación/regresión:** residual de F15, parte preexistente; ampliar matriz por API y preparar avance atómico/error previo, validar memoria física y parámetros. El caso de fricción no requiere arreglar fase ECU ni fitter R03. Cada error debe conservar exactamente el estado de entrada. Evidencia ejecutable; los síntomas originales record=0/ηcomb/masa negativa sí quedaron corregidos.

## RV06 — Thermal y restauración térmica

- **Archivo/símbolo:** `thermal/src/network.rs:179–235`, `ThermalState::step`; `engine/src/run.rs:495–515,544–586`, restore/validación.
- **Contrato:** F15, temperaturas físicas y rollback. No contradice que el ledger F10 ya cierre para estados válidos.
- **Reproducción:** `thermal_invalid_state`, `thermal_invalid_ambient`, `thermal_overflow_atomic`, `restore_boundary`; nuevo caso 15.
- **Esperado/observado:** -300 °C en nodos o ambiente debe rechazarse; **se acepta**. Estado térmico infinito produce Err pero cambia sus componentes a NaN. Capacidad positiva **1e-308 J/K** con 100 J produce Inf y Err **después de mutar**. Restore acepta thermal con pared -300 °C o NaN en memoria.
- **Causa confirmada:** solo se exige ambient_c finito; no se valida estado inicial ni cero absoluto. La validación de resultados se aplica después de asignar temperaturas. Restore reconstruye ThermalState sin validarlo.
- **Alcance:** public API; ejemplo de capacidad extrema no se afirma físicamente plausible, pero demuestra el contrato de overflow/atomicidad. El ejemplo -300 °C es inválido físicamente y es una prueba negativa deliberada. Capacidad cero ya se rechaza correctamente.
- **Novedad/recomendación/regresión:** residual preexistente del contrato F15; validar Kelvin/ECT/state/params antes y verificar estado candidato finito antes del commit. Ensayar NaN/Inf, límites absolutos, capacidad mínima, calefacción/enfriamiento y rechazo sin cambio. Evidencia ejecutable, **P1**.

## RV07 — Decoder y ECU frente a entradas inválidas

- **Archivo/símbolo:** `ecu/src/decoder.rs:105–150,201–219`, update/estimate; `ecu/src/scheduler.rs:130–198`, schedule.
- **Contrato:** F15 por API/observaciones, sin alterar semántica F13. **P1**.
- **Reproducción:** `decoder_boundary`, `ecu_boundary`; nuevo caso 17. Constructor válido, update(NaN), estimate(Inf); scheduler con Synced720 y rpm o load NaN, o fase NaN.
- **Esperado/observado:** error/inhibición explícita de observaciones inválidas. Estimate devuelve tiempo Inf/fase NaN. Scheduler con load/rpm NaN devuelve **4 inyecciones y 4 encendidos**, con magnitudes no finitas. Con fase NaN devuelve comandos sin validar esa entrada (no se demuestra un comando no finito en este último caso).
- **Causa:** new valida flancos y now, pero las llamadas posteriores no validan; SyncEstimate público permite estados no finitos y schedule usa clamp/max/cálculo sin frontera tipada.
- **Alcance:** ECU de referencia genérica, sin integración runtime D16Y7 (K01/G06); no es código OEM ni prueba de daño material.
- **Novedad/recomendación/regresión:** residual F15 independiente de los fallos de anclaje/dropout F13. Validar observaciones en toda llamada pública y fallar cerrado antes de emitir actuación. Tests de flancos nuevos buenos permanecen verdes. Evidencia ejecutable.

## RV08 — Panic del fitter por dimensiones del modelo

- **Archivo/símbolo:** `calibration/src/fit.rs:32–78`, `least_squares`.
- **Contrato:** error tipado por dimensiones y F15 frontera calibration; **P1**.
- **Reproducción:** `calibration_dimension`, caso nuevo 18: `least_squares(|_|vec![], &[1.,2.], &[1.,1.], &[1.], 1e-8)` bajo catch_unwind.
- **Esperado/observado:** FitError::Invalid por dimensión, sin panic. **Panic index out of bounds** en fit.rs:78. El probe lo captura; su exit 0 no significa PASS y el test contractual queda rojo.
- **Causa:** las longitudes de salidas del callback no se validan antes de indexar fp/fm con n=y.len().
- **Alcance:** función pública incluso con configuración numérica finita; no prueba ajuste/identificabilidad experimental. Residual preexistente, distinto de covarianza F18 y descenso F19, que siguen atribuidos a R03.
- **Recomendación/regresión:** validar todas las salidas/dimensiones antes de operaciones; callbacks que cambian longitud, finitos inválidos y cantidades observadas; no modificar aquí ecuaciones/optimización. Siete controles nuevos de Manifest sí rechazan observaciones/metadatos inválidos. Pesos negativos se registraron solo informativamente: no se clasifican como defecto sin contrato explícito, ya que el algoritmo usa W².
- **Evidencia:** ejecutable y stderr con ubicación exacta.

## RV09 — Overflow de corrección dyno

- **Archivo/símbolo:** `dyno/src/brake.rs:37–46,66–80`, Ambient::validated y Correction::of; **P2**.
- **Contrato/reproducción:** F15; `dyno_overflow`, caso nuevo 16. Ambiente con p=1e-308 kPa y T=25 °C, apply=true.
- **Esperado/observado:** factor finito o error de dominio/overflow. Devuelve **Ok con factor Inf**.
- **Causa:** la entrada cumple p finita>0 y T>K0; `99/p` desborda y la salida no se valida.
- **Alcance:** extremo representable dentro del dominio declarado de API, fuera de atmósfera físicamente plausible. No se atribuye defecto a la potencia de un punto nominal. No coincide con F03/F04, que siguen en R04.
- **Recomendación/regresión:** validar resultado o definir límites públicos justificados; probar extremos y apply=false sin reajustar umbrales estacionarios. Residual preexistente de frontera F15; evidencia ejecutable.

## RV-D01 — Conteo documental

`R01_R02_BATCH_REPORT.md:72` dice «163 passed / 0 failed (59 targets con casos)». Los logs originales muestran **59 bloques**, de los cuales **29** tienen casos; después son 30. 163/183 casos sí se reproducen. Fuente autoritativa: logs `results/final/workspace-test-original.log` y `workspace-test.log`. **P3 DOCUMENTATION_DRIFT**. Propuesta: informar casos, bloques/targets con/sin casos y doc-tests separadamente. No se corrige el reporte original ni se alteran gates.

## Riesgos y restricciones sin clasificación de defecto nuevo

- **Ledger después de restore:** `th_q_amb_j` se reinicia en restore y no está en EngineState. En replay térmico se pierden hasta **2.56014 J** de esa acumulación al corte, pero estado físico/traza futuras coinciden. Los balances de fronteras se rebases por segmento; no hay API que prometa concatenación exacta de esos ledgers. Registrar esa limitación para uso de auditorías globales, sin presentar un defecto dinámico F07 ni revertir la corrección F10.
- **AUD-R01 original:** el caso específico de vector vacío/escape exclusivo es reproducible; no se agota la sospecha sobre diagnósticos de múltiples causas. Requiere campaña de indistinguibilidad y disponibilidad instrumental.
- **Densidad configurable:** no existe entrada de densidad en Fluid::Oil/Coolant. Los dos valores positivos se prueban; no se inventa una API para cambiarla ni se asegura rechazo de una densidad imposible de suministrar.
- **K01–K03:** integración runtime parcial, modelos L1 y contacto genérico siguen siendo limitaciones conocidas, no regresiones. **G01–G07:** evidencia externa, estabilidad general, segundo host/expertos, cadenas completas, identificabilidad, electrónica y manifiestos siguen abiertos. Ningún resultado concede calibración o validación.
- No se demuestra una nueva regresión P0 de advección/thermal en el dominio probado; tampoco se afirma ausencia de regresiones fuera de ese dominio. RV02 identifica un defecto nuevo del helper; RV03 un defecto nuevo de representación. Los demás defectos de frontera/reacción son residuales preexistentes o de contrato incompletamente remediado.
