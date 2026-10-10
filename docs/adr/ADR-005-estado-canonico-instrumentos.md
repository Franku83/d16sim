# ADR-005 — Estado físico canónico, instrumentos como observadores
- **Fecha:** 2026-10-10 · **Estado:** ACEPTADO
- **Decisión:** ground truth separada de observación; sensores con ruido/sesgo/fallos/ancho de banda.
- **Motivo:** evita que el juego "sepa" y regale la avería.
- **Alternativas:** lectura directa de ground truth (descartada: destruye diagnóstico).
- **Consecuencias:** API `GroundTruthSnapshot` vs `ObservedState`; S17–S20.
- **Trigger:** si un instrumento futuro exigiera acceso omnisciente, reabrir con justificación.
- **Refs:** Foundation §1, §7, §10.1.
