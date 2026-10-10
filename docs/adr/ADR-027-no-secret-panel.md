# ADR-027 — Instrumentos sin acceso al ground truth

- **Fecha:** 2026-10-10 · **Estado:** ACEPTADO (S20)
- **Decisiones:**
  1. `diagnostics` no depende de `engine`/`assembly` (por construcción);
     el scanner solo contiene lo entregado; el manual son enlaces.
  2. Multímetro con carga (Zin), rango y guarda de alimentado; scope con
     trigger/calibración y aliasing declarado (no oculto).
  3. Ambigüedad por diseño: sin medición complementaria el veredicto es
     `Inconclusive`; la verdad del harness solo monta escenarios.
- **Trigger:** sondas de presión/temperatura reales, leak-down, compresí-
  metro y bore-gauge llegan con S24 (instrumentos mecánicos).
- **Refs:** Foundation §7/S20; casos E/M del research.
