# Combustión reducida Wiebe — S10

## Ecuaciones

`x(θ) = 1−exp[−a·((θ−θs)/Δθ)^(m+1)]`, `dx/dθ` analítica; zona única
`dp/dθ = −γ·p/V·dV/dθ + (γ−1)/V·dQ/dθ`,
`dQ/dθ = m_fuel·LHV·eff·dx/dθ` (una sola vez), `Q_pared = 0`.
Torque indicado `τ = p·dV/dθ`; `IMEP = W_neto/Vd`.

## Verificación (medida 2026-10-10, entradas ilustrativas 25 mg/LHV44/eff1)

Retrasar θs 340°→350°→360°: W 562.5→527.1→477.6 J (monótono),
pmax 5852@377.8°→4314@386.3°→3179@397.0° (desplaza y cae).
Calor auditado por GL: err 5.5e−8. Combustible 0 ⇒ motored reversible.
IMEP ilustrativo 14.1 bar: por encima de la cordura familiar en la
dirección esperada de un modelo sin pérdidas (no validación).

## Dominio y deuda

Ventana dentro de [180°,540°], γ constante, sin residuales/fugas/pared,
sin variación ciclo a ciclo, sin knock (UNSUPPORTED). `a/m/Δθ` no
universales: etiquetados EXTRAPOLATED hasta p(θ) real.
