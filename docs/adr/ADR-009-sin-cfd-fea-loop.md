# ADR-009 — Sin CFD/FEA 3D completo en el loop interactivo
- **Fecha:** 2026-10-10 · **Estado:** ACEPTADO
- **Decisión:** solvers externos/offline para mapas sustitutos verificados.
- **Motivo:** costo y falta de parámetros/materiales.
- **Alternativas:** CFD propio (rechazado, §19.3).
- **Consecuencias:** L4 solo offline; surrogates versionados.
- **Trigger:** disponibilidad de geometría+materiales+validación reabre alcance, no loop.
- **Refs:** Foundation §1, §3.1.
