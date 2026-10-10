# ADR-022 — Arranque arrastrado sin bootstrap + acoplo V

- **Fecha:** 2026-10-10 · **Estado:** ACEPTADO (S15)
- **Decisiones:**
  1. `started = false` por diseño en S15 (ni con combustible y chispa
     presentes): el arranque con combustión es S16. Ningún bootstrap a RPM.
  2. Acoplo L1 par–tensión: τ aplicado = τ_nominal·(V/V0) con V en bornes
     bajo carga (bug real: sin acoplo la batería muerta giraba igual).
     La eficiencia es diagnóstico (≤ 1 testeado), no identidad.
  3. Reductora 12:1 ilustrativa explícita (sin ella el par no vence la
     compresión: detectado por diseño, no por tuning).
  4. Chispa = disponibilidad por dwell×V (E = ½L(Vt/L)² ≥ umbral); el
     CUÁNDO lo programa S18. Sin DTC en S15 (monitores en S18).
- **Trigger:** curvas τ/I/V medidas del starter, Rint(SOC,T) medida,
  compresión a rpm de arranque con gas exchange vivo (S16).
- **Refs:** Foundation §6.2/S15; traza S12 como carga resistiva.
