# ADR-016 — Integrador RK4 angular + auditoría como gate numérico

- **Fecha:** 2026-10-10 · **Estado:** ACEPTADO (S09)
- **Decisiones:**
  1. Integración por ángulo de cigüeñal con RK4 de paso fijo + Simpson
     para el trabajo (mismo orden; trapecios degradarían a O(h²)).
  2. Toda feature con solver entra con referencia analítica (o cruzada) y
     estudio de convergencia; la tolerancia se justifica por medición, no
     al revés.
  3. `ThermoState` con `p = mRT/V` por construcción (imposible imponer
     `p ≠ mRT/V` silencioso).
- **Trigger:** rigidez (tiempos químicos) o DAE en S10+ reabre con método
  implícito; la API de auditoría no cambia.
- **Refs:** Foundation §9.3–§9.4; ASME VVUQ (verificación).
