# Verificación científica y técnica independiente R01–R02-C1

**Veredicto: `C1_REMEDIATION_REQUIRED`. No se ejecuta integración.**

Fecha: 2026-10-10. HEAD examinado: `caad1cbf31d01531e0fbd81be6190f52b2d1c2d7`. Destino autorizado, conservado: `8a7e8686e4e8831fb30d1a71b815e67e7d7139ef`. Evidencia anterior: `1d614378dd8481acb2a513b4c6dc6572345e5c66`. Rama de entrega: `verification/r01-r02-2026-10-10`. El SHA de esta entrega se obtiene con `git rev-parse verification/r01-r02-2026-10-10` y se comunica en la entrega final; no se incrusta recursivamente dentro de su propio commit.

C1 corrige los síntomas RV01–RV09 del harness anterior. Se reproducen los 198 tests y 19 contratos aprobados del implementador. Sin embargo, las fronteras ampliadas demuestran configuraciones inválidas aceptadas, restauración con capacidad cero y balances no finitos. El protocolo diagnóstico público sigue discriminando con observaciones no finitas o un reparto incompleto. Un snapshot R02 genuino durante combustión se restaura sin procedencia, aunque arrastra una contabilidad química distinta de C1. Estos defectos y la omisión de migración semántica impiden cerrar todos los contratos exigidos. No hay nueva regresión física P0 demostrada.

## Integridad y fuentes

Se contrastaron la spec R01–R05, ambos reportes del implementador, los cinco documentos de la revisión anterior, ADR-038/039, Foundation, Evidence Policy, DoD y snapshot contract. Se revisaron diffs reales, implementaciones y llamadas Engine/Thermal/CLI/Diagnostics/ECU/Calibration/Dyno. Las fuentes históricas de auditoría siguen en `25d528e`, S30 en `ff09a0d`, el productor v1 en `2541266`. Los seis documentos originales y los cinco reportes anteriores no se modificaron.

La secuencia es `8a7e868 → 23e48eb → edb36cc → caad1cb`, con padres únicos y completos en [git-review-state.json](c1-results/git-review-state.json). C1 y destino no divergen. Ambos worktrees de implementación estaban limpios y permanecen sin cambios. El worktree principal conserva un archivo de spec no rastreado preexistente; no participa en la integración ni fue alterado.

El worktree de ensayo `/home/franku/projects/d16sim-verification-c1` está detached en C1, separado del implementador. La rama del revisor permanece basada en `1d61437`/producción `8a7e868`: solo añade documentos, probes y resultados. Sus dependencias para C1 se materializan en el worktree de ensayo; ejecutar los probes directamente sobre los crates de la rama revisora ensayaría la versión anterior.

Los dos oráculos son byte-idénticos; también lo son los experimentos originales y los de verificación anterior. El main conserva exactamente los cuatro campos neutros de R01. El helper diagnóstico se extrajo byte-exacto desde C1. Únicamente el objetivo del runner, la selección del productor original al añadir un bin auxiliar y la extracción del helper se adaptaron; no cambiaron expectativas, tolerancias ni condiciones de esos experimentos. [independent-integrity.json](c1-results/independent-integrity.json) registra hashes y referencias. Los nuevos controles tienen un checker separado.

`base_inputs` sigue byte-idéntico desde S30; no cambian los crates/constantes de parámetros físicos nominales. Foundation, Evidence Policy y DoD son idénticos a S30. PROGRESS y reporte R01–R02 reciben únicamente adendas del implementador; los gates históricos no se reescriben. La revisión no modifica producción, parámetros ni documentos históricos.

## Resultado por RV-ID

| RV / contrato | Veredicto independiente | Evidencia, dominio y límites |
|---|---|---|
| RV01 / F01 | `VERIFIED_FIXED` | 12 corridas de presupuesto por paso, nominal/corte de futuras chispas y combustible/escasez/reflujo, dt100/50/25µs; consumo medido por diferencia de inventarios, no solo por el contador de quemado. 36 cortes durante pasos reactivos reproducen estado y traza. Referencia integral analítica Wiebe para η=.25/.5/1 y cinco pasos temporales. No demuestra combustión química detallada ni oxígeno/AFR calibrado. |
| RV02 / F02 | `PARTIALLY_FIXED` | Ruta real con 70% a escape da .1/.7/.2; cierre e independencia de etiquetas verificados. None y vector vacío dan Inconclusive. El protocolo público acepta NaN/Inf, fracciones negativas y vector con suma .1 y produce Rings; C1-03. El camino CLI de instrumentos válidos sí funciona. AUD-R01 multicause permanece abierto y es distinto de este defecto de frontera. |
| RV03 / F05 | `VERIFIED_FIXED` | +0 y −0 conservan bits, hash, JSON y restore para tres campos opcionales; pasan 49 floats físicos, positivos de precisión alta y wear. Compatibilidad real v1/v2 históricos y snapshot nativo producido con 8a. No se afirma portabilidad bit-exacta entre arquitecturas. |
| RV04 / F06 | `PARTIALLY_FIXED` | Productores genuinos v1 (`2541266`) y v2 S30 generan cold/hot. Procedencia recoge esquema/hash/transformación/ausencias, exact_neutral o approximate_hot, y persiste en serialización/continuación. Integridad histórica ≠ continuidad del solver histórico: hot explícitamente aproximado. Un snapshot genuino8a con pmax y campos de fuel presentes, cortado a0.3596s, mantiene 1157.15J de exceso químico sin procedencia; C1-05. Metadatos de procedencia no autenticados: C1-G01; la parte verificada es generación/persistencia v1/v2 pre-R02 desde una fuente confiable, no toda compatibilidad semántica R02 ni protección criptográfica del metadata. |
| RV05 / F15 | `PARTIALLY_FIXED` | Los cinco negativos anteriores y dos errores dinámicos de flujo/térmico rechazan sin mutar WorkState completo y ledgers. Pero NaN en control, combustible, starter/inercia y WorkState inconsistente se aceptan; inercia NaN devuelve RunTrace con cierre energético no finito. C1-01. Atomicidad cuando se retorna Err está corregida; validación completa no. |
| RV06 / F15 | `PARTIALLY_FIXED` | NaN/Inf/≤cero absoluto, capacidad diminuta y overflow de transferencia rechazan atómicamente. Restore acepta c_wall=0 y wall_frac=NaN con hash válido y estado térmico presente. Capacidad MAX finita produce stored_j=Inf y RunTrace con cierre no finito. C1-02/C1-04. No se demuestra problema térmico nominal. |
| RV07 / F15 | `VERIFIED_FIXED` | Matriz de 15 observaciones ECU no finitas (5 campos×NaN/±Inf) inhibe inyección/chispa; tres tiempos decoder inválidos dan SyncLost, tiempos/fase finitos y cero comandos. F13 de fase permanece fuera de alcance. No se demuestra protección ante toda configuración pública mutable de ECU. |
| RV08 / F15 | `VERIFIED_FIXED` | Diez negativos: callback corto/largo inicial o posterior, NaN/Inf, y/w/p no finitos y pesos de longitud distinta; todos Err tipado, cero panic. F18/F19 no cerrados; no se garantiza que un callback arbitrario que hace panic internamente sea recuperado. |
| RV09 / F15 | `VERIFIED_FIXED` | Catorce controles apply on/off con presión subnormal, NaN, temperatura Inf/−300/MAX y nominal: ninguna salida no finita aceptada. No cambian criterios estacionarios ni F03/F04. |

## Contratos originales y regresiones

| Contrato | Estado C1 |
|---|---|
| F01 | `VERIFIED_FIXED` dentro de representación homogénea L1 y dominio numérico probado |
| F02 | `PARTIALLY_FIXED`: causalidad de observación reparada; frontera de protocolo incompleto pendiente |
| F05 | `VERIFIED_FIXED` para las rutas JSON/bits/hashes ensayadas |
| F06 | `PARTIALLY_FIXED`: lectura histórica y migración pre-R02 etiquetada pasan; falta política/versionado para estado R02 caliente con contabilidad antigua; metadata no autenticado |
| F15 | `PARTIALLY_FIXED`: C1-01/C1-02/C1-04 impiden cierre transversal |
| F07 / F08 / F09 / F10 | `VERIFIED_FIXED` conservados en dominios anteriores |

Replay nativo: 36 cortes/horizontes + 18 trayectorias de tres cortes, c2=0/.01/.05, térmico on/off, sin fuel/sin spark y estados completos. Comparación de todas las muestras y estado/hash; no solo RPM final. La procedencia de la inicialización artificial cold() también permanece idéntica entre las ramas de comparación. Ledger de frontera por segmentos sigue rebasado; su historial completo no forma parte del replay dinámico declarado.

F08: 18 mezclas cerradas y cuatro controles de regulador, gradientes invertidos por puertos dirigidos. F09: ocho controles con Oil870/Coolant1060 kg/m³ y volumen analítico; no hay API de densidad arbitraria ni se simula su existencia. F10: 24 balances, seis fronteras nulas, tres refinamientos exponenciales y controles sobre cero absoluto. Dominio físico y balances siguen verdes. Estas pruebas son fixtures numéricos de verificación, nunca datos experimentales.

## Ejecución y cobertura

| Batería | Resultado independiente |
|---|---|
| cargo test --locked --workspace | 198 aprobados / 0 fallos; 59 bloques de resultado, 30 con casos |
| cargo fmt --all -- --check | exit0 |
| cargo clippy --locked --workspace --all-targets -- -D clippy::unwrap_used | exit0; otros warnings conservados |
| Oráculo S30 original | 22/32; 10 fallos esperados de R03/R04; 89 predicados, 138 filas, 42 familias |
| Oráculo adversarial anterior intacto | 19/19; 196 predicados, 198 filas, 31 familias |
| Controles C1 nuevos | 10/15; 5 fallos, 0 errores; 121 predicados ejecutados, 138 filas, 20 familias |
| Repetición adversarial anterior | cinco modos JSONL byte-idénticos |
| Reconstrucción final desde Git |198/0 C1 y163/0 S30; oráculoS30 original9/32; nueve modos nuevos byte-idénticos a la ejecución directa |

Los comandos fmt/clippy anteriores corresponden al workspace de producción; los probes tienen workspace Cargo separado. La extracción privada byte-exacta mantiene un newline adicional deliberado: git diff --check lo señala, no se normaliza porque rompería el chequeo de integridad. Los logs/diffs crudos conservan sus bytes. Son tres targets Python de contratos; casos, filas y predicados se cuentan por separado. Los máximos por paso comprimen muchas observaciones internas: no equivalen a 196008 assertions individuales. Los checks se detienen al primer fallo por caso; las matrices completas permanecen en JSONL aunque el contador de predicados no alcance todos sus elementos.

Química: **196008 pasos**, 12 corridas, cierre de combustible máximo **2.20e-18kg**, error máximo entre dQ e inventario consumido **2.40e-12J**; todos los inventarios permanecen en [0,m]. Referencia Wiebe: masa esperada 9.93262053e-7kg; error100µs 2.099e-10kg, 6.25µs 9.51e-13kg. Error no monotónico en cada subdivisión por posición del borde de ventana; no se finge un orden estricto para todas las filas. El control limita variación angular del motor con inercia declarada1e9, sin modificar parámetros nominales productivos.

Ralentí nominal8s/dt50µs: S30 1025.8177 → R02 1047.3173 → C1 **1047.30387374RPM**; Q=46146.847889J, trabajo indicado=13286.519774J, residuo energético=6.95678e-5. El cambio procede del consumo dentro del paso; la inicialización nominal no cambia. Red térmica120s: residuo 2.17888e-5/1.11946e-5/5.64914e-6 para dt100/50/25µs. Disminuir residuo no prueba convergencia general de RPM ni estabilidad asintótica. G02 continúa abierto.

## Alcance de defectos y limitaciones

C1-01/C1-02/C1-04 se reproducen también sobre 8a con las mismas condiciones: **residuales de F15**, no regresiones introducidas por C1. C1-03 es una frontera diagnóstica sin validación ya presente en 8a; no es lectura de LeakScenario ni una prueba de AUD-R01. C1-05 demuestra omisión de migración semántica de la historia química R02: una fuente con todos los campos estructurales no garantiza semántica C1. No se atribuye a C1 haber creado los1157J históricos; sí se compromete el etiquetado/continuidad al aceptar ese estado como nativo sin advertencia. C1-G01 es un riesgo documentado de integridad del metadata nuevo, no prueba de torque incorrecto ni una exigencia retroactiva de firmar todos los snapshots. Las declaraciones generales de validación antes de mutación del ADR requieren restricción o corrección, C1-D01.

K01–K03 y G01–G07 originales permanecen. No se examinan segundo host/ARM/Windows, fuzzing exhaustivo, callback malicioso, horizontes mayores120s, gradientes espaciales, reconstrucción física exacta de snapshots legacy calientes ni identificación multicause. No se declara calibración o validación experimental D16Y7. Las áreas fuera de alcance son explícitas y no transformadas en PASS.

Para las acciones y repros concretas consultar [C1_FINDINGS.md](C1_FINDINGS.md), [C1_REPRODUCTION.md](C1_REPRODUCTION.md) y [C1_GATE_DECISION.md](C1_GATE_DECISION.md). La misión termina aquí, sin nuevas funcionalidades, cambios productivos ni inicio de R03/R04/R05/S31.
