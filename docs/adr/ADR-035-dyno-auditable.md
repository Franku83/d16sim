# ADR-035 — Dyno que rechaza puntos no estabilizados

- **Fecha:** 2026-10-10 · **Estado:** ACEPTADO (S28)
- **Decisiones:**
  1. Potencia solo desde τ×ω integrado (indicada/fricción de la traza,
     freno = carga declarada × ω); corrección ambiente con observado +
     corregido; validez Covered/Extrapolated/Unsettled en el manifiesto.
  2. Hallazgo real: un punto (carga 8) no estabiliza la cola (residuo
     60 %) aunque el cierre global es 1e-5 — la media de cola con
     muestreo 20 ms no representa el ciclo (aliasing/inestabilidad
     candidata, material S29). El dyno lo RECHAZA (Unsettled), no se
     ajustó ninguna constante para hacerlo pasar.
  3. Identidad estricta por auditoría (< 1 % con ΔKE); test de mezcla de
     ventanas eliminado por incorrecto (cola vs global).
- **Trigger:** dyno real (C1), corrección certificada, inestabilidad S29.
- **Refs:** Foundation S28; R17; FIELD-20.
