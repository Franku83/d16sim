# ADR-013 — Trazas JSONL + SHA-256 en S02 (Parquet diferido)

- **Fecha:** 2026-10-10 · **Estado:** ACEPTADO (S02)
- **Decisión:** trazas `traces.jsonl` (una fila JSON canónica por línea) +
  `metrics.json` + `hashes.json` con SHA-256; Parquet diferido a cuando haya
  telemetría numérica real que justifique dependencia pesada.
- **Motivo:** JSONL es inspeccionable, dif-friendly, sin dependencias
  nativas; suficiente para plumbing + replay de S02/S05.
- **Alternativas:** Parquet ya (rechazado: coste sin beneficio con
  autocontador); CSV (rechazado: tipos anidados pobres).
- **Consecuencias:** `simulation::runner` fija LCG + JSON canónico;
  cambiar serializador invalida hashes (documentado en código).
- **Trigger:** volumen de telemetría (S14+) que degrade I/O reabre con
  benchmark.
- **Refs:** Foundation §10, §12.1; S02.
