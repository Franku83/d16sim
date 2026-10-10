# ADR-031 — Instrumentos mecánicos puros + protocolo de dos mediciones

- **Fecha:** 2026-10-10 · **Estado:** ACEPTADO (S24)
- **Decisiones:**
  1. Instrumentos en `diagnostics::mech` como funciones puras (sin
     depender de `engine`/`leak`): el CLI cablea escenarios S23 a
     instrumentos; el reparto de fuga es observación del harness (oídos
     del técnico), como en S20.
  2. Cranking L1 y leak-down L1 con constantes ilustrativas declaradas
     (C_CRANK 8e5, orificio 1 mm²); dirección exacta, magnitud S25.
  3. Protocolo exige ≥ 2 mediciones independientes; umbrales
     (900 kPa, 10 %, 0.15 refrigerante) son ayudas de taller declaradas,
     NO física calibrada.
  4. Ambigüedad residual documentada: subapretada firma como anillos con
     2 tests (exige 3º: reapriete+retest o química); gap ×2 bajo el umbral
     de cranking (límite de detectabilidad honesto, no ajustado).
  5. Bug real: el cableado sumaba el respiradero al área de cámara
     (colapsaba el pico sano a 50 kPa); corregido filtrando from == 0.
- **Trigger:** calibración contra banco (S25), compresímetro real con
  curva, válvula de fuga (causa válvula modelada, hoy solo en tests).
- **Refs:** Foundation S24; FIELD-07/08; CF26.
