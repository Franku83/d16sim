# Verificación independiente R01–R02

**Decisión: `REMEDIATION_REQUIRED_BEFORE_R03_R04`.** Las correcciones solucionan los síntomas de los casos contractuales originales, pero cinco de los nueve contratos todavía requieren remediación o una restricción explícita. No se autoriza ni se inicia R03, R04, R05 o S31. No se modifican gates históricos.

Fecha: 2026-10-10. Producción examinada: **`8a7e8686e4e8831fb30d1a71b815e67e7d7139ef`**, rama `remediation/r01-r05`. Trabajo independiente: `verification/r01-r02-2026-10-10`, `/home/franku/projects/d16sim-verification`. Todos los cambios de esta revisión son artefactos nuevos en `verification/remediation_review/` y `docs/remediation/review/`.

## Resultado por contrato

| ID | Veredicto | Evidencia y resultado | Dominio cubierto y restricción | Impacto en el gate |
|---|---|---|---|---|
| F01 | **PARTIALLY_FIXED** | Caso original 10 verde. Inventarios conservados; cierre absoluto máximo de combustible 2.40e-18 kg en los nuevos controles. Sin embargo, se liberan aproximadamente **1156.6 J con `fuel_burned_kg=0`**, mientras todo el combustible sigue almacenado. RV01; `engine_species`, nuevo caso 02. | Arranque 1 s, con/sin combustible y chispa; inventario marcado inicial; dt 100/50/25 µs; reflujo y mezclas entre nodos. La contabilidad química durante la reacción no cumple el contrato. | R01-F01, spec líneas 61–63, no cerrado; reevaluación del requisito de especies/energía de S16/S30. |
| F02 | **PARTIALLY_FIXED** | Desaparece el `match LeakScenario` del camino de medición. Los cuatro demos ejecutan y el reparto nominal viene de flujos. Un escape instrumentado recibe 70% del caudal físico y se reporta **0%**; un reparto sin observación produce `Rings`. RV02; casos nuevos 03–04. | Convenciones de nodo 0=cámara, 1=cárter, 2=refrigerante; nuevos controles con puerto a otro gas. El helper se extrae byte-exacto y se compila aparte; los demos se ejecutan directamente. No se demuestra independencia causal de dos instrumentos por contar dos interfaces. | R01-F02, spec 70–72, no cerrado; mantener recomendación de reevaluación S24. |
| F05 | **PARTIALLY_FIXED** | Casos 12–13 verdes; 49 campos f64 de un estado físico térmico y de combustible conservan bits; desgaste y extremos finitos también pasan. **`-0.0` pierde su bit de signo** en los tres nuevos inventarios opcionales, con hash igual y restore aceptado. RV03; casos nuevos 09–10. | Parseo real serde/JSON y features del grafo actual. Extremos 1e-200/1e100 se ensayan como valores de tiempo finitos, no como trayectoria física de duración extrema. | El defecto decimal original está corregido; la promesa universal de bits requiere excluir/normalizar cero firmado explícitamente o preservarlo. No demuestra un error de torque. |
| F06 | **PARTIALLY_FIXED** | Caso 14 verde. Un snapshot caliente generado con **el código/tipo histórico `2541266`** verifica, migra y continúa. Alteración/hash inválido/esquema desconocido/campo obligatorio ausente se rechazan. Falta procedencia persistida y una limitación explícita para historia ausente. El control de eliminación de campos nativos cambia la continuación **3488.46→3678.26 RPM** con c2=.05. RV04. | Integridad histórica y ejecutabilidad demostradas; equivalencia de trayectoria histórica **no demostrada**. El control de eliminación es v2 nativo y se identifica como tal, no como snapshot v1 histórico. | R02-F06 pide procedencia, migración explícita y rechazo de ambigüedad. Aceptar un hash no reconstruye inventarios ausentes. No puede cerrarse la compatibilidad física caliente sin limitar su alcance. |
| F07 | **VERIFIED_FIXED** | Caso 11 verde. **54 comparaciones** nuevas: 36 corte/horizonte + 18 trayectorias con tres cortes, incluyendo c2=0/.01/.05, thermal on/off y controles sin combustible/sin chispa. Coinciden hash, todos los campos físicos serializados y cada muestra de la traza. | dt 50 µs; cortes .173/.5/.913 s y .173/.501/.913 s; horizontes 1.2/1.5/2 s. Validez para snapshots **nativos completos**, parámetros y dt idénticos. No extiende equivalencia a snapshots legacy ni a todos los entornos. | R02-F07 satisfecho en el dominio ejecutado. El ledger de fronteras no es una historia dinámica; su reinicio se registra como restricción. |
| F08 | **VERIFIED_FIXED** | Caso 15 verde; 18 pruebas de mezcla con flujo en ambos sentidos, más 4 de suministro/venteo. Máximo error absoluto del invariante cerrado **1.39e-17 kg**; mezcla receptora **4.44e-16**. | Red dirigida; la inversión se representa con dos puertos opuestos. Mezclas 0/1, .2/.7 y 1/0; dt 2/1/.5 ms. No se afirma que un único puerto dirigido sea bidireccional. | R01-F08 satisfecho; no integra por sí mismo fugas con motor runtime (K01). |
| F09 | **VERIFIED_FIXED** | Caso 16 verde; 8 controles con Oil/Coolant, ambos sentidos y dt 1/.5 ms: mismo volumen **1e-8 m³**, masas **8.7e-6/1.06e-5 kg**. Error relativo <1e-12. | Densidades implementadas: **870 y 1060 kg/m³**, constantes positivas ilustrativas. No existe parámetro público de densidad: densidades arbitrarias/inválidas quedan fuera de esa API y no se finge haberlas ensayado. | Conversión dimensional corregida en el dominio disponible. Para introducir otra densidad/fluidología se exigirá una frontera validada; no se solicita esa funcionalidad aquí. |
| F10 | **VERIFIED_FIXED** | 24 balances locales/globales positivos y negativos con fuentes, 6 controles nulos/adiabáticos, más 3 refinamientos contra solución exponencial independiente. Error global máximo **4.69e-9 J**; error analítico disminuye aproximadamente a la mitad al reducir dt. El harness original confirma reducción del residuo integrado en thermal-long. | T inicial 0/20/84/100 °C, ambiente 20/120 °C, termostato cerrado/en banda/abierto; dt .1/.05/.025 s; red engine habilitada/deshabilitada. Restricción: ledger global de corrida **ininterrumpida**, no concatenación automática de ledgers restaurados. | R01-F10 satisfecho para el ledger implementado; estados inválidos/rollback corresponden al F15 pendiente. |
| F15 | **PARTIALLY_FIXED** | Casos originales 24–27/29–30 verdes y siete nuevos controles de manifiesto verdes. Persisten estados imposibles aceptados por restore, rechazo con mutación, NaN convertido en fricción cero, comandos ECU no finitos, overflow dyno y panic del fitter por dimensión. RV05–RV09; casos nuevos 13–18. | Fronteras públicas engine/thermal/dyno/decoder/ECU/calibration/restore. NaN/Inf por API de memoria se distinguen de JSON, que rechaza `null` donde exige f64. No es fuzzing exhaustivo. | R02-F15 exige validación antes de mutar y errores tipados por API; no cerrado. F13/F18/F19 siguen en R03 y no se corrigen. |

## Identidad, aislamiento y fuentes

Se comprobaron los commits reales y sus padres, no solo los nombres reportados:

| Commit | Padre | Identidad |
|---|---|---|
| `ff09a0d` | `4624edf` | Base S30 original |
| `25d528e` | `ff09a0d` | Auditoría original |
| `2a42df6` | `ff09a0d` | R01 fugas/diagnóstico/thermal |
| `f4dd610` | `2a42df6` | Especies engine, persistencia y fronteras |
| `8a7e868` | `f4dd610` | Reporte/ADR/documentación del batch |

La remediación parte de S30, **no del commit de auditoría**. Que `git diff 25d528e..8a7e868` muestre documentos de auditoría ausentes no demuestra su borrado: no pertenecen a esa ascendencia. El worktree original de auditoría conserva sus seis entregables byte-idénticos. El diff pertinente de producción es `ff09a0d..8a7e868`, guardado en [production.diff](results/production.diff).

Estados iniciales y referencias completas: [git-initial.json](results/git-initial.json). Auditoría y remediación estaban limpias. Master tenía una spec **no tracked** preexistente, registrada y preservada. No se escribe en ninguno de esos tres worktrees.

Se inspeccionaron la spec R01–R05, el reporte R01–R02, los seis entregables originales, Foundation v0.2, EVIDENCE_POLICY, DoD, snapshot-contract, las cuatro fichas físicas solicitadas y ADR-038. Las especificaciones posteriores se leen solo para atribuir alcance. [original-audit-integrity.json](results/original-audit-integrity.json) registra los hashes de los seis entregables. El informe no reescribe sus conclusiones ni adopta automáticamente sus sospechas como defectos.

## Integridad del harness

El `check_contracts.py` original y el adaptado por el implementador son byte-idénticos: SHA-256 **`b027d9e3ab697b1974e87980765b7b4872e24e3792153ec3e02b598bd0756b36`**. Cargo.toml/Cargo.lock originales también coinciden. El main del implementador solo añade cuatro campos neutros y un comentario; no cambia oráculos, expectativas, tolerancias o condiciones iniciales de masa/energía/control. Diff y hashes: [harness-integrity.json](results/harness-integrity.json).

Esta revisión reconstruye el original desde Git y usa una adaptación propia de exactamente cuatro campos en `cold()`: inventarios cero y `pmax_pa=None`. La ausencia de historia en un arranque frío deriva la presión inicial del estado físico, como el Core corregido; no se suministra un valor para forzar la igualdad de RPM. La corrección del manejo de presión del solver es parte de F07 y se evalúa como tal.

El original se conserva adicionalmente en `verification/remediation_review/reference/original_main.rs`. El Cargo.toml de revisión añade únicamente un bin de probes separado. El main/oráculo originales permanecen separados de los nuevos probes. El reproductor verifica automáticamente los bytes y la adaptación antes de ejecutar. Su ubicación final evita depender de la copia ignorada `/audit/` del implementador.

## Ejecución y cobertura real

| Batería | Resultado independiente |
|---|---|
| S30 original, workspace desde `git archive ff09a0d` | **163 passed / 0 failed** |
| Remediación exacta, `cargo test --locked --workspace` | **183 passed / 0 failed** |
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --locked --workspace --all-targets -- -D clippy::unwrap_used` | exit 0; warnings permitidos distintos de unwrap permanecen registrados |
| Harness original sobre S30 | **32 casos: 9 pasan / 23 fallan**, 88 predicados ejecutados, 138 filas |
| Harness original adaptado sobre remediación | **32 casos: 22 pasan / 10 fallan**, 89 predicados ejecutados, 138 filas |
| Nuevos contratos adversariales | **19 casos: 9 pasan / 10 fallan**, 174 predicados ejecutados, 198 filas en 31 familias |
| Repetición de cinco modos nuevos | JSONL byte-idéntico en el mismo host |
| CLI de producción, cuatro `mech-demo` | Ejecutables, exit 0; trazas conservadas |

Los diez fallos del harness original son exactamente 17/18/19/20/21/22/23/28/31/32. Su atribución a R03/R04 es correcta. **Los diez fallos nuevos son otros casos**, que revelan cobertura insuficiente de R01/R02. No se exige resolver los primeros antes de ejecutar el siguiente batch; el impedimento es el incumplimiento residual del batch revisado.

Un target no es un caso, una fila no es una assertion, ni un conteo de assertions es cobertura científica. Los logs del workspace contienen 59 bloques de resultados, **30 con casos tras remediación y 29 en S30**; muchos son doc-tests vacíos. La frase del reporte «59 targets con casos» requiere corrección editorial, no altera los 163/183 casos reproducidos.

Los nuevos oráculos incluyen invariantes de especie en una red cerrada, presupuesto químico, balance global de capacidades térmicas, solución exponencial de enfriamiento y comparación de trayectoria completa. Los experimentos usan datos numéricos de control `SYNTHETIC_SELFTEST`, **no evidencia experimental**. La prueba del helper diagnóstico es una extracción byte-exacta, contrastada contra el source de producción en cada ejecución; se declara esta técnica y no se presenta como una llamada pública del CLI a una red arbitraria.

## Restricciones y conclusión técnica

Se verifican correcciones útiles de advección, unidades, calor ambiente, hash histórico y presión persistida. El nuevo valor nominal 1047.317 RPM se reproduce frente a 1025.818 RPM; el inicializador `base_inputs()` es byte-idéntico y los crates de parámetros nominales no cambiaron. No se obliga a conservar una trayectoria dependiente de defectos antiguos.

Permanecen abiertos el latch/presupuesto químico transitorio, la observación del escape/información incompleta, el alcance físico de migración legacy y fronteras fail-closed. El cero firmado compromete la garantía de bits, con impacto físico no demostrado. No hay evidencia de una nueva regresión P0 del transporte en los controles ejecutados; sí existe un nuevo defecto en el helper observacional de R01. No se reclama ausencia universal de regresiones.

Un único Linux x86_64 con Rust/Cargo 1.99.0; builds dev para tests workspace y release para probes; ningún segundo host/arquitectura. No se mide vida útil, se calibra, se valida experimentalmente el D16Y7 ni se demuestra estabilidad universal. No se ejecuta una campaña completa de combinaciones de averías, fuzzing, todos los fluidos o todas las versiones históricas. Las 36 comparaciones de un corte más las 18 de tres cortes verifican el dominio descrito, no cualquier configuración posible. K01–K03 y G01–G07 siguen vigentes.

La revisión se entrega completa con restricciones explícitas y contratos rojos preservados. Véanse [FINDINGS.md](FINDINGS.md), [REPRODUCTION.md](REPRODUCTION.md), [REGRESSION_ASSESSMENT.md](REGRESSION_ASSESSMENT.md) y [BATCH_GATE_DECISION.md](BATCH_GATE_DECISION.md). La fase siguiente debe corregir o acotar los contratos señalados antes de solicitar nuevamente autorización; no se implementa aquí.
