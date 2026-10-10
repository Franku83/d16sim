# Regresión y atribución del batch

Objetivo `8a7e8686e4e8831fb30d1a71b815e67e7d7139ef`, contra base `ff09a0d`. Los cambios de remediación son 20 archivos; el diff íntegro se conserva. No se modifica producción durante esta revisión.

## Resultados transversales

| Área | Comparación independiente | Conclusión limitada |
|---|---|---|
| Arranque/ralentí | Negativos originales y barrido de paso/horizonte; 8 s dt50 µs, idle original **1025.817725 RPM**, corregido **1047.317273 RPM** | Cambio reproducido; no exigir conservar valores que dependían de defectos. No demuestra estabilidad universal ni validación real |
| Combustión/trabajo | Q de **46373.1245→46146.8578 J**; trabajo indicado **13333.5355→13286.5361 J** en ese mismo fixture | Cambio causal tras especies; permanece RV01 de química dentro de ventana |
| Masa y combustible | 12 nuevos controles engine y 9 de reflujo, dt100/50/25 µs; fuente inicial marcada y controles negativos | Cierre de combustible ≤2.4e-18 kg y total ≤1.1e-17 kg en esos experimentos; fracciones acotadas |
| Composición fresca | Invariante cerrado, mezcla receptora, suministro/venteo, puertos en ambos sentidos | F08 verificado numéricamente; red dirigida no es una tubería implícitamente bidireccional |
| Estabilidad/refinamiento | Se repiten los originales hasta 30 s y thermal 120 s; refinamiento térmico contra referencia exponencial | Evidencia acotada, no cierre G02. No se cambian umbrales ni parámetros para converger |
| Snapshots/replay | 36 cortes/horizontes + 18 trayectorias de tres cortes; c2=0/.01/.05; thermal y controles de combustible | Nativo completo: estado/hash y todas las muestras coinciden; legacy requiere RV04; -0.0 requiere RV03 |
| Diagnóstico | Cuatro demos reales, extracción exacta y red con escape instrumentado | El acceso por etiqueta desaparece; escape/observación incompleta fallan RV02 |
| Auditoría térmica | Signos positivo/negativo/nulo, adiabático, termostato, global/local y refinamiento | F10 corregido para corrida ininterrumpida; atomicidad/temperaturas inválidas siguen RV06 |
| Determinismo | Repetición de los cinco modos nuevos | JSONL byte-idéntico en un host; no se extrapola a otros SO/CPU/toolchains |
| Consumidores de estado ampliado | workspace test/fmt/clippy; harness original; serialize/restore/resume; CLI, dyno, wear | Integración de build conservada y 183 casos verdes; no equivale a cumplimiento físico de todas las fronteras |

El reproductor compara `base_inputs()` byte a byte entre S30 y remediación y falla si difiere. No hay cambios en los crates de parámetros físicos nominales. La adaptación del fixture tampoco modifica geometría, temperaturas, mapas, coeficientes, controles o entradas de caudal. El nuevo régimen procede de los cambios de transporte/estado y de las ecuaciones, no de un ajuste para preservar resultados. Persistir pmax cambia legítimamente escenarios con c2≠0; c2 usados en probes son controles numéricos ilustrativos, no calibración del D16Y7.

## Diez fallos originales correctamente fuera de alcance

| Casos originales | F-ID | Batch de spec | Contrato incumplido conservado |
|---|---|---|---|
| 17 | F03 | R04 | Calidad dyno depende del muestreo de torque |
| 18 | F04 | R04 | Canales de potencia usan ventanas distintas |
| 19 | F11 | R03 | Desgaste UNSUPPORTED produce evolución |
| 20 | F12 | R03 | Creep se revierte en frío |
| 21–22 | F13 | R03 | Recuperación/origen de fase decoder |
| 23 | F14 | R03 | Semilla de wiring no gobierna secuencia |
| 28 | F16 | R03 | Centerline+720 no conserva eventos de inyección |
| 31 | F18 | R03 | Covarianza en coordenadas escaladas |
| 32 | F19 | R03 | Óptimo inicial exacto rechazado |

La lista coincide con el reporte. No se mezclan RV07 (NaN/Inf de entrada) con F13 (semántica de fase), RV08 (dimensiones/panic) con F18/F19 (optimización), ni RV09 (overflow de frontera) con F03/F04 (integración/muestreo). Esos son contratos distintos aunque compartan crate.

## Defectos residuales y nuevos

- **Residuales de R01/R02:** RV01 (consumo diferido), RV05–RV09 (fronteras incompletas), clasificación de reparto vacío de RV02. Los casos originales que motivaron la corrección ya pasan, pero el contrato exigido es más amplio.
- **Defectos nuevos en implementación de R01/R02:** el nuevo helper omite escape que él mismo marca instrumentado (RV02); la nueva omisión opcional pierde el signo de cero (RV03). No se extrapola a un empeoramiento cuantificado del motor nominal; tampoco se declara una comparación A/B de topologías que el CLI antiguo no exponía.
- **Migración parcialmente demostrada:** RV04 acepta hash verdadero y ejecuta continuación, pero no acredita reconstrucción de historia ausente ni documenta procedencia/límites. El control v2 de pérdida de historia no se presenta como v1 auténtico.
- **Sin nueva regresión P0 de transporte demostrada** en los controles. Queda comprometido un requisito P0 de reacción/observación; esto basta para rechazar el gate sin inventar una regresión de masa.

## Restricciones vigentes

K01: no existe integración runtime completa de fuga/lubricación/wear/ECU con motor; ejecutar sus crates no la añade. K02: combustión/llenado L1 sin AFR/oxígeno calibrados; no autoriza presupuestos químicos incoherentes. K03: contacto/interferencia no predicen fractura ni vida útil.

G01–G07 siguen siendo gaps: datos experimentales, estabilidad general, revisión externa/segundo host, cadenas físicas completas, identificabilidad, electrónica genérica y metrología/manifiestos. Esta revisión verifica software/modelos en controles numéricos; no adquiere ni ajusta datos del motor.

Los ledgers de ambiente/masa se rebases por tramo en resume; `th_q_amb_j` se reinicia en restore, mientras los contadores de calor/trabajo físico persisten. El replay ensayado es del estado dinámico y trazas, **no del historial contable completo de fronteras**. No concatenar esos ledgers como si cubrieran toda la corrida. Esta restricción debe acompañar su uso, sin confundirla con el defecto pmax ya corregido.

Los targets/tests preexistentes mantienen nombres y expectativas en el diff revisado; la ejecución S30 y remediación confirma 163/183. No se relajaron oráculos del auditor para hacer pasar R01/R02. Se conserva el informe original y se propone reevaluar solamente requisitos identificados: S16/S30 especies/energía, S24 observabilidad, S30 persistencia/migración y DoD de fronteras numéricas. No se degradan ni editan gates históricos automáticamente.
