# ADR-017 — Combustión Wiebe de zona única, adiabática, sin calibrar

- **Fecha:** 2026-10-10 · **Estado:** ACEPTADO (S10)
- **Decisiones:**
  1. Wiebe paramétrico (a/m/inicio/duración explícitos) + primera ley de
     zona única con `Q_pared = 0`; calor contado una sola vez vía `dx/dθ`.
  2. Auditoría de calor por Gauss-Legendre por tramos: la tasa salta en el
     borde de ventana y Simpson hereda O(h·salto) ~ 1e-4 (analizado, no
     bug del solver).
  3. `WallHeat::Adiabatic` como única variante; la L1 de pared es costura
     futura. Sin `knock_predictor` (UNSUPPORTED hasta validación).
  4. IMEP reportado como ilustrativo (14.1 bar con entradas ideales supera
     la cordura familiar 11.5–13.0 en la dirección esperada: sin
     residuales/pérdidas; NO es validación ni calibración).
- **Trigger:** p(θ) medida o coeficientes γ(T)/pared reabren con datos.
- **Refs:** Foundation §6.4; priors Wiebe/Woschni del research.
