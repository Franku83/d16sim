# ADR-004 — Ángulo de cigüeñal como coordenada primaria + tiempo físico
- **Fecha:** 2026-10-10 · **Estado:** ACEPTADO (diseño; implementación S06/S09)
- **Decisión:** `theta_crank` continuo global + `theta720`; tiempo físico para transitorios.
- **Motivo:** sincroniza volumen, distribución, chispa y cilindros.
- **Alternativas:** solo tiempo (pierde sincronía de ciclo); solo ángulo (singular cerca de ω=0).
- **Consecuencias:** integrador en tiempo para arranque/parada; mapeo y localización de eventos.
- **Trigger:** inestabilidad numérica documentada en S06/S09 obliga a revisar esquema.
- **Refs:** Foundation §6.1, §9.1.
