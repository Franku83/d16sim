# Decisión independiente del gate R01–R02

**`REMEDIATION_REQUIRED_BEFORE_R03_R04`**

Objetivo revisado: `8a7e8686e4e8831fb30d1a71b815e67e7d7139ef`, 2026-10-10. La revisión está terminada; el batch no obtiene aprobación independiente. No se inician R03/R04/R05/S31 ni se fusiona nada.

| Condición de autorización | Evaluación |
|---|---|
| F01 VERIFIED_FIXED | **No:** advección corregida, energía química/consumo temporal no cerrado (RV01) |
| F02 VERIFIED_FIXED | **No:** etiqueta desacoplada, pero escape/información incompleta mal observados (RV02) |
| F05–F10/F15 verificados o residual que no comprometa contrato | **No:** F05 bits, F06 alcance/procedencia y F15 atomicidad/dominio comprometen contratos exigidos. F07/F08/F09/F10 pasan en dominios declarados |
| Sin nuevas regresiones P0 | No se demuestra regresión P0 del transporte; existe defecto nuevo del helper observacional. **No puede certificarse ausencia universal**, ni esta condición suplir F01/F02 |
| Originales reproducibles | **Sí:** 163/0→183/0, 23/9→10/22 en 32 casos |
| Integridad de harness | **Sí:** checker byte-idéntico, adaptación neutral exacta, originales preservados, repro desde Git |
| Diez restantes atribuidos correctamente | **Sí:** casos 17/18/19/20/21/22/23/28/31/32 son R03/R04. Nuevos casos rojos son distintos y detectan residuales R01/R02 |
| Riesgos/limitaciones declarados | **Sí:** dominios, host único, técnicas de extracción, ledger por tramo, migration legacy, K01–K03/G01–G07 |

## Trabajo necesario antes de reconsiderar el gate

1. **F01:** cerrar el presupuesto de reacción durante la ventana, con inventario no quemado/quemado/Q coherentes; revisar latch e interrupciones. Pasar el control donde Q≈1156 J pero quemado=0; verificar transporte y replay sin alterar parámetros nominales ni tolerancias para disimularlo.
2. **F02:** contabilizar escape instrumentado en denominador/reparto físico y modelar disponibilidad de canales. Ausencia de información no debe generar un reparto inventado ni una causa discriminada. Separar pruebas sintéticas del clasificador de los demos basados en red y mantener independencia de etiquetas. Reevaluar AUD-R01 en un dominio acotado de ambigüedad.
3. **F05:** preservar el signo de cero o precisar/rechazar/canonicalizar ese dominio antes de prometer bits universales. La pérdida decimal original ya está corregida; no se exige reescribir la solución ni preservar valores numéricos defectuosos.
4. **F06:** conservar procedencia de migración y definir compatibilidad de lectura/hash separada de continuación física. Campos ausentes no deben declararse historia conocida; limitar explícitamente snapshots calientes legacy, devolver estado aproximado rotulado o rechazar la continuación que requiera información inexistente. Probar con productor/tipo históricos reales. No exigir igualdad entre ecuaciones viejas defectuosas y corregidas.
5. **F15:** completar validación por API y avance atómico: input/state/historia engine, Kelvin/ECT y thermal candidato, restore+inputs, decoder/update/estimate y scheduler, salidas/dimensiones del fitter, salida finita dyno. Rechazos antes de mutar, sin NaN como torque cero ni panic no tipado. Mantener aparte F13/F18/F19 para el siguiente batch.

No son solicitudes para implementar estas correcciones dentro de la verificación. Son criterios concretos para una futura fase autorizada. Las correcciones de F07/F08/F09/F10 deben conservarse con sus regresiones de presión, mezcla, unidades y calor ambiente.

No bloquean el gate por sí solos: K01–K03, G01–G07, no disponer de densidad configurable ni un segundo host, los diez casos originales fuera del batch, y la ausencia de integración experimental D16Y7. Sí impide un PASS convertir un contrato crítico inconcluso en satisfecho.

La evidencia sostiene cuatro correcciones verificadas y cinco parciales, no un simulador predictivo ni una validación experimental. Los gates históricos quedan intactos; se recomienda reevaluar requisitos concretos con [FINDINGS.md](FINDINGS.md) y [REPRODUCTION.md](REPRODUCTION.md).
