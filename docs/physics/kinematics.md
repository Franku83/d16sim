# Cinemática biela-manivela — S06

Mecanismo ideal sin offset de bulón. Ecuaciones, unidades SI (m, rad) y
derivación en `crates/kinematics/src/geometry.rs` (documentadas en código).

## Ecuaciones implementadas

`r = S/2`, `λ = r/l < 1`, `A = πB²/4`, `x(θ) = r(1−cosθ) + l − √(l²−r²sin²θ)`,
`dx/dθ`, `d²x/dθ²` analíticas (ver código), `V = Vc + A·x`,
`Vc = Vd/(CR−1)`, `dV/dθ = A·dx/dθ`, `v = dx/dθ·ω`, `a = d²x/dθ²·ω²`
(estacionario). Fases 720°: firing-TDC por cilindro, carrera por local,
posición por `local % 360` (dos cilindros en PMS en carreras distintas).

## Verificación (no validación)

- PMS/PMI exactos (x=0/S, Vmin=Vc, Vmax=Vc+Vd, velocidad nula).
- Simetría `x(θ)=x(−θ)`; derivadas analíticas vs FD4 (tol 1e−9/1e−6).
- `λ ≥ 1` y `CR ≤ 1` rechazados; preset ilustrativo da 1590.43 cm³/4cil.
- Compañeras (1,4)/(2,3) comparten manivela en todo global; encendidos
  cada 180°.

## Dominio y deuda

Válido para `λ < 1`, `θ` finito, `ω ≥ 0` estacionario. Sin offset de bulón,
sin juego térmico/dinámico, sin masas (parámetros de inercia en S14),
sin contacto pistón-válvula (S07/S11). Masas/inercia: separadas de
topología por diseño (llegan en S14); S06 no integra RPM.
