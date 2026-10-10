# ADR-039 — Presupuesto químico transitorio, procedencia de migración y fronteras atómicas

- **ID:** ADR-039
- **Fecha:** 2026-10-10
- **Estado:** ACEPTADO (remediación R01–R02-C1)
- **Decisión propuesta:**
  1. **Presupuesto químico (RV01):** la masa quemada de cada paso es la que
     corresponde a su calor liberado (`dm = dQ/(LHV·η)`); el inventario
     `burn_latched` es la masa comprometida de referencia y el remanente no
     quemado permanece en `c.fuel` como especie transportable. No se suma calor
     liberado y combustible intacto como energías independientes.
  2. **Observación diagnóstica (RV02):** toda ruta de gas instrumentada desde la
     cámara entra en el denominador físico; un reparto sin fracciones observadas
     no es una medición y no puede discriminar causa.
  3. **Cero firmado (RV03):** se preserva el bit de signo de `-0.0` en los
     campos opcionales (solo se omite el cero positivo). La identidad de bits de
     estados válidos se mantiene y los snapshots previos conservan su hash.
  4. **Procedencia de migración (RV04):** un snapshot legacy se migra con
     registro de esquema/hash de origen, transformación, campos ausentes y
     alcance (`exact_neutral` o `approximate_hot`). Integridad de lectura no es
     equivalencia de continuación física.
  5. **Atomicidad fail-closed (RV05–RV09):** las APIs públicas validan dominio y
     finitud antes de mutar; el avance de engine se confirma sobre una copia;
     errores de fricción no se convierten en torque cero; decoder/ECU no
     propagan NaN/Inf; el fitter valida la dimensión del callback; el dyno
     rechaza factores no finitos.
- **Contexto/motivo:** verificación independiente R01–R02
  (`verification/r01-r02-2026-10-10`, `REMEDIATION_REQUIRED_BEFORE_R03_R04`)
  confirmó RV01–RV09: 1156 J liberados con `fuel_burned=0`, escape
  instrumentado descartado, cero firmado perdido, migración sin procedencia y
  fronteras que mutaban antes de rechazar.
- **Alternativas consideradas:** (a) mantener contabilidad diferida del quemado
  y declarar un inventario «pendiente» — rechazada: duplica el inventario
  químico; (b) descartar el escape del denominador — rechazada: pierde masa
  física; (c) canonicalizar `-0.0` a `+0.0` — rechazada: rompe la promesa de
  bits; (d) rechazar la continuación legacy — rechazada por la spec: se expone
  como aproximada y etiquetada.
- **Consecuencias:** Δ numérico mínimo por orden de operaciones en la ventana de
  combustión (~1e-5 relativo en ralentí nominal); no se reajustaron parámetros.
  Los snapshots nativos conservan continuidad exacta; los legacy quedan
  explícitamente aproximados.
- **Trigger de revisión:** nueva evidencia experimental (S25), cambio de modelo
  de combustión/fricción o migración de esquema incompatible.
- **Referencias:** FINDINGS RV01–RV09; `BATCH_GATE_DECISION.md`;
  `docs/remediation/R01_R02_C1_BATCH_REPORT.md`; ADR-038.
