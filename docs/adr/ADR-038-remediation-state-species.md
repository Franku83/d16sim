# ADR-038 — Estado de especies, historia de presión y fronteras fail-closed

- **ID:** ADR-038
- **Fecha:** 2026-10-10
- **Estado:** ACEPTADO (remediación R01–R02)
- **Decisión propuesta:**
  1. El estado del motor representa explícitamente el combustible como
     especie transportable en cilindros **y** nodos (plenum/escape), con
     advección homogénea en cada flujo y contabilidad de frontera
     (`fuel_emitted_kg`). El presupuesto químico es
     `inyectado = almacenado + quemado + emitido`.
  2. La historia física que modifica fuerzas se persiste: `pmax_pa`
     (máximo de presión de cilindro para Chen–Flynn) entra en el snapshot.
  3. La verificación de integridad se hace **por esquema**: el hash v1 se
     reconstruye sin `thermal` antes de migrar (R02-F06); el parseo de f64 es
     lossless (R02-F05).
  4. Los campos nuevos son opcionales y se omiten cuando valen su valor
     neutro, preservando los hashes v1/v2 previos sin bump de esquema.
  5. Las fronteras públicas validan dominio **antes** de mutar (fail-closed):
     masas/temperaturas positivas y finitas, capacidades > 0, fracciones en
     [0,1], combustible en [0, m], entradas de instrumentos finitas y
     cronológicamente válidas.
  6. La observación diagnóstica se deriva de flujos/nodos físicos, nunca del
     identificador de escenario (R01-F02).
- **Contexto/motivo:** la auditoría independiente (`audit/independent-2026-10-10`,
  commit `25d528e`) confirmó F01/F02/F05–F10/F15: cierre de masa con especies
  imposibles, observación dependiente de la verdad, pérdida de bits JSON,
  hash v1 rechazado, presión de fricción no persistida, especie fresca no
  advectada, volumen tratado como masa, calor al ambiente omitido y fronteras
  sin validación.
- **Alternativas consideradas:** (a) cambiar la forma de onda de fricción para
  evitar persistir presión — rechazada: altera el modelo; (b) guardar nodos
  sin combustible y descartar el reflujo — rechazada: pierde inventario
  químico; (c) clamp tardío de `m_fuel` — rechazada por la spec (no sustituye
  advección); (d) bump de esquema a v3 — innecesario con campos opcionales
  retrocompatibles.
- **Consecuencias:** cambian cifras nominales de RPM/trabajo (el combustible
  que sale/refluye y la presión de fricción dejan de reiniciarse). No se
  reajustaron parámetros para conservarlas. Los snapshots previos siguen
  válidos. Se documenta en `snapshot-contract.md`, `integrated_engine.md`,
  `leaks.md`, `thermal.md` y `diagnostic_tools.md`.
- **Trigger de revisión:** nueva evidencia experimental (S25), cambio de
  modelo de fricción/combustión, o migración de esquema incompatible.
- **Referencias:** FINDINGS F01/F02/F05–F10/F15; `D16SIM_REMEDIATION_R01_R05_SPEC_v1.0.md`;
  `docs/remediation/R01_R02_BATCH_REPORT.md`.
