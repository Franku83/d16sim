# D16Y7 — Informe de calibración (S25)

- **Fecha:** 2026-10-10 · **Variante:** `US_1997_CIVIC_DX_2DR_5MT_OBD2A_P2E_FAMILY`
- **Estado: NUMERICALLY_VERIFIED_ONLY — NO VALIDADO.** Sin dataset de
  campo (P1 externo) no existe comparación real/simulado que reportar.
  Este informe documenta la maquinaria lista, los priors, los vacíos y
  el protocolo que se ejecutará cuando lleguen los datos. **No contiene
  ningún dato de dyno** (inventarlo violaría el gate S25).

## 1. Maquinaria (verificada, no validada)

- Fitter Gauss-Newton con escalado + backtracking + covarianza
  `(JᵀWJ)⁻¹`: recupera C y p_ideal exactos desde picos
  SYNTHETIC_SELFTEST (self-consistency rotulada, `calibration-demo`).
- Lección registrada: sin punto de ancla casi sellada, C y p_ideal son
  casi colineales (corr −0.98) y la covarianza es singular — el protocolo
  C2 incluye la referencia sellada por este motivo.
- Sensibilidad local (elasticidades) + global (Morris determinista):
  en cranking nominal el área de fuga domina sobre el volumen muerto.
- Manifiesto train/holdout con anti-leakage (motor distinto, reales con
  procedencia, sintéticos prohibidos en holdout): 4 tests de reglas.

## 2. Parámetros (6 identificables con taller, 8 bloqueados)

Ver `calibration_set()` en `crates/calibration/src/params.rs` + priors.
Bloqueados hasta: p-θ/torque vs CA (Wiebe), motored (fricción),
MAF/MAP dyno (vol_eff), banco de inyectores, mapa P2E medido.

## 3. Intervalos, residuos y percentiles por régimen

SIN DATOS: sin objeto. Plantilla comprometida para el cierre con datos:
residuos por punto operativo, covarianza + correlación máxima del fit,
percentiles 5/50/95 por régimen y condición, gráficos real-vs-simulado
con train/holdout distinguidos. **Sin accuracy universal**: la precisión
se reporta por régimen/condición o no se reporta.

## 4. Vacíos (los 8 bloqueantes)

1. Dyno torque vs RPM + MAP/lambda/consumo (C1) — P1.
2. p(θ) indicado (C7) — P1. 3. Motored torque (fricción) — P1.
4. MAF/MAP mapeado (vol_eff) — P1. 5. Banco de inyectores (C6) — P1.
6. Presión de aceite vs T/rpm (C4) — taller. 7. Warmup instrumentado
   (C5) — taller. 8. Segundo motor D16Y7 para holdout — P1.

## 5. Decisión de cierre

S25 se cierra como NUMERICALLY_VERIFIED_ONLY. Reabrir exige C1–C6 en el
manifiesto + holdout en motor distinto. Hasta entonces, rige el bloqueo
de marketing de precisión (model cards).
