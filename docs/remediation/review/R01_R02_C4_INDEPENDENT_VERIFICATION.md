# Verificación independiente R01–R02-C4

**Fecha2026-10-15. Gate A: READY_FOR_DEVELOPMENT_INTEGRATION. Gate B: VERIFIED_WITHIN_TESTED_DOMAIN por contrato, con límites explícitos.**

Los cuatro hallazgos C3-01…C3-04 se verifican corregidos dentro de sus contratos. Se acepta independientemente la opción A: conserva el source y oráculos históricos, modifica solo model=None en una copia y representa el mismo estado/hash que el v1 genuino. **El harness literal conserva21/32 y falla caso14; el adaptado obtiene22/32.** No son resultados intercambiables.

No se detectaron nuevos defectos materiales ni residuales P2/P3 dentro del alcance que requieran aceptación adicional. No se afirma ausencia universal de defectos, calibración, capacidad predictiva u OEM-exactitud. Gate A autoriza solo el fast-forward al SHA examinado si las precondiciones Git siguen cumpliéndose; la constancia de integración posterior está en C4_GATE_DECISION.md e integration-receipt.json.

## Identidades, fuentes e independencia

| Referencia | SHA completo |
|---|---|
| S30/master | ff09a0dda82188e6bf27a80d17c902f8972a39d2 |
| Auditoría original | 25d528e3ff5b31eaeab0c894c4227b3e7d3a574c |
| Destino inicial/R01–R02 | 8a7e8686e4e8831fb30d1a71b815e67e7d7139ef |
| C1 | caad1cbf31d01531e0fbd81be6190f52b2d1c2d7 |
| C2 | a0040c58174a6cf6a2ac84e38605eafd2f584159 |
| C3 | 3aef5f6ccdc076dad6e8be627a5eb836b26ee998 |
| Revisión C3 | 36919b0455e8eaf741a72ff4a74ae85f40e116ba |
| Preflight C4 | ed56f68a95eca81ed20ee2c0b943cf8bba884a5c |
| **C4 examinado** | **8c51b5d9c705745b1a558b00e8bc41d73aad0d7f** |

Cadena lineal comprobada destino→C1→C2→C3→ed56f68→8c51b5d. Padres completos en git-identities.json; sin divergencia. Implementador C4 y destino limpios al inicio, HEAD candidato estable; se vuelve a comprobar inmediatamente antes de integrar. La revisión se ejecutó en worktree nuevo detached C4, sin modificar el checkout del implementador. Producción de la rama revisora permanece8a: sus nuevos archivos de auditoría no forman parte del fast-forward productivo.

Documentos leídos desde sus blobs Git: decisión/autoridad/reporte/reproducción C4, matriz/reporte C3, cuatro informes independientes C3 desde36919b0, spec R01–R05, ADR038–042, snapshot-contract y normativos. source-index.json registra referencias reales. Foundation/Evidence Policy/DoD/SPRINTS/HANDOFF permanecen idénticos a S30; PROGRESS agrega entradas identificables y no reescribe gates históricos. La política C3 de tolerancia se sustituye explícitamente por coherencia estricta autorizada en C4, sin alterar el ADR histórico.

## Resultados observados

| Batería | Resultado | Conteos distintos |
|---|---|---|
| Rust C4 workspace | **215/0** |59 bloques,30 con casos; no se contaron assertions Rust |
| S30 Rust reejecutado | **163/0** |59 bloques,29 con casos |
| fmt / clippy exigidos | **exit0 / exit0** |Warnings permitidos permanecen |
| **S30 literal** | **21/32**,11 fallos,0 errores |89 predicados,138 filas |
| **S30 adaptado A** | **22/32**,10 fallos,0 errores |89 predicados,138 filas |
| Adversarial R01–R02 intacto | **19/19** |196 predicados,198 filas/31 familias |
| Controles C1 intactos | **15/15** |150 predicados,138 filas/20 familias |
| Controles C2 intactos | **11/11** |1442 predicados,752 filas/15 familias |
| Controles C3 intactos | **11/11** |360 predicados,333 filas/11 familias |
| **Ampliación independiente C4** | **8/8**,0 fallos/errores |285 predicados,183 filas/8 familias |

Cada checker es un target Python; los casos pueden evaluar varias observaciones y pararse al primer predicado fallido. Una fila no es un test ni un mecanismo independiente. Se ejecutaron también cuatro CLI mecánicos y repeticiones de cinco modos adversariales con identidad de bytes.

El caso14 literal falla por model2/payload1, **no** por hash v1 inválido ni rechazo del productor histórico genuino. Los otros diez fallos son exactamente17/18(F03/F04),19/20(F11/F12),21/22(F13),23(F14),28(F16),31/32(F18/F19), asignados a R03/R04 y fuera de C4. La referencia S30 original sigue9/32 con23 fallos. No se omiten ni se recatalogan esos rojos como aprobados.

## Cierre individual y nuevos contrafactuales

| Hallazgo | Veredicto | Evidencia y restricción |
|---|---|---|
| C3-01 presión de nodos | VERIFIED_FIXED |C3 intacto+24 filas nuevas de estados/nodos/endpoints y12 factory runs. Overflow/underflow/ρ inválidos rechazan hash correcto, restore/no-op/avance sin mutar ledgers. Extremos finitos se admiten sin límites OEM. |
| C3-02 composición | VERIFIED_FIXED |Caso genuino C3 ahora approximate_hot actual con prior completo. Ocho transiciones nuevas,2→9 registros y320 rereads/recapturas/round-trips; orden, hash/esquema/transformación y origen preservados sin duplicación. |
| C3-03 identidad/OpciónA | VERIFIED_FIXED; adaptador A legítimo |Igualdad estricta de esquema/modelo; diez pares nuevos más C3 calientes. Cambio exclusivo metadata en copia; mismos estado/hash/expectativas; literal14 sigue rojo y genuino/adaptado aceptados. |
| C3-04 resistencia | VERIFIED_FIXED |126 combinaciones nuevas temperatura/SOC/R, incluidosR=0, subnormal, producto y suma extremos. Referencia decimal independiente; error explícito coherente en validated/terminal/run/restore/no-op y atomicidad. |

No se implementaron correcciones por el revisor. La primera corrida de su checker C4 tuvo un falso negativo por precisión Decimal100 insuficiente para la identidad f64MAX×1; se documentó/corrigió la referencia a2000 dígitos y se reprodujo desde Git en carpeta nueva. No se alteró ningún criterio o umbral del contrato ni checker histórico. Ambas corridas se conservan; evidencia en C4_FINDINGS.md y new-checker-reference-correction.json.

## Evaluación de opción A

Source literalSHA256 b28adcc0b93950aadf73c950e47b4679f6b90f746dba688ad4cad09c37607982. DerivadoSHA256 c194a4625139f4bb53ef74a8a5cb1b60bb3212bfb1af586d5615eb0603de4e5b. Única línea `v1.model = None;`; el literal y los cinco checkers(incluido C3) permanecen byte-idénticos a36919b0. Las inicializaciones neutrales R01 ya auditadas no cambian.

El estado/hash/provenance del fixture derivado coinciden con v1 genuino2541266, hashdeb04a1b413455e20c057ec5cca14c28d691ab52630cef28d172c9224b0dea93. Las dos ejecuciones completas solo difieren en v1_hash_migration.accepted. El adaptador preserva la intención original de compatibilidad histórica; no valida metadata contradictoria ni modifica física o tolerancias. Se acepta exclusivamente esta adaptación autorizada, sin permiso general para cambiar fixtures futuros.

## Gate B — Evaluación por contrato

| Contrato | Evaluación | Garantía y dominio |
|---|---|---|
| F01 | VERIFIED_WITHIN_TESTED_DOMAIN |Combustible homogéneoL1; inventario, consumo/calor por paso,3 refinamientos, escasez/reflujo/sin combustible/sin futuros encendidos.12 corridas/196008 pasos,36 continuaciones en combustión,15 controles de antiderivada Wiebe. No quenching ni validación química experimental. |
| F02 | VERIFIED_WITHIN_TESTED_DOMAIN |Medición derivada de flujos,70% escape, etiquetas invariantes, firma completa normalizada y ausencia/invalidez honestamente limitada. AUD-R01 sigue abierto; no inferencia multicause demostrada. |
| F05 | VERIFIED_WITHIN_TESTED_DOMAIN |JSON lossless/bits/cero firmado/hashes en rutas probadas; metadata prior opcional con default/omisión y origen preservado. No autenticación de metadata. |
| F06 | VERIFIED_WITHIN_TESTED_DOMAIN |Productores genuinos v1/S30/R02/C1/C2(25 snapshots), hash por esquema, coherencia estricta, composición/idempotencia y clasificación aproximada sin reconstrucción arbitraria. Literal/adaptado separados. |
| F15 | VERIFIED_WITHIN_TESTED_DOMAIN |Matrices públicas originales+levas/inyector/derivadas/térmico y resistencias; errores controlados/atomicidad en escenarios probados. No certifica todas las combinaciones posibles. |
| F07 | VERIFIED_WITHIN_TESTED_DOMAIN |Snapshots nativos completos:36 cortes/horizontes+18 trayectorias multicorte, térmico on/off,c2=0/.01/.05, estados/hashes/trazas. Historia incompleta no se declara equivalente. |
| F08/F09/F10 | VERIFIED_WITHIN_TESTED_DOMAIN |22 mezclas/fronteras,8 líquidos Oil/Coolant,33 balances térmicos/analíticos y ledger ambiental. Densidades fijas de API, no barrido arbitrario inventado. |
| RV07/RV08/RV09 | VERIFIED_WITHIN_TESTED_DOMAIN |15 ECU+3 decoder no finitos,10 controles dimensionales del fitter,14 correcciones dyno; no cierra F13/F18/F19/F03/F04. |

Este Gate B está calificado por cada dominio; aprobar Gate A no lo produjo automáticamente. La falta de evidencia experimental se mantiene explícita y no se convierte en calibración por completar tests.

## Magnitudes, límites y riesgos

Ralentí8s/dt50µs:1047.3038737395177RPM; Q46146.847889316596J, W_ind13286.519773754073J, residuo6.956782748242368e−5. Base_inputs idéntico y sin reajustes. Térmico120s mantiene residuos2.178879805e−5/1.119464901e−5/5.649137702e−6 para100/50/25µs; no demuestra estabilidad/convergencia general de RPM.

R02 mid-burn conserva su hash151f2e8b54d7d154bedce8ae3473f3ba4d38831a468d23c40ae22a46b0f150b8, approximate_hot y discrepancia1157.151388065635→1157.1513880656353J. No se resta energía histórica para cuadrar.

Mantener abiertos K01–K03,G01–G07,AUD-R01 y los diez casos R03/R04. Host único Linux x86_64/Rust1.99; sin fuzzing exhaustivo, nuevos datos externos, profundidad arbitraria de metadata ni reconstrucción exacta de legacy caliente. prior/model/provenance no están autenticados por state_hash; writers antiguos pueden descartar prior y literales Rust externos requieren nuevo miembro. Con c2=0, samples iguales no implican identidad universal de pmax/hash ni autorizan perder otras memorias. Estos límites conocidos no son nuevos defectos automáticos ni riesgos P2/P3 pendientes de aceptación.

La aprobación se limita al desarrollo R01–R02 acumulado y al SHA candidato. No inicia ni autoriza R03/R04/R05/S31 ni merge a master. Evidencia y prueba reproducible en [C4_REPRODUCTION.md](C4_REPRODUCTION.md); decisiones y constancia posterior en [C4_GATE_DECISION.md](C4_GATE_DECISION.md).
