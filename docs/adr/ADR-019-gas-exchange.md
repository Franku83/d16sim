# ADR-019 — Gas exchange 0D con Cd medido y sin VE

- **Fecha:** 2026-10-10 · **Estado:** ACEPTADO (S12)
- **Decisiones:**
  1. Válvula = orificio compresible con `Cd(L)·A_cortina` (Cd medido por
     válvula, research v0.1/donantes); SIN factor min(cortina, garganta):
     sería doble conteo (§24.4). La limitación de puerto vive en la caída
     del Cd; la garganta explícita es deuda.
  2. Dos especies (fresca/residual) con transporte en reflujos; el residual
     quemado neto exige combustión (S16) — en arrastrado es ≈ 0 por física,
     no por imposición.
  3. Ciclo de primer ciclo desde PMI con nodos a ambiente (transitorio
     documentado); la convergencia periódica es S16. Euler explícito con
     cierre de masa exacto por construcción y energía auditada.
  4. `lift_at` envuelve a (−360°, 360°] (bug S12: se perdía la apertura a
     caballo del 0°/720°).
- **Trigger:** tabla lift-vs-CA medida, Cd con incertidumbre de banco,
  geometría de puertos/colector y red 1D reabren con datos.
- **Refs:** Research v0.1 §1, donantes §2; Foundation §6.5.
