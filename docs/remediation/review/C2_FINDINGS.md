# Hallazgos independientes R01–R02-C2

Objetivo: `a0040c58174a6cf6a2ac84e38605eafd2f584159`. Veredicto: **C2_REMEDIATION_REQUIRED**. Ninguna corrección productiva ni integración ejecutada. Estos IDs son hallazgos de revisión, no estados científicos nuevos.

## C2-01 — Parámetros públicos de inyector y leva eluden la frontera Engine

- **Categoría/severidad:** CONFIRMED_DEFECT, P1; residual de C1-01/RV05/F15.
- **Localización:** `crates/engine/src/run.rs::validate_inputs`, líneas 1345–1478; `drive`, 942–955 y camino de inyección; `crates/fuel/src/injector.rs::flow_at/pulse_for_mass/deliver`; `crates/valvetrain/src/cam.rs::lift_at`.
- **Contrato:** validación de todos los campos públicos relevantes de EngineInputs, parámetros de combustible y estado físico antes de mutar. La matriz C2 omite `injector` y las levas públicas mutables.
- **Reproducción:** bin `c2-boundaries -- matrix`; casos `injector_q`, `injector_p`, `deadtime`, `deadtime_unc`, `cam_lift`, `cam_duration`, `cam_flank`, `cam_center` con NaN/±Inf, intervalos 0 y 1 ms. Controles activos a 0.5 s: `negative_injector` y `nan_cam`.
- **Esperado:** Err controlado, estado completo intacto; ninguna traducción de configuración imposible a un fallo físico nominal.
- **Observado:** run/restore/resume aceptan NaN en estos ocho campos; resume no-op rebasa ledgers (`unchanged=false`). Caudal de referencia −3e-6 m³/s produce `Ok`, inyectado=0, calor=0, RPM=237.0766724637006. Centerline NaN produce `Ok`, inyectado=0, calor=0, RPM=289.1534526487779. No panic.
- **Causa confirmada:** validate_inputs no comprueba esos miembros. `lift_at(...).max(0)` convierte NaN en alzada cero; un pulso resultante de caudal negativo puede entrar en `BelowDeadtime`. Un resultado final finito no valida la configuración que lo generó.
- **Impacto:** frontera pública configurable de Engine; no se demuestra defecto de conservación ni inestabilidad en parámetros nominales válidos. Es una omisión residual, no evidencia de nueva física errónea introducida por C2.
- **Recomendación/regresión:** validar los campos activos y sus relaciones en todas las entradas públicas, sin cotas OEM inventadas; probar NaN/Inf/caudal o presión inválidos, deadtime y perfil inválido, además de controles válidos. Conservar atomicidad completa y propagación de errores.
- **Evidencia:** `from-git-expanded/new-matrix.jsonl`; nuevo checker caso 02 rojo. Oráculos históricos intactos.

## C2-02 — Restore y resume no-op aceptan temperatura de cilindro no representable

- **Categoría/severidad:** CONFIRMED_DEFECT, P1; residual C1-01/RV05/F15.
- **Localización:** `run.rs::validate_physical_state`, 645–678; `WorkState::restore`, 538–546/592–594; `resume`, 1499–1531.
- **Reproducción:** `c2-boundaries -- focused`, `derived_cylinder`: masa `f64::MIN_POSITIVE`, energía `f64::MAX`, restantes estados nominales, pmax presente. Todos los operandos son finitos y positivos; `u/(m*cv)` es infinito. Snapshot capturado con hash válido. También matriz `cylinder_derived_overflow`.
- **Esperado:** rechazo de la cantidad física derivada no representable antes de devolver estado válido o modificar ledgers.
- **Observado:** `restore_ok=true`, `noop_ok=true`, `temperature_finite=false`, `unchanged=false`. Con avance 1 ms sí se rechaza atómicamente en drive.
- **Causa confirmada:** se comprueban operandos m/u y nodos, no temperatura/presión derivadas en la frontera. Si pmax existe, la presión calculada para defaults no determina rechazo. No-op evita el chequeo dinámico.
- **Impacto:** estados extremos representables por operandos aceptados como físicamente válidos; no afecta los ejemplos nominales ensayados. Viola la distinción hash válido/estado físico válido.
- **Recomendación/regresión:** validar temperatura/presión y productos/divisiones necesarios incluso para restore y avance nulo; tests con operandos válidos, resultado no finito, hash correcto y comparación de todo WorkState.
- **Evidencia:** `from-git-expanded/focused.jsonl`, checker caso 03 rojo. No se introdujeron valores OEM ni se ajustaron parámetros nominales.

## C2-03 — Validación térmica en Engine rechaza energía actual finita

- **Categoría/severidad:** CONFIRMED_DEFECT, P2; resultado REGRESSION_FOUND; C2-04/C1-04/RV06/F15.
- **Localización:** `run.rs::validate_inputs`, 1469–1475, llamado desde restore 436 y resume 1491; `thermal/network.rs::stored_j_checked/step`.
- **Reproducción:** `c2-boundaries -- focused`, `finite_actual_thermal`: c_wall=1e307 o MAX, red y configuración coincidentes, estado térmico 0 °C, energía inicial=0 J; otros parámetros ilustrativos. Step térmico de 100 µs con ambiente20 produce energía finita ~0.066 J. Productor `producer_c1 --bin previous-control` ejecuta iguales estados/API sobre caad1cb.
- **Esperado:** distinguir energía del estado restaurado de energía del arranque de fábrica; aceptar estos estados dentro del dominio representable, o justificar previamente una restricción legítima explícita. No imponer tope OEM arbitrario.
- **Observado:** standalone step acepta y mantiene energía finita; C2 restore/resume rechazan ambos, sin mutar, por «thermal stored energy not representable». C1 restore/resume aceptaban ambos.
- **Causa confirmada:** validate_inputs calcula obligatoriamente `cold(20).stored_j_checked`, apropiado para run_engine desde fábrica, pero ajeno al estado real de restore/resume. La capacidad finita grande cabe con temperatura0; no cabe con20.
- **Impacto:** regresión de dominio de APIs de continuación en extremos; no se demuestra problema térmico nominal ni overflow ocultado. Severidad P2 por alcance, no por facilidad de arreglo.
- **Recomendación/regresión:** separar validez intrínseca de parámetros y energía dependiente del estado; validar estado inicial real en cada entrada. Controles pareados capacidadMAX/T0 (finito) y capacidadMAX/T20 (overflow), locales/globales y atómicos.
- **Evidencia:** focused + previous-c1-controls en `from-git-expanded/`; checker04 rojo. No requiere cambiar tolerancias ni modelos.

## C2-04 — Identidad semántica desconocida/incoherente se ignora en schema1

- **Categoría/severidad:** CONFIRMED_DEFECT, P1; C2-05/C1-05/RV04/F06.
- **Localización:** `run.rs::restore`, 552–570: `schema_version==1` evita validar `snap.model`.
- **Reproducción:** cinco snapshots v1 auténticos del productor2541266, añadir model `{schema_version:1,combustion_accounting:999}` o `{schema_version:999,combustion_accounting:1}` sin cambiar estado/hash; `c2-boundaries -- snapshots`.
- **Esperado:** rechazo fail-closed de identificador semántico desconocido o incoherente, también en v1. Ausencia histórica de model sí debe seguir admitida.
- **Observado:** los diez negativos v1 se aceptan con hash válido; transformación registra incluso «combustion accounting unknown». Los correspondientes cuarenta negativos schema2 sí se rechazan.
- **Causa confirmada:** validación de model está dentro de la rama exclusiva schema2.
- **Impacto:** contradice el contrato explícito C2 de versiones semánticas; no rompe lectura del v1 genuino sin metadata ni demuestra trayectoria experimental incorrecta. Nueva omisión del mecanismo C2, distinta de necesidad de autenticar metadata.
- **Recomendación/regresión:** validar cualquier identificador presente antes de seleccionar migración, sin rechazar snapshots históricos sin marcador. Matriz 1/2 × ausente/conocido/desconocido/incoherente.
- **Evidencia:** `from-git-expanded/new-snapshots.jsonl`, checker08 rojo. No se exige firma criptográfica nueva.

## C2-05 — Marcador C2 no garantiza historia de presión completa

- **Categoría/severidad:** CONFIRMED_DEFECT, P1; resultado REGRESSION_FOUND; C2-05/C1-05/RV04/F06, frontera de F07.
- **Localización:** `run.rs::restore`, 546, 552–576; `engine/state.rs::EngineSnapshot::capture`, 158–174.
- **Reproducción:** `focused/native_missing_history`: productor C2 integrado hasta0.5s con Chen–Flynn c2=.05; quitar únicamente el campo opcional pmax, recapturar con API pública C2 (hash correcto/model2,1); comparar contra snapshot íntegro continuando hasta0.55s. No se modifica ley ni parámetro entre las dos trayectorias. Control idéntico en productor C1.
- **Esperado:** rechazar historia nativa incompleta o registrar continuación aproximada; nunca inferir continuidad exacta solo por model conocido.
- **Observado:** C2 acepta, provenance=null; pmax 5256861.000979931→701715.9266370728Pa; omega final215.94097001258632 frente218.37292220188047rad/s; estados/trayectorias distintos. C1 marcaba la misma pérdida de pmax como `approximate_hot`.
- **Causa confirmada:** `native_model=true` basta para desactivar detección legacy, mientras pmax ausente se reconstruye silenciosamente por default. La estructura sigue permitiendo None.
- **Impacto:** regresión en clasificación de continuidad de un estado incompleto; replay de snapshots nativos completos sí permanece exacto. No es pérdida de bits JSON ni fallo de autenticidad: se recalcula hash legítimo del payload modificado para ejercitar su validez física/semántica.
- **Recomendación/regresión:** exigir historia requerida para declarar nativo exacto, o migrarla con límites explícitos; mantener compatibilidad histórica y replay multicorte de snapshots completos. No inventar presión histórica.
- **Evidencia:** focused y previous-c1-controls; checker09 rojo. El caso R02 genuino de1157J sí queda correctamente aproximado en C2.

## C2-06 — Batería a temperatura Celsius negativa legítima se rechaza

- **Categoría/severidad:** CONFIRMED_DEFECT, P2; residual de frontera eléctrica F15, fuera de los cinco bloqueos principales.
- **Localización:** `crates/electrical/src/battery.rs::resistance`, 102–109: exige `temp_c>=0` en el mismo loop de resistencias.
- **Reproducción:** matrix/active/cold_battery, −20°C, resto nominal; run0.5s devuelve Err «battery voltage». Los inputs y la batería validada admiten el valor finito.
- **Esperado:** temperatura Celsius negativa por encima del cero absoluto es válida; el propio factor de frío del modelo debe poder evaluarla.
- **Observado/causa:** rechazo explícito sin panic; se confunde dominio no negativo de resistencias con temperatura. No es fallback cero.
- **Alcance:** política de frío ya existente antes de C2 (`resistance` sin cambios funcionales); no clasificar como nueva regresión C2 ni como validación OEM pendiente.
- **Recomendación/regresión:** dominio de temperatura absoluta coherente en validated/resistance; control −20°C y rechazo bajo cero absoluto. No se implementó.

## Riesgos y límites separados de defectos

- **C2-L01 — metadata no autenticada:** model/provenance no entran en state_hash. Forjar un marcador conocido sobre R02 puede ocultar origen; el contrato C2 exige verificarlo contra una fuente externa fiable. `verified()` comprueba solamente el estado; no se presenta aquí como autenticación de procedencia ni se exige criptografía nueva. Esta limitación declarada no se usa como defecto bloqueante adicional. Serializar una migración auténtica conserva su provenance, incluso con model actual.
- **AUD-R01:** la firma normalizada completa y dos interfaces no demuestran cobertura instrumental independiente ni identificabilidad multicause. Sigue SUSPECTED_DEFECT/EVIDENCE_GAP previo: no se confirmó una solución por añadir validación. Mezclas de pérdidas pueden seguir clasificándose como HeadGasket. No se duplica este riesgo como nueva regresión C2.
- **Configuración térmica externa:** resume comprueba presencia y valida ambos juegos de parámetros, pero puede conservar valores del WorkState distintos de inputs. El probe lo registra; caller debe mantener configuración para replay. No se etiqueta aquí automáticamente como defecto sin contrato de igualdad adicional.
- `node_energy_overflow` es nombre de estrés: en este fixture concreto el producto sí permanece representable. Su aceptación NO se utiliza como prueba de defecto.
- NaN/Inf en JSON de observaciones de auditoría se representan como null cuando procede; flags de finitud y etiquetas explícitas son la evidencia, no convertir null en valor físico válido.
