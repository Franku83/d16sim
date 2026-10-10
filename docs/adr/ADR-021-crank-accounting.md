# ADR-021 — Contabilidad de torques sin doble conteo + J media

- **Fecha:** 2026-10-10 · **Estado:** ACEPTADO (S14)
- **Decisiones:**
  1. Indicado = ciclo COMPLETO `Σ∮p·dV` (incluye bombeo); fricción y carga
     separadas. Solo-circuito-cerrado + bombeo explícito es la alternativa
     documentada, nunca sumada dos veces.
  2. `J_eff = J_rot + m·⟨(dx/dθ)²⟩` (equivalencia energética media);
     `J(θ)` variable real como deuda con costura (`InertiaParams` ya
     separa topología de masa).
  3. Chen–Flynn con `c2 = 0` por defecto (Pipitone: término de carga débil
     en SI); coeficientes del llamante, sin calibración D16Y7.
  4. Rizo intra-revolución por encima de ω₀ es volante legítimo, no energía
     gratis: la prueba es la auditoría, no una cota instantánea.
- **Trigger:** FMEP medido / J(θ) / cargas accesorias reabren con datos.
- **Refs:** Foundation §6.2; Pipitone SAE 2009-01-1984 (vía research).
