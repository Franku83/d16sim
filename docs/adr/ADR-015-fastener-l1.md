# ADR-015 — Precarga L1 + costura SealConductance + secuencia sin efecto

- **Fecha:** 2026-10-10 · **Estado:** ACEPTADO (S08)
- **Decisiones:**
  1. Precarga por `T = K·F·d` con `K` explícito o presets ilustrativos de
     literatura genérica (NO Honda, NO ARP trasladado). Paso solo por
     ángulo: `MethodNotSupported` sin curva clamp-load.
  2. `SealConductance::Unsupported` como costura: S23 implementará la fuga
     desde `SealContactState` sin cambiar la API de la junta.
  3. La secuencia de apriete se registra pero no altera el campo L1: sin
     modelo de interacción elástica, cualquier efecto sería un multiplicador
     oculto (prohibido por §5.3 del Foundation).
- **Trigger:** medición de `K`/curva clamp-load o modelo elástico reabre
  con datos; S23 conecta la conductancia.
- **Refs:** Foundation §5.3, §8; ARP/Fel-Pro como principios generales.
