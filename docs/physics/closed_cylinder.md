# Cilindro sellado ideal — S09

## Ecuaciones

Gas ideal `pV = mRT` (R/γ explícitos; `cp/cv` derivados); primera ley con
geometría variable y Q = 0: `dp/dθ = −γ·p/V·dV/dθ`. RK4 paso fijo por
ángulo; trabajo por Simpson. Referencia: `p = p_ref·(V_ref/V)^γ`;
`W_comp = (p2V2−p1V1)/(1−γ)`; reversible ⇒ `W_neto ≈ 0`.

## Verificación (medida 2026-10-10, preset ilustrativo)

`W_comp` num vs analítico: 1.4e−10 relativo; `W_net` 8.6e−9 J;
`max_p_rel_err` 7.4e−11 (paso 0.25°); orden de convergencia 3.888 (≈4).
Masa atrapada consistente (448.8 mg a 101.325 kPa/350 K). Positividad
p/V/T en toda la serie.

## Dominio y deuda

Cilindro cerrado, γ constante, sin fugas/residuales/pared/combustión;
válido con `m,T,V,p > 0`. Esto verifica el integrador, no valida el D16Y7.
