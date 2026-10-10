# ADR-025 — ECU P2E-like con mapas y códigos no OEM

- **Fecha:** 2026-10-10 · **Estado:** ACEPTADO (S18)
- **Decisiones:**
  1. Decoder 24+1 con respaldo 4+1 (difiere de Speeduino monorueda con
     diente faltante: referencia conceptual, cero código GPL).
  2. Mapas y códigos `REF-*` de referencia etiquetados (P2E sin dumps).
     Latencia modelada como desplazamiento visible (solicitado vs efectivo).
  3. Sin sinc no hay actuación (fallback 360° documentado); DTC solo por
     persistencia temporal de monitores.
- **Trigger:** estrategia OEM documentada + datalogs reabren mapas y
  monitores; el scheduler S16/open-loop sigue vigente hasta S25.
- **Refs:** TRIG-2026-10-10; Foundation §7/S18; Speeduino como concepto.
