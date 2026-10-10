# ADR-028 — Red térmica L1 opt-in + fricción al aceite

- **Fecha:** 2026-10-10 · **Estado:** ACEPTADO (S21)
- **Decisiones:**
  1. Red (metal/pistón/aceite/refrigerante) opt-in: `thermal: None` conserva
     S16 bit-idéntico (verificado por regresión); `Some` reparte quemado
     (fw) a paredes y fricción al aceite.
  2. Auditoría ramificada: sin red la fricción es sumidero (S16 exacto);
     con red es transferencia interna al aceite. Ambas cierran.
  3. Termostato como factor 0–1 en banda genérica 80–88 °C; ECT con lag
     que alimenta enriquecimiento (cadena L1 de S19).
  4. Snapshot esquema 2 con `#[serde(default)]`: v1 migra (red a frío o
     ausente según entradas).
- **Trigger:** hA/C medidos, curva de termostato/bomba, ECT real con S19
  completo, Stribeck/viscosidad (S22).
- **Refs:** Foundation S21; H-S19-04 (cadena).
