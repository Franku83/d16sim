# Campañas de adquisición — S25-T1 (protocolo, sin datos aún)

Cada ensayo registra: fecha/operador/banco/condiciones ambiente
(p/T/HR), combustible (lote, RON), aceite (grado, horas), ECT/IAT,
régimen estabilizado ≥ 30 s y procedencia completa. Sin esto, el dato
NO entra al manifiesto (fail-closed).

| ID | Campaña | Observables | Condiciones | Desbloquea |
|---|---|---|---|---|
| C1 | Torque vs RPM (dyno chasis/motor) | torque, rpm, MAP, lambda, ECT, consumo | WOT + 3 cargas parciales, 1000–6500 rpm, ECT 85–90 °C | wiebe, vol_eff, friction |
| C2 | Arranque/cranking | pico compresión/cil (WOT + cerrada), rpm arrastre, Vbat | ECT ambiente + caliente, 2 compresímetros (FIELD-08) | crank_c_leak, crank_p_ideal |
| C3 | Leak-down + escucha | % fuga, reparto, posición declarada | TDC por cilindro, 500 kPa regulados | p_seal_ref, ring_cd |
| C4 | Aceite | p_galería vs rpm+T, nivel, grado | 1000–5000 rpm, 40–130 °C | pump_disp, k_film |
| C5 | Warmup térmico | ECT/IAT/T_aceite vs t | ralentí 20 °C → 90 °C, sin carga | wall_frac, hA |
| C6 | Inyectores (banco) | Q estático/dinámico, deadtime vs V | 3 pulsos, 8–14 V | injector_Q |
| C7 | P-V indicado (opcional) | p(θ) por cilindro | 2–3 puntos C1 | wiebe_m/dur, IMEP |

Split: train = motor M1 campañas C1–C6; holdout = motor M2 DISTINTO
(mismo código D16Y7) con C1+C3 como mínimo. Mismo motor nunca en ambos
(leakage). Sintéticos jamás en holdout.
