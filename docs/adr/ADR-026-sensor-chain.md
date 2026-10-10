# ADR-026 — Cadena sensorial separada del ground truth + monitores

- **Fecha:** 2026-10-10 · **Estado:** ACEPTADO (S19)
- **Decisiones:**
  1. Cadena físico→transductor→cable→ADC→filtro con tipos disjuntos del
     ground truth termodinámico (separación por construcción).
  2. Transferencias ilustrativas (MAP lineal, NTC con pull-up, O2
     conmutado); tabulación OEM pendiente. Ruido determinista por seed.
  3. Fallos por estado (R/abierto/cortos/intermitente con seed).
     Monitor de rango genérico REF-* con persistencia/curación.
  4. Acción ECU (enriquecimiento/carga) vive en funciones testeables que
     no nombran termodinámica.
- **Trigger:** curvas/tablas OEM, IACV PWM, O2 con heater/retardo,
  closed-loop (S20+).
- **Refs:** Foundation §7.3/S19; casos E01–E12 del research.
