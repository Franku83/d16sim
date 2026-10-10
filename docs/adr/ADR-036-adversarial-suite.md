# ADR-036 — Adversarial sin gobierno de fallos sobre la física

- **Fecha:** 2026-10-10 · **Estado:** ACEPTADO (S29)
- **Decisiones:**
  1. Fallos = deltas puros de `EngineInputs` (10 variantes); no existe
     `fault_count` ni flag que altere la física (estructural, no promesa).
  2. Interacción cuantificada, no recetada (superaditiva +132 en
     restricción+pobre; enmascaramiento NoSpark→motorado).
  3. Correcciones honestas: starter motora sin chispa (347 rpm, no 0);
     hash canónico ordenado (el orden no identifica el caso).
  4. Punto carga-8/S28 heredado como candidato a inestabilidad (deuda de
     investigación, no veredicto).
  5. Par junta+cooling parcial (blow-by motor pendiente); Monte Carlo
     xorshift con seed (N=16 tests, N=32 demo).
- **Trigger:** acople blow-by S23–motor, inestabilidad carga-8, C1.
- **Refs:** Foundation S29; FIELD-01/16; S24 (observabilidad).
