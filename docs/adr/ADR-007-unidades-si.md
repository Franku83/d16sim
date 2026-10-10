# ADR-007 — Unidades SI internas y tipos dimensionales
- **Fecha:** 2026-10-10 · **Estado:** ACEPTADO (implementación S03)
- **Decisión:** SI interno + tipos; UI con unidades seleccionables.
- **Motivo:** previene errores psi/Pa, °F/K, N·m/lb·ft.
- **Alternativas:** floats crudos (rechazado).
- **Consecuencias:** S03 tipa Pa_abs/Pa_gauge/Δp, °BTDC/°CA, cc/min@ΔPref; property tests.
- **Trigger:** bug dimensional en revisión obliga a ampliar tipos.
- **Refs:** Foundation §1, §12.3; research §24.4 guardrails.
