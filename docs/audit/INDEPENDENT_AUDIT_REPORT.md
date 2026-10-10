# Auditoría científica, técnica y documental independiente — D16SIM

**Veredicto:** revisión completada con defectos materiales, incluidos dos P0 en cadenas científicas centrales. La base implementa mecanismos L1 útiles para investigación y demostraciones numéricas acotadas. **No está suficientemente demostrada una verificación numérica general del Engine Core integrado, ni calibración/validación del D16Y7, ni capacidad predictiva.** Las aprobaciones históricas se conservan; se recomienda reevaluar requisitos concretos con la evidencia de esta auditoría.

Base auditada: `ff09a0dda82188e6bf27a80d17c902f8972a39d2` (S30), 2026-10-10. Rama aislada `audit/independent-2026-10-10`, worktree `/home/franku/projects/d16sim-audit`. **No se modificaron modelos, código de producción, parámetros nominales, Foundation, especificaciones ni gates. No se corrigieron bugs, no se fusionó a master y no se inició S31.**

## Entregables y criterio de evidencia

El diagnóstico completo está en [FINDINGS](FINDINGS.md) y su [catálogo estructurado](findings.json). [REPRODUCTION](REPRODUCTION.md) contiene comandos/fixtures/logs y un script ejecutable aislado. [DOCUMENTATION_CONSISTENCY](DOCUMENTATION_CONSISTENCY.md) inventaría 26 entradas de deriva con archivo/líneas, contraste, severidad y corrección propuesta. [CAPABILITY_RISK_ASSESSMENT](CAPABILITY_RISK_ASSESSMENT.md) revisa las 18 capacidades S30 y gates afectados. [S25_EVIDENCE_ACQUISITION_PLAN](S25_EVIDENCE_ACQUISITION_PLAN.md) desarrolla los ocho vacíos, fuentes primarias, incertidumbres, dependencias y suficiencia mínima.

Se adoptó postura adversarial: buscar contrafactuales, invariancias y presupuestos que falsen afirmaciones, no justificar el roadmap. Cada defecto confirmado tiene reproducción numérica o flujo de datos/dimensiones inequívoco con fuentes y líneas. Riesgos e integración ausente están separados. Los fixtures numéricos son SYNTHETIC_SELFTEST; **no son datos experimentales**. Los términos CONFIRMED_DEFECT, SUSPECTED_DEFECT, KNOWN_LIMITATION, DOCUMENTATION_DRIFT y EVIDENCE_GAP son etiquetas de esta auditoría, no estados científicos nuevos.

Verificación comprueba ecuaciones/software dentro del dominio ensayado; calibración estima parámetros con observaciones externas; validación compara predicciones con evidencia experimental independiente. La auditoría realiza verificación/falsación y revisión de evidencia. No realizó las otras dos.

## Fuentes normativas y método

Antes de ensayar se inspeccionaron Foundation v0.2, EVIDENCE_POLICY, DoD, capacidad S30, Big Three, hipótesis, limitaciones, riesgos/backlog, documentos calibration y el release real en `docs/FOUNDATION_RELEASE_REPORT.md` (no en raíz). Se contrastaron inventarios de ADR001–037 y fichas S00–S30. Revisión detallada de reportes S11–S12, S16–S30; reportes anteriores e infraestructura se contrastaron con historia, contratos y baseline, sin auditoría exhaustiva de cada assertion o línea temprana.

Se revisó directamente el código de integración motor, eventos/flow/fuel, estados/replay, leak, wear, damage, dyno, decoder/scheduler, sensores/wiring, diagnóstico y fitter/manifest; también dependencias y fronteras SI. Los 30 paquetes se compilaron/ensayaron con tests existentes. La revisión de assembly/math/units/contactos/runner es estructural y de baseline, con algunas APIs ejercidas; no es inspección exhaustiva de todo su código ni prueba de todos los límites.

Se creó un crate independiente con lockfile propio que usa APIs existentes; no sustituyó modelos por mocks. Se diseñaron pruebas nuevas de salida homogénea de fuel, marca pasiva de gas, dimensiones de flujo líquido, angularidad equivalente, cortes de replay, tipo v1 histórico, round-trip JSON, reanclaje de fase, semillas, invalidación de entradas y ajustes con referencia analítica. Barridos de paso/horizonte y muestreo separan discretización de observación. Una inicialización que repite supuestos del Core sirve como fixture, no oracle; soluciones analíticas/invariancias/dimensiones proporcionan los contrastes independientes.

## Resultados ejecutados y significado de los conteos

| Paquete de evidencia | Resultado observado | Lo que no demuestra |
|---|---|---|
| Baseline `cargo test --locked --workspace` | 163 casos pasan, 30 targets unitarios (uno vacío), 29 doc-test targets con cero casos | Ausencia de defectos; independencia de las referencias; validación física |
| Formato / Clippy del CI de producción | Exit0; Clippy con warnings, bajo `-D clippy::unwrap_used` | Cero warnings ni corrección científica |
| Nuevo generador numérico | Un target binario; 138 filas en 42 familias (73 motor,50 subsistemas,15 dyno) | 138 tests o 138 mecanismos independientes |
| Nueva suite contractual Python | Un target,32 casos,88 predicados ejecutados: **9 pasan,23 fallan,0 errors** | 23 defectos distintos; assertions posteriores a un fallo no ejecutadas |
| Repetición completa release en mismo host | Los tres JSONL son **idénticos byte por byte**; mismos 23 failures | Determinismo entre máquinas ni completitud del snapshot |
| Comparación debug/release | 47 filas comunes exactamente iguales | Segundo host; cobertura de los tres probes fitter añadidos después |
| CLI | 15 ejecuciones exit0; release-check anuncia cuatro PASS | Que cada cadena anunciada tenga todos sus eslabones medidos |
| CI documental: dos controles S00 | Ambos exit1 sobre S30 | Resultado de ejecución remota GitHub Actions |

[Log baseline](results/workspace-test.log), [resumen independiente](results/independent-test-summary.json), [log rojo](results/independent-contract-tests.log), [repetición](results/reproduction-comparison.json). Los 582 sitios estáticos de macros assert registrados en inventory.json no se cuentan como assertions ejecutadas. El exit1 contractual se conserva: no se fabricó un PASS invirtiendo expectativas o cambiando umbrales.

## Hallazgos confirmados por severidad

**19 hallazgos confirmados: 2 P0, 12 P1 y 5 P2; ningún defecto confirmado P3.** F15 agrupa huecos de validación de varias APIs, explicitados por separado dentro del hallazgo. No se utiliza este conteo para inferir densidad de fallos o probabilidad.

| Severidad | IDs / mecanismos | Evidencia decisiva |
|---|---|---|
| P0 | F01 combustible no advectado; F02 observación diagnóstica elegida por verdad del escenario | Fuel/masa total alcanza2.3657 sin chispa; salida homogénea pierde masa sin fuel. `LeakSplit` se obtiene por `match LeakScenario`, incluso 10% a escape sin puerto |
| P1 | F03/F04 dyno: aliasing y ventanas de potencia incompatibles | Carga8 residuo .619587→.008276 al registrar cada paso, misma física; canales carga4 no cierran por −2.8964kW |
| P1 | F05/F06/F07 persistencia: pérdida JSON, migración v1 y memoria de presión omitida | Estado legítimo no verifica tras JSON; hash v1 válido rechazado; replay c2=.05 termina175.608 frente a859.536RPM |
| P1 | F08/F09 fugas: especies y unidades de líquidos | Receptor fresco permanece0 con transferencia positiva; volumen de aceite usado como kg, factor870 |
| P1 | F10 ledger térmico; F11 leyes UNSUPPORTED | Omisión de calor al ambiente; aceite UNSUPPORTED devuelve oxidación1.0 |
| P1 | F13 fase decoder; F15 fail-closed | Inicio de vector cambia fase150°, pérdida deja error30° con confianza1; panic/NaN/estado imposible/observaciones inválidas aceptados |
| P1 | F18 incertidumbre del calibrador | std física esperada.25, reportada.08333/.04167 según p0; parámetro ajustado correcto |
| P2 | F12 creep frío; F14 seed wiring; F16 periodicidad; F17 CI; F19 óptimo inicial | Desgaste acumulado decrece sin servicio; seed ignorada; +720° altera motor; checks S00 fallan; óptimo exacto rechazado |

La gravedad P0 deriva del contrato científico central comprometido, aunque la corrección sea pequeña o el fallo aparezca en una demostración particular. F01 rompe presupuesto de especies/química pese a cierre total; F02 invalida evidencia de independencia observacional. Los defectos de persistencia son P1 aun cuando la pérdida de bits JSON sea mínima, porque rompe hash/restore. No se atribuye daño material por interferencia ni vida útil por coeficientes ilustrativos.

La deriva documental se distribuye en 7 P1, 15 P2 y 4 P3 (ninguna P0); las entradas P3 son claridad de conteos, rutas y terminología, con propuestas concretas. Estas entradas no se suman a los 19 defectos de comportamiento.

Hay una sospecha separada R01: firmas diagnósticas mixtas se clasifican como causa única; falta escenario físico multicause válido para confirmar inferencia errónea. K01–K03 son limitaciones reconocidas: subsistemas sin feedback runtime, combustión L1 sin AFR/oxígeno y riesgo/fluencia genéricos sin fractura. G01–G07 describen evidencia faltante: datos experimentales, estabilidad/convergencia general, segundo host/experto, cadenas completas, identificabilidad, electrónica genérica y manifiestos físicos autosuficientes. Ninguna sospecha se cuenta como defecto confirmado.

## Cobertura P0 y áreas explícitamente no examinadas

| Sistema P0 | Examinado y evidencia nueva | Límites abiertos de esta auditoría |
|---|---|---|
| Motor integrado S16 | Masa/energía, especies/fuel, no fuel/no spark/corte, idle/base,3–30s, térmico120s, dt200→6.25µs en subsets, snapshots/replay | Sin dominio completo de estados iniciales/cargas/ambiente; sin periódico multiciclo demostrado ni horizonte físico mayor120s; sin reference solver independiente de todo el motor abierto; ningún banco real |
| Distribución/flujo S11/S12/S16 | ±6/±30/±180° y nominal, +720° metamórfico; refinement angular .25/.125° standalone; eventos IVC/latch y masa/Q/work/RPM dentro del motor | No perfil/flow stock medidos; no aislamiento air timing frente a fuel timing; no barrido integrado exhaust timing/lash completo; no validación de mezcla/combustión; extremos son inputs aceptados, no averías materiales reales |
| Fugas/diagnóstico S23/S24/S29 | Precarga/sellado/área/red, cuatro escenarios, dos pasos, especies/líquidos, reparto frente a CLI, ambigüedad y causa múltiple | Motor vivo sin integración implementada; válvula física ausente; sin instrumento real/tercer test externo; no red de todas las rutas posibles |
| Desgaste/daño S26/S27 | Unidades, film/T, avance50h y subdivisión5000tramos, persistencia JSON, soporte, irreversibilidad, Clear/Risk y perno genérico | Condiciones constantes ilustrativas; sin historia variable/contacto runtime, tribología/materiales, fatiga/scuffing/gripado/rotura o predicción de vida; servicios cubiertos por baseline, no nueva campaña de cada combinación |
| Dyno S28 | Carga4/6/8 a6s, cadencia400/40/4/1 con dt50µs, mediaτω/ventanas, balances, criterios sin cambios y carga8 | Sin barrido completo dt/ventana de todos los puntos, sin banco real/rampas/inercia de freno independiente ni certificación de corrección SAE |

P1: GENERIC/REF frente a OEM, fases CKP/CYP, pérdidas/recuperación, scheduler solicitado/efectivo e inhibición; ECT/MAP/wiring/DTC a través de demos y código. TDC/4+1, transitorios bajo aceleración y ejecución ECU→combustión no quedan demostrados. Infraestructura: DAG sin ciclos Cargo detectados, hash/tamper/schema, true-v1, restore físico, límites dimensionales y determinismo local. No auditoría exhaustiva de dependencia transitiva de terceros, portabilidad, todas las migraciones assembly/runner ni fuzzing global de todas las APIs. Lubricación: fuente/baseline, sin nuevo barrido independiente de red de galerías.

La revisión está limitada a un host y toolchain, APIs públicas disponibles y fixtures deterministas; no hubo acceso a banco físico, datos propietarios, segundo motor, segundo host o revisión de experto de dominio. No hubo un límite de tiempo impuesto por el usuario; se acotaron ensayos a horizontes descritos. No se infiere ausencia de defectos en zonas no examinadas.

## Qué sobrevive y qué debe restringirse

Sobreviven la arquitectura headless modular sin ciclos Cargo detectados, las ecuaciones cinemáticas y del cilindro sellado dentro de los tests, homogeneidad de flujo/Bernoulli, masa total nominal (máximo residuo relativo9.63e-10 en24runs3–30s), respuesta local a combustible/chispa y corte, tendencias ring/bearing/oxidación bajo film/T constantes, control geométrico sin inferir doblado, y separación nominal GENERIC/REF de datos OEM. Cada una conserva sus supuestos y cobertura; no se combinan en una verificación completa del motor.

Cilindro sellado adiabático: errores relativos pV^γ a pasos2/1/.5/.25° son2.987e-7/1.878e-8/1.178e-9/7.356e-11, compatibles con orden4 en ese problema. Ese orden no se transfiere al splitting temporal del motor abierto. Idle fino a8s muestra medias de cola1021.30/1021.18RPM a12.5/6.25µs, evidencia favorable local; a120s térmico medias985.82/928.14/912.39RPM a100/50/25µs dejan convergencia general abierta.

La cadena cam utiliza físicamente lift→área/Cd→flow. No se encontró el acoplamiento artificial llenado/alzada descartado en S30. Las nuevas observaciones IVC/latch/Q/work complementan el antiguo check de RPM. Su amplitud/fidelidad no están calibradas, fuel timing cambia con IC, el inventario de fuel falla y combustión no está limitada por oxígeno; por ello solo sobreviven flechas y tendencias implementadas dentro de esos supuestos.

Deben restringirse las garantías globales de snapshots/replay, decoder sincronizado, fuga líquida/especies, incertidumbre del fitter, protocolo diagnóstico independiente, balance térmico completo, aceptación dyno y «cuatro cadenas completas». La interpretación de carga8 como inestabilidad física debe reevaluarse: el indicador sufre aliasing. No se cambió la tolerancia10%/deriva2%; muestreo resuelto cumple el criterio anunciado en la misma trayectoria, sin demostrar estabilidad de toda la familia.

S30 puede continuar describiéndose como research foundation no predictiva, pero el release requiere un addendum que reconozca estos defectos y la cobertura restringida. La recomendación de reevaluación identifica requisito y evidencia para S16, S18–S26, S28–S30; S27 necesita precisión documental del criterio de fluencia, sin nuevo defecto material demostrado. Tabla detallada en CAPABILITY_RISK_ASSESSMENT. No se revocó automáticamente ninguna aprobación.

## Deriva documental y programa S25

SPRINTS y blockers de HANDOFF están operativamente obsoletos pese a encabezados S30; README/BACKLOG/índice ADR y limitaciones mezclan etapas. Los cuatro briefs históricos están claramente rotulados y no se confirmó doble definición operativa contradictoria. También hay contratos/claims más amplios que código/evidencia, incluido snapshot v1, 4+1, torque llamado kN, ruptura confundida con fluencia y la explicación de carga8. Correcciones concretas y sus autoridades están inventariadas, sin aplicarlas.

S25 permanece sin datos reales. Se propone primero banco de inyectores y MAF/MAP, luego dyno con observables conjuntos, warmup y aceite; adquirir p(θ)/motored conjuntamente si hay laboratorio disponible. Reservar segundo motor desde el inicio y mantenerlo como holdout independiente. Este orden cualitativo combina acceso e identificabilidad; p(θ) tiene gran poder para separar fenómenos, pero exige metrología más costosa. Torque solo no separa combustión/fricción/llenado; presión de aceite sola no identifica film; ECT sola no identifica todos los nodos. Las ocho campañas incluyen variables, procedimiento, incertidumbre, coste/dificultad, dependencias y mínimo suficiente. Fuentes OEM/fabricantes/Sandia/SAE se registran con URL y fecha; proxies de otros motores no validan D16Y7.

## Plan de actuación para una futura fase autorizada

1. Revisar F01/F02 y sus contratos antes de utilizar diagnósticos o resultados para calibración. Diseñar balances de especies y observación independiente con revisores técnicos.
2. Corregir persistencia/migración/estado completo y fronteras fail-closed, con regresiones que continúen siendo rojas hasta resolver cada causa. Revisar ledger térmico y unidades de líquidos.
3. Revisar fase/reanclaje del decoder, soporte/irreversibilidad de desgaste e incertidumbre/óptimo del fitter. Mantener estados no soportados donde falte física.
4. Resolver reporte/muestreo dyno con ventanas únicas y observables integrados; justificar criterios mediante convergencia y datos, sin ajustar parámetros/umbrales para ocultar residuos.
5. Publicar propuestas documentales en una revisión separada, preservando textos/gates históricos y registrando reevaluaciones, y ejecutar el plan externo S25 solo con instrumentación/procedencia adecuadas.

Es una propuesta de correcciones, no implementación ni continuación del roadmap. S31 permanece prohibido. No se presume autorización para merge ni ensayos físicos/pagos.

## Cierre de los ocho gates de esta auditoría

| Gate de auditoría | Evidencia de cierre |
|---|---|
| 1: todos los P0 examinados o zonas declaradas | Tabla de cobertura anterior con áreas no examinadas |
| 2: confirmados reproducibles/suficientes | F01–F19, harness y logs; F02/F10/F17 además con código/dimensiones/controles directos |
| 3: riesgos separados | R01, K01–K03 y G01–G07 no contados como defectos |
| 4: deriva documental concreta | Inventario D01–D26 con propuestas sin edición |
| 5: capacidades reflejan resultados | 18 filas reevaluadas y requisitos de gates explicitados |
| 6: S25 sin inventar datos | Ocho vacíos, plan priorizado y registro de fuentes |
| 7: modelos/gates intactos | Comparación de todos los paths versionados y worktree original en production-isolation.json |
| 8: límites declarados | Host, horizonte, cobertura, datos/experto y APIs explicitados |

La misión concluye con evidencia, diagnóstico y plan de revisión. Completar estos entregables no equivale a PASS del Engine Core ni a ausencia de defectos.
