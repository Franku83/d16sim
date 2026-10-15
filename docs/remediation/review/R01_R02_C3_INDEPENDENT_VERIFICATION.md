# Verificación independiente R01–R02-C3

**Fecha: 2026-10-15. Gate A: DEVELOPMENT_INTEGRATION_BLOCKED. Gate B global: PARTIALLY_VERIFIED. No fast-forward ejecutado.**

C3 corrige los once casos de la extensión C2 sin modificar sus oráculos. La ampliación dirigida confirma dos incumplimientos P1 de contratos ya autorizados: presión derivada de nodos sin validar y pérdida posterior de pmax no señalada cuando hay procedencia exact_neutral heredada. También registra dos P2 delimitados, sin convertirlos automáticamente en P1: tolerancia nueva de esquemas conocidos incoherentes y overflow residual en resistencia de batería. No se implementó producción ni se continuó R03/R04/R05/S31.

La decisión de integración está separada de la certificación científica. No se demostró ausencia universal de defectos ni se aportó calibración o validación experimental D16Y7.

## Identidades y aislamiento

| Referencia | SHA completo |
|---|---|
| S30/master | ff09a0dda82188e6bf27a80d17c902f8972a39d2 |
| Auditoría original | 25d528e3ff5b31eaeab0c894c4227b3e7d3a574c |
| Destino/R01–R02 | 8a7e8686e4e8831fb30d1a71b815e67e7d7139ef |
| C1 | caad1cbf31d01531e0fbd81be6190f52b2d1c2d7 |
| C2 | a0040c58174a6cf6a2ac84e38605eafd2f584159 |
| Revisión previa | 40e75d54ba5b09d33e358b5777314ea2c657740a |
| C3 implementación | 8fc7a59747c5b1acfec7695f830a9b1cd8869d47 |
| **C3 evaluado** | **3aef5f6ccdc076dad6e8be627a5eb836b26ee998** |

Padres C3: 3aef5f6→8fc7a59→a0040c5. C2 desciende linealmente de C1 y8a; `git merge-base --is-ancestor` confirma el destino como ancestro. No hay commits divergentes en la cadena. `git-identities.json` registra padres completos y asuntos reales. Source C3/destino/auditoría/revisor estaban limpios al inicio; source/destino permanecen limpios y sus HEAD no cambiaron. Se ejecutó en worktree nuevo detached `d16sim-verification-c3`, con dependencias productivas exactas de ese commit. Los archivos nuevos no rastreados allí son exclusivamente fuentes de auditoría extraídas de Git; ningún archivo productivo ajeno interviene.

La rama revisora `verification/r01-r02-2026-10-10` conserva producción8a y agrega únicamente nueva evidencia sobre40e75d5. No se mezcló historia productiva con el commit de verificación. Master conserva el único spec no rastreado preexistente; no se usa como autoridad. Las versiones obligatorias se leyeron mediante Git desde C3 o40e75d5 según correspondiera. `source-index.json` identifica sus blobs.

Foundation/Evidence Policy/DoD/SPRINTS/HANDOFF permanecen byte-idénticos a S30. PROGRESS agrega entrada C3 identificada; no se reescribieron gates S00–S30. Snapshot-contract sí cambia su política de coherencia, analizada expresamente en C3-03. El cambio de un unit test C2 de rechazo a tolerancia es visible en el diff, no evidencia independiente de aceptación.

## Estado de las seis correcciones

| ID | Resultado independiente | Contraste de causa raíz y límite |
|---|---|---|
| C2-01 | **VERIFIED_FIXED dentro del dominio ensayado** |20 campos de inyector/ambas levas×6 valores×2 intervalos=240 filas; incluye cinco deadtimes, incertidumbre, advertised/at050/LSA/centerline/flank/lift y relaciones at050>adv de ambas levas. run/restore/resume, no-op/2ms, negativos NaN/±Inf/0/−1 según unidad y positivos admisibles. Los dos activos negative_injector/nan_cam ahora rechazan en el oráculo C2 intacto. Errores preservan WorkState/ledgers. |
| C2-02 | **PARTIALLY_FIXED** | Temperatura y presión inválidas del cilindro y densidad inválida de nodos se rechazan. Falta presión de nodos en restore/no-op: C3-01 P1, residual dentro del contrato. |
| C2-03 | **VERIFIED_FIXED dentro del dominio ensayado** |24 pares capacidad/temperatura, separando parámetros intrínsecos y energía del estado real. Capacidades1e307/MAX a0°C aceptadas; a20°C rechazadas atómicamente. run_engine sigue validando su inicialización20°C. Negativos Celsius térmicos legítimos no se rechazan por signo. |
| C2-04 | **PARTIALLY_FIXED / desviación P2** | Desconocidos se rechazan en v1/v2 y snapshots genuinos sin model siguen leyendo. Pares conocidos incoherentes se aceptan por política nueva unilateral; no equivale a cumplir el requisito anterior de coherencia. C3-03. |
| C2-05 | **PARTIALLY_FIXED** | Sin procedencia previa: pmax ausente/c2≠0 genera approximate_hot; controlc2=0 conserva muestras dinámicas. Con exact_neutral heredado de una migración genuina, una nueva pérdida de pmax no queda acotada: C3-02 P1. |
| C2-06 | **VERIFIED_FIXED para temperatura** |11 temperaturas, incluso−20,0,20,100°C y el borde−273.149999°C; bajo/igual cero absoluto y NaN/±Inf rechazan coherentemente. R(−20)=.0164Ω>R(20)=.01Ω. Residual numérico distinto en R0 extremo: C3-04 P2, ya existente en C2 a10°C. |

Son etiquetas de la revisión, no nuevas categorías científicas del Core. Ver [C3_FINDINGS.md](C3_FINDINGS.md) para archivo/símbolo/líneas, entradas, esperado/observado, causa, alcance, acciones y pruebas por ID.

## Ejecución y conteos reales

| Batería | Casos / resultado | Predicados / observaciones |
|---|---|---|
| Rust C3 `cargo test --locked --workspace` | **209 aprobados,0 fallos**;59 bloques de resultado,30 con casos | Assertions internas no instrumentadas |
| Rust S30 reejecutado | **163 aprobados,0 fallos**;59 bloques,29 con casos | No confundir targets con casos |
| fmt / clippy exigidos | **exit0 / exit0** | Clippy conserva warnings permitidos por el comando |
| Oráculo original S30 sobre C3 | **22/32** |89 predicados,138 filas;1 target Python |
| Oráculo adversarial R01–R02 | **19/19** |196 predicados,198 filas/31 familias |
| Oráculo controles C1 | **15/15** |150 predicados,138 filas/20 familias |
| Extensión C2 intacta sobre C3 | **11/11** |1442 predicados,752 filas/15 familias |
| **Nuevos controles dirigidos C3** | **7/11;4 fallos,0 errores** |323 predicados ejecutados,333 filas/11 familias;1 target Python |

Los cuatro fallos dirigidos son los dos P1 y los dos P2 descritos; no se suman automáticamente como cuatro bloqueos P1. Los generadores completan sus observaciones aun si un caso Python falla al primer predicado. No son333 tests ni323 mecanismos físicos independientes.

Los diez rojos originales son exactamente17/18(F03/F04,R04),19/20(F11/F12,R03),21/22(F13,R03),23(F14,R03),28(F16,R03),31/32(F18/F19,R03). No bloquean por sí solos esta integración. La referencia S30 sigue9/32 con23 fallos. Los cuatro CLI mecánicos pasan; cinco modos adversariales repetidos producen bytes idénticos. No hubo fallo de build de los probes científicos.

Se conservó una ejecución incompleta de adaptación del runner: el primer `run_history.py` copiado apuntaba todavía al directorio c2_review y no encontró sus lockfiles. Se corrigió únicamente esa ruta en la copia nueva c3_review; la reproducción posterior completa pasó. No fue fallo del Core ni se cambió ningún oráculo. Los logs iniciales se conservan, no sustituyen las ejecuciones completas.

## Cobertura y magnitudes independientes

- **Termodinámica:**20 filas nodos (dos nodos×nominal/overflowP/overflowρ/underflowP/extremo finito×no-op/avance) y3 negativos nuevos de cilindro. En el overflowP de nodo, m=1e−5,T=1e100,V=1e−214: presión matemática≈2.87e311Pa excede f64MAX, pero energía≈7.175e97J y densidad1e209 son representables. No es solo repetir una ecuación como oráculo: la cota de representabilidad independiente demuestra por qué debe rechazarse. ControlV=1e−200 da presión finita2.87e297Pa; restore/no-op la admiten. No se exige que un paso Euler grande resuelva extremos sin error dinámico controlado.
- **Contexto térmico:**4 capacidades(1e−308,1e300,1e307,MAX)×6 temperaturas(−200,−1,0,1,20,1e100°C). Stored_j es energía referida a0°C, por lo que un valor negativo finito no es energía absoluta inválida. Los errores preservan el estado; aceptación/rechazo corresponde a su energía actual.
- **Batería:**11 temperaturas y6 estreses adicionales(T−20/10/20×R0MAX/4 yMAX). El residual a10°C se reproduce igual contra C2; no se atribuye a C3.
- **Semántica:**10 combinaciones dirigidas v1/v2×ausente/coherente/incoherente/desconocido, además de los25 snapshots genuinos de cinco productores y sus100 negativos/controles del oráculo C2. No se alteraron los productores para maquillar su historia. La lectura/hash histórica se conserva; metadata permanece no autenticada.
- **Presión/replay:**12 contrafactuales nuevos(3 fricciones×4 cortes), pérdida posterior a migración genuina y4 controles de memoria adicional con parseo negativo. En c2=0 las4 trazas y otros campos dinámicos coinciden, pero pmax final difiere en3; no es identidad universal de snapshot. Los cuatro campos burn_latched/burn_accounted/prev_local/fuel se preservan y su ausencia en JSON se rechaza. Ninguno puede omitirse por el control de presión0.
- **Antes/después:** los mismos sources `c3_directed.rs` se compilan contra crates C3 y C2 archivados desde Git. Esto separa la omisión residual de nodos/batería/procedencia heredada de la nueva relajación de coherencia de esquemas.

Ralentí nominal8s/dt50µs reproduce **1047.3038737395177RPM**, Q46146.847889316596J, W_ind13286.519773754073J y residuo6.956782748242368e−5. Base_inputs byte-idéntico al original; no hay reajuste nominal. Térmico120s reproduce residuos2.178879805e−5/1.119464901e−5/5.649137702e−6 para100/50/25µs; no demuestra estabilidad general ni convergencia larga de RPM.

R02 mid-burn genuino conserva hash151f2e8b54d7d154bedce8ae3473f3ba4d38831a468d23c40ae22a46b0f150b8 y `approximate_hot`; discrepancia histórica1157.151388065635J→1157.1513880656353J tras continuación, sin resta artificial. Los25 archivos genuinos de2541266/ff09/8a/caad/a004 siguen leyendo/verificando/recapturando; los reportes JSON registran procedencia completa.

## Gate B — Certificación de contratos científicos

| Contrato | Veredicto Gate B | Dominio realmente demostrado / restricción |
|---|---|---|
| F01 | **VERIFIED_WITHIN_TESTED_DOMAIN** | Especie combustible homogénea L1;12 corridas/196008 pasos,3 resoluciones, escasez/reflujo/sin combustible/sin futuros encendidos;36 continuaciones durante combustión;15 referencias de antiderivada Wiebe. No validación química experimental ni quenching de llama existente. |
| F02 | **VERIFIED_WITHIN_TESTED_DOMAIN** | Flujos físicos→firma completa normalizada;70% escape, invariancia de etiquetas y limitación de canales/observaciones incompletas. AUD-R01 no resuelto; dos interfaces no prueban independencia ni causa única. |
| F05 | **VERIFIED_WITHIN_TESTED_DOMAIN** | JSON lossless, bits/hashes, cero firmado y rutas históricas probadas para motor/desgaste; no autenticación de metadata. |
| F06 | **PARTIALLY_VERIFIED** | Lectura y hashes genuinos, versión desconocida y R02 caliente bien acotados; nueva pérdida con procedencia heredada no señalada y coherencia de esquemas pendiente. |
| F15 | **PARTIALLY_VERIFIED** | Validaciones originales/levas/inyector/cilindro/térmico/batería/ECU/calibrador/dyno y errores atómicos probados; C3-01 y C3-04 impiden garantía completa dentro de los extremos declarados. |
| F07 | **VERIFIED_WITHIN_TESTED_DOMAIN** | Snapshot nativo completo:36 cortes/horizontes+18 trayectorias multicorte, estados/hashes/toda traza, térmico on/off,c2=0/.01/.05. No extender a historia incompleta; C3-02 pertenece a esa frontera F06/F07. |
| F08/F09/F10 | **VERIFIED_WITHIN_TESTED_DOMAIN** |22 controles de especie fresca/fronteras,8 líquidos para Oil/Coolant,33 térmicos locales/globales/analíticos; dos densidades fijas reales de la API, no densidad arbitraria inventada. |
| RV07/RV08/RV09 | **VERIFIED_WITHIN_TESTED_DOMAIN** |15 ECU+3 decoder no finitos,10 callbacks/datos de calibración sin panic dimensional,14 correcciones dyno finitas. No cierra F13/F18/F19/F03/F04. |

**Resultado global: PARTIALLY_VERIFIED.** La conservación nominal que sobrevive no compensa una frontera de continuación mal acotada; tampoco estos defectos invalidan por sí solos toda ecuación verificada.

## Límites y decisión

Mantener abiertos AUD-R01,K01–K03,G01–G07 y F03/F04/F11–F14/F16/F18/F19. Single host Linux x86_64/toolchain1.99; no fuzzing exhaustivo, >120s, reconstrucción exacta de legacy caliente, parámetros/materiales OEM, ni experimentos independientes. Atomicidad se compara con Debug completo(incluidos ledgers), snapshots y hashes según contrato; no prueba payloads arbitrarios de NaN ni representación byte a byte de memoria Rust.

Gate A falla por C3-01/C3-02, que corresponden a contratos autorizados y quedan P1 conocidos sin resolver. C3-03/C3-04 son P2 acotados; no se ascienden para fabricar un bloqueo. Si se retienen en una futura propuesta de baseline, deben corregirse o contar con aceptación expresa previa, ausente aquí. No basta etiquetarlos «documentados» en el reporte del implementador. El estado no es solamente AWAITING_OWNER_RISK_ACCEPTANCE: siguen dos P1.

[Hallazgos](C3_FINDINGS.md), [reproducción](C3_REPRODUCTION.md) y [decisión](C3_GATE_DECISION.md) contienen acciones y evidencia. Se detiene la misión sin integración, sin fixes y sin autorizar el siguiente batch.
