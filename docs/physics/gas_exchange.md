# Gas exchange 0D — S12

## Ecuaciones

Orificio: `ṁ = Cd·A·p0/√(R·T0)·Ψ(pr)`, `Ψ` estrangulado bajo
`pr_crit = (2/(γ+1))^(γ/(γ−1))`, continuo a `pr → 1`, reversible por
orden up/down. Válvula: `A = 2·Cd(L)·π·D·L` con Cd medido (adm Y7
FORUM-MEASURED ±7%, esc P2A-06 ±10%, D 30/26 mm MFR-TECH).
Nodos `p = mRT/V`, `U = m·cv·T`, dos especies; cilindro abierto
`dU/dθ = −p·dV/dθ + (Σh_in − Σh_out)/ω`; Euler explícito; ambiente BC.

## Verificación (medida 2026-10-10, preset ilustrativo)

Cierre de masa ~1e−15; energía ~1e−15 (identidad contable, no prueba de
precisión); estrangulamiento satura; límite incompresible ±2%;
hermético con área 0; mariposa dosifica el plenum (0.79×); masa atrapada
converge al refinar (< 1% en malla fina); reflujo medido −16.9 g/s con
especies exactas (2/3); traza finita en 2881 puntos.

## Dominio y deuda

Sin ondas/inercia de columna (1D), sin pérdidas distribuidas de puerto,
sin garganta explícita (doble conteo evitado), alzada sen² (bordes
subestimados), γ/R constantes, un cilindro (multi-cilindro en S16),
primer ciclo transitorio (periódico en S16), ralentí/arranque (S15+).
