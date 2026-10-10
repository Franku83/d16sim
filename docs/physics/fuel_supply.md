# Suministro de combustible L1 — S13

## Modelo

Rail con regulador referenciado (`p_rail = p_man + ΔP_base`, ΔP 3 bar
ilustrativo); bomba on/off + capacidad. Inyector:
`Q = Q_ref·√(ΔP/P_ref)`, `PW = m/(Q·ρ) + deadtime(V)`,
deadtime analógico 1.29/0.93/0.74/0.62/0.50 ms @10–15 V ±0.3 ms;
no-lineal bajo 2.0 ms efectivos; entrega recortada por capacidad.
Agenda por estrategia (Sequential/Grouped/Simultaneous) con ángulos
ilustrativos (scheduler real en S18).

## Verificación (hipótesis etiquetadas, 2026-10-10)

Ralentí: 7.69 mg ⇒ PW 3.37+0.62 = 3.99 ms (aritmética research exacta);
±0.3 ms ⇒ ±7.5%; 12 V ⇒ ~8% pobre; Q(2.5 bar) = 168.9 (ambigüedad ±6%);
round-trip comandado→entregado exacto; bomba off ⇒ NoPressure;
capacidad ⇒ Saturated parcial; pulsos < deadtime ⇒ 0.

## Dominio y deuda

Sin atomización/wall-wetting, sin curva de caída del rail, sin
temperatura de combustible, sin caracterización P2A-000 (banco S25),
sin ECU (S18). Ningún rendimiento se calcula desde AFR.
