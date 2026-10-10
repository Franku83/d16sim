# D16SIM — Independent Audit Remediation R01–R05

**Versión:** 1.0 · **Fecha:** 2026-10-10 · **Estado:** PROPUESTA DE EJECUCIÓN; no acredita correcciones realizadas.

**Base de producción auditada:** `ff09a0dda82188e6bf27a80d17c902f8972a39d2` (S30).  
**Auditoría independiente:** rama `audit/independent-2026-10-10`, commit `25d528e`.  
**Proveniencia:** `docs/audit/FINDINGS.md`, `REPRODUCTION.md`, `INDEPENDENT_AUDIT_REPORT.md`, `DOCUMENTATION_CONSISTENCY.md`, `CAPABILITY_RISK_ASSESSMENT.md`, `S25_EVIDENCE_ACQUISITION_PLAN.md` y los contratos del repositorio. Este documento organiza hallazgos existentes; **no afirma haber auditado o corregido código nuevo**.

> **Regla central:** corregir los defectos que el auditor pudo falsar sin convertir una research foundation L1 no calibrada en un supuesto D16Y7 predictivo. Preservar evidencia, permitir nuevas cifras físicas que resulten de correcciones causales, nunca modificar fixtures/tolerancias para simular éxito.

## 0. Alcance, nomenclatura y autorización

- **R01–R05 de este plan** son etapas de **remediación**. El identificador **`R01` de la auditoría** es una *sospecha* de clasificador multicause, **no** la etapa R01; cuando se cite se escribirá `AUD-R01`.
- Los **19 hallazgos confirmados F01–F19** se asignan **exactamente una vez** a una etapa propietaria. Las **26 derivas D01–D26** se resuelven documentalmente en R05, aunque cada corrección de comportamiento debe incluir la actualización documental técnica mínima de su etapa.
- **Bloques de autorización:** batch 1 = **R01–R02**; batch 2 = **R03–R04**; cierre = **R05**. **Detenerse después de cada batch y esperar aprobación explícita**. No comenzar R03 por acabar R02 ni empezar R05 por acabar R04. Se pueden usar revisiones/hotfixes para defectos bloqueantes solo tras aprobación, con trazabilidad.
- **S31–S37 prohibidos**; no renumerar ni reescribir S00–S30. No iniciar campañas físicas, gastos, adquisición de bancos ni búsqueda de datos para entrenar sin autorización. S25 permanece sin calibración/validación experimental.
- Crear rama/worktree de remediación desde el estado de producción autorizado (idealmente `remediation/r01-r05`). Conservar **separado e inmutable** el checkout original del auditor, los 23 contratos rojos, probes, logs y hashes. Referenciar la auditoría desde la rama de remediación mediante commit/referencias, sin alterar sus expectativas o umbrales.
- Mantener revisión por agente **distinto del implementador** para aceptar cada gate; si no está disponible, emitir `IMPLEMENTED_PENDING_INDEPENDENT_REVIEW`, nunca afirmar PASS independiente.

### Inventario de responsabilidad sin omisiones

| Etapa | Confirmados asignados | N.º | Resultado exigible |
|---|---|---:|---|
| **R01 — Balances y diagnóstico observable** | F01, F02, F08, F09, F10 | 5 | Conservación por especie, unidades de líquidos, calor exterior e instrumentos sin verdad del escenario |
| **R02 — Estado, replay y fail-closed** | F05, F06, F07, F15 | 4 | Snapshots compatibles y completos, restore y APIs con validación estricta |
| **R03 — Modelos y control** | F11, F12, F13, F14, F16, F18, F19 | 7 | Estados UNSUPPORTED, irreversibilidad, fase, seeds, periodicidad e incertidumbre correctos |
| **R04 — Dyno auditable** | F03, F04 | 2 | Muestreo representativo, ventanas alineadas y potencias integradas |
| **R05 — CI, documentación y release restringido** | F17 | 1 | Checks vigentes, D01–D26 resueltas y reevaluación de capacidades/gates |
| **Total** | **F01–F19, sin duplicados** | **19** | **No incluye como defectos AUD-R01, K01–K03 o G01–G07** |

### Estados permitidos para cada issue

`OPEN` · `IN_PROGRESS` · `FIXED_PENDING_VERIFICATION` · `VERIFIED_BY_INDEPENDENT_REVIEW` · `BLOCKED` · `RECLASSIFIED_WITH_EVIDENCE`. No basta `tests green`. Toda reclasificación necesita explicación, reproducción y revisión. Limitaciones de alcance (`K*`) y vacíos externos (`G*`) reciben estados *tracked/open*, no `FIXED` por código no implementado.

## 1. Preparación común — antes de R01

1. Registrar SHA del commit base S30, del commit auditor y del HEAD de remediación; `git status --short` limpio. No hacer merge prematuro a la rama principal.
2. Reproducir el **baseline** en el entorno disponible: `cargo test --locked --workspace` (la auditoría contó **163 casos**, no “59 tests”; 30 targets unitarios y 29 doc-test targets sin casos), `cargo fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets -- -D clippy::unwrap_used` (exit0 con warnings). Registrar versión Rust/Cargo, OS y flags.
3. Reproducir harness independiente sin cambiarlo: `bash audit/independent/reproduce.sh`, desde el checkout donde exista `audit/independent/`; en el commit auditado el **resultado esperado era exit 1, 23 fallos, 9 aprobados**. El generador produce 138 filas / 42 familias, **no 138 tests**. Conservar resultado íntegro y distinguir build/harness/test failures.
4. Establecer mapeo `F-ID → fixture/caso → causa raíz → código corregido → regresión → evidencia → revisor`. El harness externo debe volver a ejecutarse en cada etapa; sus fallos no asignados a la etapa pueden continuar pendientes, documentados.
5. Para cada defecto, producir primero un test rojo mínimo que falle contra la base auditada. Luego corregir **causa raíz**, evaluar contrafactuales, comprobar invariantes y regresiones. Los tests de propiedades físicas no deben codificar una salida exacta del solver como su único oráculo.
6. Preservar snapshots v1 originales y los actuales v2. Cambios de serialización/contratos públicos requieren control de compatibilidad + ADR, no reemplazo silencioso.
7. Crear evidencia por etapa en `docs/remediation/R0X/` (puede ajustarse a convenciones del repo sin perder nombres/evidencia) y registrar cambios en `PROGRESS.md` mediante **nueva entrada**; no modificar resultados históricos.
8. Cada corrección que cambie una demo debe registrar valores **antes/después**, explicación causal, parámetros constantes y por qué siguen válidas —o dejan de estarlo— las afirmaciones previas.

### Política de parada transversal

Detener el batch y solicitar decisión si: hace falta alterar Foundation/DoD/evidence policy, cambiar arquitectura o esquema incompatible sin migración probada, fabricar coeficientes/datos, añadir física de válvulas/CFD/ECU runtime fuera de alcance, romper un contrato normativo sin solución trazable, o fallar una reproducción de auditoría por entorno no diagnosticado. Un gate parcial debe declararse **BLOCKED** con pruebas y deuda, no PASS.

---

## 2. R01 — Integridad de especies, flujos y observación diagnóstica

**Defectos propios:** F01 (P0), F02 (P0), F08 (P1), F09 (P1), F10 (P1).  
**Dependencia:** base S30 + fixtures originales. **Salida:** contrato físico de transporte, presupuesto químico/energético y observaciones independientes.

### R01-F01 · Transporte de combustible en el Engine Core — P0

- **Fuente:** `crates/engine/src/run.rs` ~644–687 y ~732–787; auditoría `fuel_outflow`, `fuel_inventory`, contrato Python **caso 10**.
- **Fallo observado:** masa de gas decrece, pero `c.fuel` no decrece con salidas; el inventario de combustible alcanzó **2.3657034×** la masa total sin chispa. El plenum/escape no transporta dicho componente.
- **Objetivo:** conservar masa de cada especie/componente definido y energía química, manteniendo separados combustible inyectado, almacenado, transferido a nodos, quemado y emitido. Establecer explícitamente si el estado representa vapor, mezcla homogénea u otro modelo L1; **no fingir atomización o límites de oxígeno no modelados**.
- **Solución exigida:** diseñar esquema coherente de estado + flujos upwind/bidireccionales; restar componentes proporcionalmente en salida y transportar/reintegrar en reflujo; contabilizar fronteras e inventario por cilindro/nodo. Controlar límites `0 ≤ m_fuel ≤ m_total` cuando la representación lo exige; un chequeo tardío/clamp no sustituye advección. Revisar el `latch` de quemado tras corregir el presupuesto.
- **Regresión:** flujo homogéneo por admisión y escape, inversión de flujo, cranking sin chispa prolongado, inyección/corte, conservación `fuel_injected = fuel_stored + fuel_burned + fuel_emitted` en frontera declarada; `m_fuel ≤ m_total`; refinamiento temporal; ninguna energía química sin masa quemada.
- **Gate:** el caso 10 y nuevos controles de especies pasan con tolerancia justificada, sin introducir combustible virtual ni alterar la física para preservar RPM previas. Reportar cambios nominales de RPM/trabajo si aparecen.

### R01-F02 · Independencia de la medición diagnóstica — P0

- **Fuente:** `crates/cli/src/main.rs` ~1681–1705 / ~1741–1755, `mech_demo_cmd`; auditoría `leak_scenario` y `mech-demo missing_head_gasket`.
- **Fallo observado:** reparto `LeakSplit` determinado por `match LeakScenario` (0.30/0.10/0.60) en lugar de flujos reales (~0.14/0/0.86); el montaje ni siquiera tiene puerto de escape para esa fuga.
- **Objetivo:** las observaciones llegan **solo** a través de instrumentos/red física/protocolo, nunca del identificador usado para construir la avería.
- **Solución exigida:** desacoplar **scenario construction** de **observation construction**; calcular reparto medible desde caudales/nodos o una sonda físicamente definida (con error/alcance declarado), **sin añadir un puerto inventado**. Si no existe sensor o canal suficiente, exponer `INCONCLUSIVE`/`UNSUPPORTED`, no un porcentaje heurístico presentado como medida. Mantener tests de clasificador con firmas sintéticas etiquetadas como fixtures **separados** de demos físicas.
- **Regresión:** renombrar/reordenar escenarios preservando inputs físicos ⇒ medición idéntica; diferentes causas con mismas observaciones ⇒ ambigüedad; ninguna dependencia de enumeraciones de verdad en el camino instrumento→diagnóstico; tercer test solo si realmente observable. Revisar `AUD-R01` como **sospecha**, no asumir resuelta.
- **Gate:** `mech-demo` informa observaciones originadas en física/instrumentos, sus límites y procedencia. El gate S24 se **recomienda reevaluar**, no reescribir retrospectivamente.

### R01-F08 · Advección de especies en red de fugas — P1

- **Fuente:** `crates/leak/src/network.rs` ~229–245, `leak_species`, **caso 15**.
- **Fallo:** masa transferida **0.000466712 kg**; fracción fresca receptora esperada **0.00400230777**, observada 0.
- **Solución:** transportar masa fresca/residual de donante a receptor, mezclar con inventarios físicos y respetar fronteras/regulador; garantizar consistencia para dirección inversa.
- **Regresión/gate:** marca pasiva homogénea 1→0, balances de ambas especies y masa total con tolerancias e independencia de resolución; caso 15 pasa.

### R01-F09 · Unidades para flujo líquido — P1

- **Fuente:** `crates/leak/src/network.rs` ~212–215, `leak_liquid_units`, **caso 16**.
- **Fallo:** se usa volumen como kg; **1e−6 m³ × 870 kg/m³ = 0.00087 kg**, pero se reportó ~1e−6 kg (**factor 870**).
- **Solución:** tipar o documentar dimensionalmente conductancia, viscosidad y densidad; convertir `m³/s → kg/s` antes de actualizar masa y entalpía; no aplicar densidad dos veces en flujos gaseosos.
- **Regresión/gate:** dos densidades con mismo volumen, solución analítica de ranura viscosa, presupuesto de líquido/entalpía y rechazo de densidad inválida; caso 16 pasa.

### R01-F10 · Auditoría energética de sumideros térmicos — P1

- **Fuente:** `crates/thermal/src/network.rs` ~185–203; `crates/engine/src/run.rs` ~909–938; `thermal_boundary`, `thermal_long`.
- **Fallo:** la red caliente pierde **13064 J** hacia ambiente (cierra localmente), pero ledger del motor no incluye el término; residuo ~−0.0039 a −0.0036 persiste con refinamiento en el experimento.
- **Solución:** contabilizar frontera ambiente/radiador y calor integrado en misma convención temporal y de signos; distinguir pérdidas físicas del error de splitting/cálculo. No forzar residuo cero ocultando términos.
- **Regresión/gate:** adiabático, frontera abierta, cold/hot-start, balances por período y refinamiento; residual explicado y dentro de criterio numérico documentado. **No hay caso Python dedicado en los 32 originales:** añadir regresión específica sin falsear conteos.

### DoD R01

- F01/F02/F08/F09/F10 con evidencia de causa y regresión por ID; preservación de mass/energy por frontera aplicable; `mech-demo` sin acceso a la etiqueta verdadera; cambios de comportamiento y de esquema documentados.
- `cargo test`, fmt, Clippy y suite independiente; **no exigir que fallos propios de R02–R04 estén corregidos prematuramente**. Commit único de etapa (`remediation(R01): ...`) o commits pequeños con cierre de etapa identificable, según política; NO merge ni R03.

---

## 3. R02 — Persistencia física, migraciones y fronteras fail-closed

**Defectos propios:** F05 (P1), F06 (P1), F07 (P1), F15 (P1).  
**Dependencia:** R01 PASS/revisión; restaurar y probar también los nuevos inventarios de especies introducidos en R01.

### R02-F05 · Round-trip JSON y hash sin pérdida — P1

- **Fuente:** `crates/engine/src/state.rs` ~69–82; `crates/wear/src/state.rs` ~162–165; `engine-types` JSON; probes `snapshot_json`, `wear_roundtrip`, **casos 12–13**.
- **Fallo:** `122.39085530309971 → 122.39085530309973`, con rechazo de hash legítimo (también desgaste).
- **Solución:** identificar exactamente la ruta de serialización/parseo y sus features; definir bytes canónicos **versionados** o representación lossless de f64 en lectura/escritura; verificar valor original tras round-trip; separar integridad del archivo de validación de estado. No invalidar archivos previos mediante cambio no versionado de formato.
- **Regresión:** archivos reales motor/desgaste, semillas, extremos f64 finitos, paths serde reales, round-trip disco, hashes, resume y replay. En ambiente reproducible, bit-identidad donde el contrato la promete. Casos 12–13 pasan.

### R02-F06 · Migración auténtica v1→v2 — P1

- **Fuente:** `engine/src/state.rs` ~44–45, `engine/src/run.rs` ~356–367; tipo v1 histórico `git show 2541266:crates/engine/src/state.rs`; **caso 14**.
- **Fallo:** hash v1 legítimo rechazado porque se agrega `thermal:null` antes de verificarlo.
- **Solución:** detectar versión, validar hash de bytes/representación bajo **esquema original**, luego migrar explícitamente; almacenar procedencia y versión transformada; fallar cerrado en unknown/tampered/ambiguous. Evitar que defaults cambien identidad histórica.
- **Regresión:** v1 real válido, v1 alterado, v2 válido, versión desconocida, datos no finitos, hash de ejemplo histórico y errores tipados. Caso 14 pasa.

### R02-F07 · Historial de presión necesario para fricción — P1

- **Fuente:** `crates/engine/src/run.rs` ~603,624,857–861; state ~17–45; probe `replay_split`, **caso 11**.
- **Fallo:** con `c2=.05`, continuo **175.6084 rpm** vs reanudado **859.5357 rpm**, mientras control `c2=0` coincide.
- **Solución:** incorporar al estado persistente cada variable de historia que afecte la ecuación de par (incluido máximo de presión y su frontera física), sin reinicio artificial por llamada. Revisar inventario de integradores y estados slow/thermal/valve/actuación agregados en R01.
- **Regresión:** múltiples cortes arbitrarios de ciclo con `c2≠0`, distintas duraciones, identidades físicas y hash, replay vía memoria y disco, control nominal. Caso 11 pasa.

### R02-F15 · Matriz de validación fail-closed en APIs — P1 transversal

- **Fuente:** run.rs ~963–980/356–367/880; thermal `network.rs`; dyno `brake.rs`; ECU `decoder.rs`; calibration `dataset.rs`; **casos 24–27,29–30**.
- **Fallas:** `record_every=0` panic; `ηcomb` negativa o mayor a 1 aceptada; restore con masa negativa y hash válido; capacidad térmica cero da estado no finito; dyno a −300 °C devuelve factor inválido; CKP NaN o futuro aceptado; manifiestos con no finitos/σ inválida/metadata incompleta aceptados.
- **Solución:** contratos **por API y unidad**, validación *antes* de mutaciones/divisiones/módulos/solver y errores tipados; prohibir NaN/Inf, negativos imposibles, capacidades cero y observaciones futuras. Separar hash válido de estado físico válido; para signo de torque accesorio precisar el contrato antes de rechazarlo (el auditor no lo contó como defecto autónomo). Rechazar inputs incompletos en manifiesto; conservar política de datasets sintéticos y holdout.
- **Regresión:** tabla pública por campo (cero/negativo/NaN/Inf/límites), rollback sin mutación, restore inválido con hash correcto, causalidad temporal, ninguna panic, errores estables; **sin tocar semántica de fase F13 (R03)** salvo protección de entradas.
- **Gate:** seis casos contractuales vuelven a pasar; auditoría adicional de API declarada y errores. No afirmar fuzzing exhaustivo ni seguridad universal.

### DoD R02

- Los cuatro IDs corregidos sin degradar F01/F02/F08/F09/F10; nuevo estado R01 serializado y migrado correctamente; matriz de fronteras públicas ejecutable.
- Registrar compatibilidad, tipos históricos, ADR de versionado si cambia contrato. Reejecutar baseline, auditor y replay continuo/dividido para varios esquemas/configuraciones. **STOP obligatorio tras R02.**

---

## 4. R03 — Semántica física, decoder, aleatoriedad y fitter

**Defectos propios:** F11 (P1), F12 (P2), F13 (P1), F14 (P2), F16 (P2), F18 (P1), F19 (P2).  
**Requiere autorización para batch 2 (R03–R04).** No ampliar runtime ECU→combustión o modelos de materiales sin permiso.

| ID | Fallo auditado / evidencia | Solución objetivo | Gate / regresión |
|---|---|---|---|
| **F11** | `wear/state.rs` ~137–144: ley `UNSUPPORTED` produce oxidación **1.0** con coeficientes inválidos; **caso 19** | Validar soporte de **todas** las leyes antes de avance atómico; resultado tipado `UNSUPPORTED`, nunca representar desconocido como cero o evolución cuantificada; no mutar estado si falla | Todas las combinaciones supported/unsupported, replay sin evolución falsa; caso 19 PASS |
| **F12** | `wear/state.rs` ~32–60,148–150: a −20 °C creep .1→.099990758 sin servicio; **caso 20** | Definir dominio en Kelvin/temperatura físicamente defendible; ley acumulativa sin signo invertido o rechazo explícito en frío; sin curación espuria | Negativos °C aceptables como ambiente solo si ley los soporta, frío/caliente, subdivisión, servicio/no-servicio, monotonicidad; caso 20 PASS |
| **F13** | `ecu/decoder.rs` ~180–195: mismo CYP pero vector desplazado 5 CKP cambia **690→540°**; dropout deja **30°** con confianza 1; **casos 21–22** | Anclar fase a referencia física CYP/TDC del patrón **GENERIC_TRIGGER**, manejar pérdida/reanclaje y degradación de confianza; no inferir 4+1 OEM no implementado | Origen arbitrario, dropout, CKP extra, CYP recuperado, aceleración y fase de scheduler. Casos 21–22 PASS; sin declarar Honda OEM |
| **F14** | `electrical/wiring.rs` ~64–85: constructor usa seed fija 7193, ignora entrada; **caso 23** | Semilla/stream parametrizan ruido intermitente y persisten en reproducibilidad | Mismo seed+stream idéntico; streams diferentes muestran diversidad estadística sin exigir diferencia por única muestra; caso 23 PASS |
| **F16** | `engine/run.rs` ~469–476/~693–695: centerline+720° mantiene lift y cambia RPM **1059.86 vs 1025.82**; **caso 28** | Normalizar dominio angular de perfil/IC/cruces de eventos coherentemente o rechazar explícitamente entrada fuera de dominio | Equivalencia ±720°, cruce de 0, +/− phase offsets, desacoplar timing de inyección del aire al probar causalidad; caso 28 PASS |
| **F18** | `calibration/fit.rs` ~50–55,161–172: ajuste `p=2` con std .08333/.04167 en vez de **.25**; **caso 31** | Transformar covarianza `Cov_p = D Cov_q D` desde coordenadas escaladas, unidades físicas y supuestos estadísticos explícitos; actualizar salidas consumidoras | Referencia f(p)=p², independencia de p0 y escala/unidad, cov multivariada y singularidad; caso 31 PASS |
| **F19** | `calibration/fit.rs` ~59–125: p0=2, residuo 0 ⇒ «no descent»; **caso 32** | Evaluar convergencia inicial antes de backtracking; distinguir óptimo, estancamiento, singularidad y fallo real de descenso | Óptimo exacto/casi exacto y negativos válidos; caso 32 PASS |

**DoD R03:** siete defects con reproducciones y verificaciones específicas; actualizar documentación de alcance de decoder/calibración/desgaste afectada; no anunciar estrategia P2E OEM ni intervalos experimentales. Controlar interacciones de F16 con R01 inventario fuel y eventos.

---

## 5. R04 — Integridad de medición y energía del dyno

**Defectos propios:** F03 (P1), F04 (P1).  
**Dependencia:** R03 PASS; conservar fijos modelo de motor, parámetros y trayectoria entre cadencias de medición.

### R04-F03 · Aliasing de muestras y punto carga-8

- **Fuente:** `crates/dyno/src/sweep.rs` ~110–128/~159; `crates/engine/src/run.rs` ~880–889; **caso 17**.
- **Fallo:** misma trayectoria y dt=50 µs, residuo **0.619587** a `record_every=400` frente a **0.0082761** a cada paso; deriva RPM densa 0.005239.
- **Solución:** integrar torque/trabajo con representación temporal resuelta (o observación de cada paso) y agregar por **ciclos completos/ventanas explícitas**; no usar el stream de salida submuestreado para auditoría física; probar sensibilidad a fase/cadencia.
- **Gate:** carga-8 se reclasifica por resultados de observación corregidos, con tolerancias **sin relajación oportunista**. No declarar estabilidad general por un solo caso; caso 17 PASS.

### R04-F04 · Potencias sobre ventanas incompatibles

- **Fuente:** `crates/dyno/src/sweep.rs` ~134–153; **caso 18**.
- **Fallo:** para carga-4 `Pind − Pfric − Paux − Pbrake − Pacc = −2.89643328 kW` usando canales publicados; `Pacc` usa todo 6 s y el resto cola, y promedios τ y ω se multiplican incorrectamente.
- **Solución:** definir y conservar ventanas global / estacionaria, integrar `∫τ·ω dt` para cada término, `ΔKE/Δt` de la **misma** ventana, distinguir potencia transitoria vs estacionaria, carga y auxiliar. Exponer denominador/ventana/rango en `SweepResult`/manifiesto.
- **Gate:** identidad energética independiente por cada ventana y criterio de muestreo; Covered y Unsettled se reportan sin ocultar datos; caso 18 PASS.

**DoD R04:** resultados de cargas 4/6/8, muestreos 400/40/4/1, ventanas iguales, integración de energía y cierre; conservar el historial de carga-8 `Unsettled` como resultado de versión antigua, no reescribirlo como si nunca ocurrió. **STOP obligatorio tras R04.**

---

## 6. R05 — CI documental, D01–D26 y reevaluación de release

**Defecto propio:** F17 (P2). **Dependencia:** batch 2 aprobado y finalizado; requiere autorización independiente para R05. **Es un cierre documental y de verificación, no un nuevo sprint.**

### R05-F17 · Workflow documental vigente

- **Fuente:** `.github/workflows/docs-check.yml` ~26,31; auditor `docs-ci-controls.json`.
- **Fallo:** checks S00 aún exigen **12 ADR** y prohíben crates, pero release tiene **37 ADR y 30 paquetes**; comandos locales exit1.
- **Solución:** checks basados en inventario/expectativas fase actual y requisitos normativos, no prohibición de crates ni conteos de bootstrap. Asegurar que el control falle si falta un documento realmente requerido. Probar comandos locales y (si disponible) CI remota sin afirmar que corrió cuando no.
- **Gate:** validación positiva con estado real, negativa al eliminar/corromper índice obligatorio, sin debilitar verificación documental; F17 resuelto.

### Matriz completa de 26 derivas documentales

Todas las correcciones conservan fecha/historia y fuente normativa; el archivo `DOCUMENTATION_CONSISTENCY.md` es la autoridad del diagnóstico, el código+pruebas de R01–R04 son autoridad para afirmar que un defecto quedó corregido.

| ID | Sev. | Ubicación / contradicción | Corrección exigida y comprobación |
|---|---|---|---|
| **D01** | P2 | `SPRINTS.md` encabezado S00–S25 | Estado operativo **S00–S30 congelado**, índices hasta S30, sin aprobar S31 |
| **D02** | P2 | `HANDOFF.md` dice 25 crates | Inventario actual **30 paquetes** verificado por `cargo metadata`; distinguir dirs vs nombres Cargo |
| **D03** | P2 | `HANDOFF.md` hipótesis solo H-S25 | Incluir H-S26–H-S30 y referencias existentes, sin alterar sus afirmaciones |
| **D04** | P2 | `HANDOFF.md` deudas S26–S30 como futuros | Separar entregas ejecutadas, defectos corregidos y bloqueos externos vigentes |
| **D05** | P2 | `BACKLOG.md` S11–S13 duplicados Pendiente/APROBADO | Una tabla vigente, historia explícitamente rotulada si se conserva |
| **D06** | P2 | `BACKLOG.md` dice S26–S30 autorizados, no entregados | Estado entregado + remediación/limitaciones actuales y veto S31 |
| **D07** | P2 | `README.md` afirma sin motor físico y S11–S37 pendientes | Presentar Core L1 existente, sin calibración/predictividad; S31–S37 pendientes |
| **D08** | P2 | `README.md` inventarios/ADRs/fichas obsoletos | Vincular índices vigentes 30 paquetes/37 ADR/31 fichas (recontar al ejecutar) |
| **D09** | P3 | `README.md` rutas research con `../` | Corregir a rutas reales desde raíz; mantener rutas correctas desde `docs/` |
| **D10** | P2 | `docs/adr/README.md` índice termina ADR017 | Indexar ADR018–037 existentes y comprobar enlaces; nuevos ADR R* por separado |
| **D11** | P1 | `docs/limitations.md` se quedó en S00/S20 | Addendum vigente por módulo y restricciones de auditoría, preservar historia |
| **D12** | P2 | `RISK_MATRIX.md` solo matriz de S00 | Addendum residual actual con owner/evidencia/severidad, sin falsificar pasado |
| **D13** | P2 | `calibration/model_cards.md` solo inventario S25 | Cobertura wear/damage/dyno/trigger y faltantes de integración; distinguir modelo y referencia |
| **D14** | P3 | “thermal 10 tests” suma crates distintos | Desglosar 1 thermal + 9 engine en ese momento; inventario real actual aparte |
| **D15** | P1 | `snapshot-contract.md` promete v1→v2 inválido en base | Actualizar con evidencias **R02-F05/F06/F07** y compatibilidad garantizada solo ensayada |
| **D16** | P1 | `physics/integrated_engine.md` dice determinismo total | Delimitar configs/host/horizontes, corregido R02 y faltantes externos; no garantía universal |
| **D17** | P2 | Release llama “torque→sellado” a 100/200 kN | Nombrar correctamente **precarga (N)→contacto→fuga**; no añadir flecha torque no ejercida |
| **D18** | P1 | Cuatro cadenas release “completas” sin eslabones medidos | Tabla cada eslabón observado/no observado, enlazar F01/F02/F13 y G04 |
| **D19** | P2 | `FOUNDATION_RELEASE_REPORT.md` cifras RPM no reproducidas | Ejecutar comando/inputs actuales, reportar addendum con versión; no inventar origen antiguo |
| **D20** | P1 | `physics/damage.md` llama ruptura a Rp0.2 | Corregir a **inicio de plasticidad genérico**, rotura UNSUPPORTED |
| **D21** | P2 | `docs-check.yml` bootstrap S00 | Resolver con **F17** y pruebas de CI positivo/negativo |
| **D22** | P3 | Ayuda CLI dice “S02 plumbing only, no physics” | Ayuda vigente; aclarar que subcomandos init/replay de runner siguen plumbing |
| **D23** | P2 | `calibration/campaigns.md` sobreafirma film desde presión/p(θ) | Limitar identificabilidad, exigir observables adecuados, plan MAF/motored/holdout sin fingir dataset |
| **D24** | P3 | `EXPERIMENTALLY_VALIDATED` vs taxonomía `VALIDATED_IN_DOMAIN` | Usar taxonomía normativa, aclarar equivalencias de prosa sin elevar capacidades |
| **D25** | P1 | S18/ADR025 anuncian decoder 24+1/4+1 | Delimitar lo realmente implementado GENÉRICO CKP/CYP; 4+1/TDC pendiente salvo nueva prueba aprobada; F13 |
| **D26** | P1 | H-S28-04 “carga-8 no estabiliza” sin evidencia suficiente | Preservar observación histórica `Unsettled`, anotar F03 aliasing y resultado de R04 sin atribuir inestabilidad física automática |

**Nota D sobre historia:** cuatro briefs `S27-histórico`–`S30-histórico` estaban **correctamente rotulados**; el auditor **no** confirmó doble definición operativa. Si se reordenan, registrar como mejora editorial, **no** como defecto adicional ni nueva aprobación.

### Reevaluación S30 y limitaciones abiertas

- Producir `docs/remediation/REMEDIATION_RELEASE_ADDENDUM.md` con matriz de **19 hallazgos** (estado, evidencia, versión, revisor); matriz D01–D26; `KNOWN_LIMITATION`, `EVIDENCE_GAP`, riesgos y gates históricos afectados.
- Reevaluar **requisitos concretos**, no borrar aprobaciones pasadas ni marcar «S16–S30 PASS» nuevos por simple suite verde. Posibles gates S16, S18–S26, S28–S30 y alcance S27; conservar control estricto de denominaciones (VERIFIED_NUMERICALLY, CALIBRATED, VALIDATED_IN_DOMAIN, UNSUPPORTED).
- Confirmar que los cambios R01–R04 no introdujeron regresiones físicas ni nuevos defectos por acoplamiento. Mantener tests originalmente verdes y ejecutar suite adversarial nueva sobre la **versión corregida**, sin editarla para obtener verde.
- No afirmar segundo host, experto externo, OEM exacto, p(θ), dyno medido, estabilidad global o validación D16Y7 si siguen sin evidencia.

### DoD R05

F17 cerrado; **26/26 D** cerradas mediante actualización verificable o escaladas `BLOCKED` con razón aprobada; addendum S30 honesto; matrices/tests reproducibles; `git diff` auditado; sin merge automático; veto S31 conservado. El cierre de R05 puede ser `BLOCKED` si algún requisito no puede verificarse, y debe reportarse como tal.

---

## 7. Obligaciones que NO son correcciones de F01–F19

Estos elementos del auditor permanecen visibles para impedir un falso “todo resuelto”. No se pueden etiquetar `FIXED` solo porque cambió un módulo.

| ID | Clase | Tratamiento en remediación |
|---|---|---|
| **AUD-R01** | `SUSPECTED_DEFECT` | Diseñar prueba diagnóstica multicause físicamente observable; con F02 corregido puede quedar `UNCONFIRMED` o convertirse en defecto con evidencia. No simular una válvula inexistente para cerrar el riesgo. |
| **K01** | `KNOWN_LIMITATION` | Fugas, lubricación, wear, damage, actuación ECU **no cierran un motor runtime completo**. R01 corrige especies/diagnóstico pero NO autoriza esa expansión. |
| **K02** | `KNOWN_LIMITATION` | Llenado/combustión L1 no equivalen a AFR/oxígeno real atrapado y variabilidad validada; cambiar fuel no autoriza afirmación OEM. |
| **K03** | `KNOWN_LIMITATION` | Contacto/fluencia genérica ≠ fractura ni predicción de vida útil. |
| **G01** | `EVIDENCE_GAP` | Cero calibración/validación experimental del D16Y7; ocho vacíos S25 abiertos. |
| **G02** | `EVIDENCE_GAP` | Estabilidad/convergencia temporal y numérica global del motor abierto sin demostrar. |
| **G03** | `EVIDENCE_GAP` | Segundo host/experto independiente siguen pendientes si no se ejecutan. |
| **G04** | `EVIDENCE_GAP` | Cuatro release-checks no prueban cuatro cadenas causales completas; documentar eslabones. |
| **G05** | `EVIDENCE_GAP` | Identificabilidad S25 necesita campañas/rango de Jacobiano; calibrador correcto ≠ parámetros identificados. |
| **G06** | `EVIDENCE_GAP` | TDC/4+1/ECU OEM/integración de actuación no demostrados; F13 corrige un decoder genérico, no los convierte en OEM. |
| **G07** | `EVIDENCE_GAP` | Los manifiestos físicos no prueban por sí solos ejecución/completitud de experimento; exigir metadata/procedencia verificable. |

### Programa S25 externo — NO ejecutar en esta remediación

Conservar `S25_EVIDENCE_ACQUISITION_PLAN.md` como backlog independiente, incluyendo **ocho vacíos**: inyectores; MAF/MAP; dinamómetro; warmup; aceite; p(θ); motored; segundo motor. Plan priorizado cualitativamente; ningún dataset inventado, ninguna curva stock inferida desde proxy, ningún gasto autorizado. **Primero reparar F01/F03–F10/F15/F18 según aplicabilidad** para evitar que parámetros absorban errores de software.

### Funcionalidad NO autorizada adicional

No implementar en R01–R05 ajuste físico del distribuidor, ECU OEM, closed-loop, manifold 1D, modelo de válvula rota, fuga→engine runtime, CFD/FEA, calibración o sprints S31+. Cualquier mejora funcional necesaria para reparar un F concreto debe ser la **mínima suficiente** y estar justificada; si exige expansión de alcance, pausar y solicitar permiso.

---

## 8. Protocolo de ejecución por batches

### Batch 1 — R01 + R02 (primera autorización)

1. Congelar/evidenciar baseline; abrir worktree de remediación.
2. R01: reproducir, corregir y documentar F01/F02/F08/F09/F10; nuevo test rojo antes del fix; tests/contrafactuales/ledger; revisión de etapa.
3. R02: reproducir, corregir y documentar F05/F06/F07/F15, **incluyendo** estado nuevo de R01; migraciones reales, replay multi-split, validación fail-closed.
4. Reejecutar baseline, independent checker y resultados; registrar lista `resolved / still failing / new regression / blocked` por F-ID sin esconder fallos de etapas posteriores.
5. Emitir `R01_R02_BATCH_REPORT.md`, commits identificables, estado de rama y evidencia; **STOP, no R03 hasta autorización**.

**Gate de continuidad:** ambas etapas con DoD y revisión independiente. Si un P0 queda bloqueado, **no iniciar R02 automáticamente** salvo autorización explícita para pruebas/documentación no dependientes (sin declarar batch completo).

### Batch 2 — R03 + R04 (segunda autorización)

1. Partir del HEAD R02 revisado; ejecutar R03 F11/F12/F13/F14/F16/F18/F19 con fixture independiente.
2. Ejecutar R04 F03/F04; muestreo resuelto, ventanas iguales, cambio de clasificación de carga-8 justificado.
3. Reejecutar toda la suite, matrices y calibrador analítico, comprobar ausencia de regresión R01/R02; emitir `R03_R04_BATCH_REPORT.md`.
4. **STOP, no R05 sin autorización.**

### Cierre — R05 (tercera autorización, etapa individual)

1. Resolver F17 y D01–D26 respetando historia y taxonomía.
2. Revisión científica independiente de claims/18 filas de capacidades y gates comprometidos.
3. Emitir addendum S30 y reporte de cierre integral; conservar externos abiertos. **STOP, S31 prohibido.**

## 9. Formato obligatorio de reporte por hallazgo y batch

```text
F-ID / severidad / sprint / estado:
Causa raíz constatada:
Commit(s) y diff relevantes:
Fixture auditor original + test de regresión añadido:
Resultado baseline S30 (esperado fallido):
Resultado tras corrección (esperado aprobado si resuelto):
Pruebas invariantes/analíticas/contrafactuales:
Masa/especies/energía/estado (según dominio):
Cambio de resultados numéricos y explicación:
Limitaciones todavía abiertas:
Revisión independiente y versión:
```

**Reporte batch:** fecha, base SHA, HEAD, branch/worktree, toolchain/OS, manifiestos, coverage nuevo, clasificación F-ID completa, logs, stdout/stderr, resultados `cargo test/fmt/clippy`, pruebas negativas, consecuencias sobre gates, `git status`, decisiones ADR, compromisos no resueltos y solicitud explícita del siguiente permiso.

## 10. Condición final de éxito

R01–R05 solo pueden declararse **remediados** cuando los **19 defectos confirmados** tengan evidencia de corrección independiente y las **26 derivas documentales** estén resueltas o explícitamente bloqueadas sin maquillaje, y no se hayan introducido regresiones. El objetivo **no** es conseguir 32/32 verdes mediante manipulación del checker; es restaurar los contratos físicos y de software que esos tests intentan falsar. La matriz de capacidades debe continuar restringida a aquello realmente verificado. Cualquier dato de campo ausente sigue ausente.
