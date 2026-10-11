# Verificación independiente R01–R02-C2

**Veredicto científico/técnico: C2_REMEDIATION_REQUIRED. No fast-forward.**

C2 corrige los cinco fixtures bloqueantes de C1, pero no satisface todavía el gate amplio autorizado. La extensión independiente reproduce cinco incumplimientos: configuración activa de leva/inyector inválida aceptada, temperatura derivada no finita aceptada en restore/no-op, rechazo térmico de energía actual finita, identidad semántica desconocida admitida en v1 y pérdida de pmax clasificada sin procedencia aproximada. Los dos últimos mecanismos de continuación y el dominio térmico requieren revisión antes de integrar. Ningún resultado verde de las suites anteriores sustituye estas fronteras nuevas.

Fecha2026-10-10; Linux x86_64, Rust/Cargo1.99.0. Evidencia de software/ecuaciones L1, **sin calibración ni validación experimental del D16Y7**. Los datos generados son fixtures numéricos, no mediciones externas. No se corrigió producción ni se inició R03/R04/R05/S31.

## Identidad y fuentes

| Referencia | SHA completo verificado |
|---|---|
| S30/master | ff09a0dda82188e6bf27a80d17c902f8972a39d2 |
| Auditoría original | 25d528e3ff5b31eaeab0c894c4227b3e7d3a574c |
| R01–R02/destino | 8a7e8686e4e8831fb30d1a71b815e67e7d7139ef |
| C1 | caad1cbf31d01531e0fbd81be6190f52b2d1c2d7 |
| Revisión previa | a0088ff0301c07aa89f4ca4f5fab32b53eba9960 |
| C2 físico/semántico | 698b2e30252f7722a233e0c8b7484d1b777b191d |
| C2 diagnóstico | 9021d7dbac4f2fb4b151b8c2e3023861698e6561 |
| **C2 objetivo** | **a0040c58174a6cf6a2ac84e38605eafd2f584159** |

Padres exactos: a0040c5→9021d7d→698b2e3→caad1cb→edb36cc72de89a881e7ef8d5944a5f56112a35f4→23e48eb7c19dc2e649fd31c30debcf615cce03cf→8a7e868. Todos son pasos lineales de un padre. Destino8a es ancestro y sigue intacto; ninguna divergencia. La revisión trabaja en su propia rama basada en a0088ff y ejecuta contra un worktree detached C2, sin trasladar commits productivos al revisor.

Se consultaron fuentes Git reales: specv1.0; reportes R01–R02/C1/C2; matrizC2; ADR038/039/040; snapshot-contract; Foundation/Evidence Policy/DoD; los cuatro informes C1 desde a0088ff y hallazgos originales desde25d. [source-index.json](c2-results/source-index.json) registra blobs consultados. [implementation-c2.diff](c2-results/implementation-c2.diff) y [implementation-accumulated.diff](c2-results/implementation-accumulated.diff) permiten inspeccionar diferencias reales. Foundation, Evidence Policy, DoD, SPRINTS y HANDOFF no cambiaron; PROGRESS agrega entradas identificadas, sin reescribir gates históricos. Se conserva la historia de los informes previos.

## Cinco bloqueos C1 y contratos

| Bloqueo | Resultado independiente | Evidencia/límite |
|---|---|---|
| C1-01 / C2-01 Engine | **PARTIALLY_FIXED** | Los negativos previos y 37 campos listados no finitos pasan; faltan parámetros activos de leva/inyector y derivadas físicas. C2-01/02 del nuevo informe. |
| C1-02 / C2-02 Thermal/Restore | **VERIFIED_FIXED** en dominio de rechazo original | Capacidades0/NaN/Inf, wall_frac inválido, estado inválido y overflow actual se rechazan; 98 configuraciones ampliadas contrastadas también en run/restore/resume. El rechazo excesivo de estados válidos queda separado en C2-03. |
| C1-03 / C2-03 diagnóstico | **VERIFIED_FIXED** en dominio completo normalizado | 30 observaciones, 5 bordes de suma, 8 ausencias; datos inválidos/parciales no discriminan. No demuestra independencia instrumental o identificabilidad multicause. AUD-R01 abierto. |
| C1-04 / C2-04 energía térmica | **PARTIALLY_FIXED / REGRESSION_FOUND** | El overflow original se rechaza atómicamente. C2 rechaza también capacidades grandes con energía actual0 finita por evaluar cold20 ajeno al restore/resume; confirmado contra C1. |
| C1-05 / C2-05 snapshots | **PARTIALLY_FIXED / REGRESSION_FOUND** | R02 mid-burn se etiqueta correctamente approximate_hot. v1 model inválido se ignora y modelo C2 con pmax ausente evita etiqueta aproximada. |

| Contrato original | Estado y alcance |
|---|---|
| F01 | VERIFIED_FIXED conservado para inputs válidos, especie homogénea L1, química por paso/refinamiento/reflujo y ventanas de replay ensayadas |
| F02 | VERIFIED_FIXED para derivación observacional de flujos y firma completa válida; no una garantía de identificación multicause |
| F05 | VERIFIED_FIXED: JSON lossless, bits/hashes/cero firmado y hashes históricos probados |
| F06 | PARTIALLY_FIXED: lectura/hash/procedencia genuinos pasan, pero contratos semánticos negativos C2-04/05 pendientes |
| F15 | PARTIALLY_FIXED: rollback de errores conservado; configuración/derivadas/domino térmico C2-01/02/03 y temperatura de bateríaC2-06 pendientes |
| F07 | Preservado para snapshots nativos completos: 54 comparaciones de trayectoria/estado/hash/traza; omisión de pmax no autorizada como exacta, C2-05 |
| F08 | Preservado: 18+4 controles de mezcla, fuentes/fronteras e inversión de gradiente |
| F09 | Preservado: 8 referencias analíticas para dos fluidos/direcciones/pasos; densidades fijas de API, no barrido arbitrario inventado |
| F10 | Preservado: 24 balances locales/globales, 6 casos sin transferencia, 3 refinamientos analíticos, controles térmicos válidos adicionales |
| RV07 / RV08 / RV09 | Preservados: 15 ECU+3 decoder negativos; 10 fitter sin panic dimensional; 14 dyno sin Ok no finito; no se cierran F13/F18/F19/F03/F04 |

## Conteos reales y reproducción

| Batería | Targets/bloques | Casos y resultado | Predicados / observaciones |
|---|---|---|---|
| C2 Rust workspace | 59 bloques;30 con casos | **204 pasan,0 fallan** | Assertions internas no instrumentadas; no inferir conteo |
| S30 referencia reejecutada | 59 bloques;29 con casos | **163 pasan,0 fallan** | Referencia original |
| fmt / clippy requeridos | comandos completos | **exit0 / exit0** | Clippy tiene warnings; flag exigido solo unwrap_used |
| Oráculo S30 sobre C2 | 1 Python target | **22/32**,10 fallos esperados fuera de alcance | 89 predicados ejecutados;138 filas |
| Oráculo adversarial R01–R02 | 1 Python target | **19/19** | 196 predicados;198 filas/31 familias |
| Oráculo controles C1 intacto | 1 Python target | **15/15** | 150 predicados;138 filas/20 familias |
| **Extensión independiente C2** | 1 Python target;5 modos Rust | **6/11**,5 fallos confirmados,0 errores | **1274 predicados ejecutados;752 filas/15 familias** |

Las filas no son casos ni assertions. Cada caso Python puede combinar varios mecanismos y deja de evaluar sus predicados al primer fallo; los generadores sí completan todas las filas. Los tres oráculos previos no se alteraron para acomodar C2. Los diez fallos originales son exactamente17/18(F03/F04,R04),19/20(F11/F12,R03),21/22(F13,R03),23(F14,R03),28(F16,R03),31/32(F18/F19,R03). No hay error de build ni fallo adicional de ese oráculo. Su exit1 esperado no es por sí solo un bloqueo C2.

La referencia S30 conserva9/32 aprobados y23 fallos al reejecutarla. Se ejecutaron los cuatro comandos CLI mecánicos y cinco modos adversariales repetidos byte-idénticos. Logs de [run-01](c2-results/run-01/) y reproducción final ampliada desde Git en [from-git-expanded](c2-results/from-git-expanded/). Las ejecuciones iniciales y ampliaciones quedan separadas, sin sobrescribir evidencia histórica.

## Cobertura nueva frente a matriz C2

- Engine: **45 campos ×5 valores ×2 intervalos=450 filas** (NaN/±Inf/0/−1), 4 relaciones (ηv/ηcomb/SOC>1 y free_current=stall_current), 14 estados/no-op/avance y3 controles activos. Para 37 campos previamente listados, los222 negativos no finitos rechazan run/restore/resume sin mutar; ocho campos de leva/inyector amplían una omisión real. Los valores0/negativos se juzgan por unidad, no como inválidos universalmente. Ejemplo: avance y torque negativos admitidos, batería−20°C rechazada indebidamente.
- Thermal: **14 campos ×7 valores=98** configuraciones, examinadas en validated/stored_j_checked/step y también run_engine/restore/resume; 2 extremos positivos válidos,9 temperaturas,2 controles de energía real0 con capacidad grande. No se inventaron máximos OEM. Ledger ambiental y referencias Newton analíticas se conservan desde los oráculos anteriores.
- Diagnóstico: **30 entradas por campo,5 bordes Σ,8 máscaras** de canales. Σ=1±1e−6 es una tolerancia absoluta de normalización publicada, no una incertidumbre instrumental; se comprueba aceptación a±0.9e−6 y rechazo a±1.1e−6. No se renormalizan firmas incompletas en el probe. Helper observacional byte-exacto de CLI C2 y contrafactuales de etiquetas/repartos anteriores;70% de escape real persiste. El clasificador requiere compresión y leak-down con firma, pero eso no prueba dos fuentes físicamente independientes.
- Snapshots: **5 productores reales ×5 cortes=25** archivos;100 negativos/controles de metadata e historia; control físico adicional de pérdida de pmax, hashes válidos y continuidad divergente. Cada productor compila contra código archivado de su Git SHA y tipos originales; no se genera JSON manual como oráculo. Fixture inicial físico declarado; historia caliente producida por solver histórico. El productor C1 elimina solo la procedencia auxiliar del montaje frío para representar una captura nativa completa; no modifica m/u/combustible/presión/trayectoria. Archivos auténticos y transformaciones negativas se separan.

[SNAPSHOT_INVENTORY.md](c2-results/SNAPSHOT_INVENTORY.md) y JSON asociado registran cada hash, versión, model, presión/fuel/estado de combustión, procedencia, campos ausentes, lectura, recaptura y continuación. Todos los25 snapshots genuinos leen y verifican sus hashes. Para v1/S30/R02/C1: frío exact_neutral, pre/mid/post/hot approximate_hot; para C2: sin migración, replay completo exacto. Migración recapturada conserva provenance; no se eleva a exacta por añadir model actual.

R02 mid-burn auténtico: tiempo0.35959999999996856, hash151f2e8b54d7d154bedce8ae3473f3ba4d38831a468d23c40ae22a46b0f150b8; exceso histórico1157.151388065635J. C2 lo marca approximate_hot, missing_fields combustion_accounting; después de continuar mantiene la etiqueta y el exceso histórico1157.1513880656353J. No se resta ni se fuerza cierre de ese pasado.

## Magnitudes nominales y regresiones

Ralentí8s/50µs: **1047.3038737395177RPM**, Q46146.847889316596J, W_ind13286.519773754073J, residuo6.956782748242368e−5. Idéntico a C1; base_inputs byte-idéntico a S30 y ausencia de reajustes nominales contrastadas por Git. Red térmica120s: residuo2.178879805e−5/1.119464901e−5/5.649137702e−6 para100/50/25µs. Disminuir residuo no prueba convergencia general de RPM; G02 permanece.

Química por paso:12 corridas/196008 pasos,3 resoluciones, sin combustible/sin futuros encendidos/escasez/reflujo según fixtures;36 continuaciones durante combustión. Referencia antiderivada Wiebe independiente:15 controles (5 pasos×3 eficiencias). Estos tests verifican inventario y cuadratura dentro de ese modelo, no una combustión real calibrada. No se apagó una llama existente al cortar futuros eventos.

Regresiones demostradas: clasificación de historia incompleta C2-05 frente C1 y rechazo de energía térmica válida C2-03. No se demuestra nueva regresión P0 de conservación nominal. C2-01/02 son omisiones residuales F15; C2-04 es omisión de la nueva frontera semántica. C2-06 era previo a C2.

## Restricciones de evidencia y confianza

Model/provenance están fuera de state_hash por contrato. API verified() demuestra integridad del estado, no autenticidad de la etiqueta. Marcadores conocidos manipulados pueden conservar hash válido: el consumidor necesita fuente externa fiable. Esa limitación declarada no se convierte aquí en requisito de firma fuera de alcance. El defecto C2-04 es aceptación de versión explícitamente desconocida; C2-05 es historia física faltante aun con marcador conocido. Ambos son contratos distintos de autenticar metadata.

K01/K02/K03 y G01–G07 permanecen abiertos: integración runtime parcial, modelos L1 ilustrativos, daño geométrico sin vida/materiales; evidencia experimental, estabilidad general, otros hosts/revisión experta, cadenas físicas completas, identificabilidad, electrónica genérica y manifiestos físicos. AUD-R01 sigue abierto. No fuzzing exhaustivo, segundo host, callback malicioso, >120s, gradientes espaciales ni continuidad legacy caliente exacta. Constructor ThermalState::cold es un constructor simple; la validez se exige en los consumidores checked/step/Engine. `stored_j_checked` comprueba finitud derivada, no sustituye validated/Kelvin. Las interfaces sin resultado como thermostat_factor/enrich_factor requieren estado/parámetros válidos; no se certifica protección universal de toda función de utilidad.

No se actualizan gates históricos ni se declara simulador predictivo. El gate actual de remediación queda retenido por los contratos concretos F06/F15 comprometidos, con acciones reproducibles en [C2_FINDINGS.md](C2_FINDINGS.md).
