# Dinámica de cigüeñal — S14

## Ecuaciones

`dθ/dt = ω`, `J·dω/dt = τ_ind − τ_fric − τ_carga`;
`τ_ind = Σ p_i·dV_i/dθ` (4 cilindros en fase, traza completa);
FMEP `= c1 + c3·Sp + c4·Sp²` (c2 = 0 declarado) → `τ = FMEP·Vd/4π`;
`J_eff` por media energética; RK4 temporal + trabajo por subpasos.

## Verificación (ilustrativo, 2026-10-10)

Sin combustión + fricción ⇒ 750→0 rpm en 2 s; W_ind 0.44 J frente a
W_fric 464.7 J; cierre 2.7e−8; p cte ⇒ trabajo neto 0; refinamiento
converge; rotor bloqueado con carga se queda; J(0.15/0) = 0.15 exacto.

## Dominio y deuda

J constante (θ-dependencia futura), un solo FMEP global (sin cojinetes
locales), sin accesorios/arranque (S15), sin combustión acoplada (S16),
sin control de velocidad. Coeficientes ilustrativos, no D16Y7.
