# ADR-032 — S25 cierra NUMERICALLY_VERIFIED_ONLY (sin dyno inventado)

- **Fecha:** 2026-10-10 · **Estado:** ACEPTADO (S25, último del batch)
- **Decisiones:**
  1. Cero datos sintéticos como validación: el self-test (recuperación
     exacta) verifica el fitter y va rotulado SYNTHETIC_SELFTEST; el
     manifiesto prohíbe sintéticos en holdout por construcción.
  2. Cierre NUMERICALLY_VERIFIED_ONLY con 8 vacíos numerados (P1
     externo): dyno, p-θ, motored, MAF/MAP, inyectores, aceite, warmup,
     segundo motor.
  3. Lecciones metodológicas ya incorporadas al protocolo: ancla sellada
     C2 (degenerescencia C/p_ideal), holdout en motor distinto, plantilla
     de reporte por régimen sin accuracy universal.
  4. Bugs reales del sprint: lockfile offline, `max` en float ambiguo,
     Gauss-Newton sin backtracking (divergía), covarianza singular sin
     ancla (colinealidad real, no bug).
- **Trigger de reapertura:** C1–C6 + holdout M2 en el manifiesto.
- **Refs:** Foundation S25; EVIDENCE_POLICY; P1 (RISK_MATRIX).
