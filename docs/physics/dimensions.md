# Especificación de dimensiones — S03

Unidades SI internas, tipos dimensionales, conversiones y política `f64`.
Implementación: crates `units` (tipos) y `math` (interpolación).

## 1. Tipos del core (SI, campo privado, constructores validados)

| Magnitud | Tipo | Unidad SI | Rango | Notas |
|---|---|---|---|---|
| Longitud | `Length` | m | `>= 0` | `from_m/mm` |
| Ángulo | `AngleRad` | rad | finito ± | `from_rad/deg` |
| Fuerza | `Force` | N | finito ± | |
| Torque | `Torque` | N·m | finito ± | **sin** `Add` con `Kelvin`: no compila |
| Masa | `MassKg` | kg | `>= 0` | `as_moles(mm)` exige masa molar |
| Energía | `EnergyJoule` | J | finito ± | |
| Caudal másico | `MassFlowKgPerS` | kg/s | `>= 0` | |
| Régimen | `Rpm` | rev/min | `>= 0` | nunca rad/s implícitos |
| Vel. angular | `AngularVelocity` | rad/s | finito ± | admite reversa; `to_rpm` satura <0 a 0 |
| Temperatura | `Kelvin` | K | `>= 0` | °C solo en `display` |
| Presión abs. | `PressureAbs` | Pa | `>= 0` | la que usa la termodinámica |
| Presión dif. | `PressureDelta` | Pa | finito ± | reguladores, sensores Δp |
| AFR | `Afr` | — | `> 0` | |
| Lambda | `Lambda` | — | `> 0` | `to_afr/to_lambda` con stoich explícito |
| Potencia banco | `CrankWatts` | W | `>= 0` | al cigüeñal (freno) |
| Potencia rueda | `WheelWatts` | W | `>= 0` | `to_wheel(e)` con `e ∈ (0,1]` explícito |

Aritmética solo intra-tipo (`Length+Length`, `Abs±Delta`, `Abs−Abs→Delta`).
`Rpm↔rad/s`, `AFR↔lambda`, `gauge↔abs`, `banco↔rueda` son llamadas
explícitas con parámetros visibles. Sin `From` silenciosos.

## 2. Presión absoluta vs gauge (regla S03.3)

El "gauge" no es un tipo almacenable: es `PressureDelta` + ambiente.
`abs.to_gauge(ambient)` / `delta.to_abs(ambient)` (satura a 0 si el sensor
implicaría vacío imposible). Tratar bar-gauge como Pa absoluto no compila
(tipos distintos) y no existe función que lo haga.

Consecuencia S13: `Δp_inyector = p_rail_abs − p_manifold_abs`; el rail con
vacío conectado/desconectado cambia ambas, no el Δp ideal.

## 3. Conversiones visuales (solo UI/CLI, `units::display`)

psi/bar/kPa, °C/K, N·m/lb·ft, hp/kW como `f64→f64` con factor documentado.
Tests de simetría/consistencia contra el core tipado. Nunca entran al core.

## 4. Política `f64`

- Finito exigido en cada constructor (`NonFinite` si NaN/inf).
- Subnormales → `+0.0` determinista (`flush_subnormal`).
- Serialización Ryū round-trip (17 dígitos); sin `f32` en el core.
- Interpolación (`math::Table1`): nodos `>= 2`, longitudes iguales, finitos,
  abscisas estrictamente crecientes; `x` no finito rechazado; fuera de
  dominio: `Clamp` o `Reject` explícito; `unit_label` viaja al error.

## 5. Deuda S03

- Suite `trybuild` de no-compilación (torque+temperatura, Rpm→rad/s
  implícito, gauge→abs silencioso): diferida; el mecanismo existe por
  construcción + tests de validadores.
- Constantes físicas (R universal, composiciones): S09 con fuente.
