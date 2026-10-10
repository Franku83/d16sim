# ADR-002 — Core headless Rust (C++ alternativa si un solver lo justifica)
- **Fecha:** 2026-10-10 · **Estado:** ACEPTADO (principio; sin código en S00)
- **Decisión:** núcleo headless en Rust; C++ solo si un solver lo justifica.
- **Motivo:** tipos, memoria, pruebas, compilación nativa.
- **Alternativas:** C++ directo; Python científico (descartado para core por tipos/rendimiento).
- **Consecuencias:** S02 fija toolchain/lints; benchmark antes de comprometer WASM.
- **Trigger:** benchmark S02/S30 que demuestre bloqueo; propuesta de solver con binding justificado.
- **Refs:** Foundation §1, §10.
