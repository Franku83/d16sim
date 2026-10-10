# ADR-014 — Convenciones cinemáticas + política de geometría ilustrativa

- **Fecha:** 2026-10-10 · **Estado:** ACEPTADO (S06)
- **Decisiones:**
  1. `θ = 0` en PMS; `x` desde PMS; ciclo 720° con PMS de encendido en
     local 360°; orden ilustrativo 1-3-4-2 con compañeras (1,4)/(2,3).
  2. Régimen estacionario en S06 (`dω/dt = 0`); el término transitorio llega
     con la dinámica S14.
  3. El preset `illustrative_d16_like` usa candidatos PENDING de S01 y está
     **prohibido** como nominal OEM: solo verifica matemática contra
     soluciones analíticas. Todo resultado numérico S06–S10 con este preset
     es verificación (`VERIFIED_NUMERICALLY`), nunca validación.
- **Alternativas:** θ desde PMI (rechazado: rompe convención de avance);
  fijar geometría como nominal (rechazado: S01 pendiente).
- **Trigger:** auditoría OEM de S01-cierre o medición directa sustituye el
  preset (versionar + regresión sin cambiar API).
- **Refs:** Foundation §6.1; S01 `D16Y7_DATA_DICTIONARY.md`.
