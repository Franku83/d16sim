# Limitaciones y no-soportado — S00

Estado inicial. Todo lo no implementado es `UNSUPPORTED` hasta su sprint.
Niveles L0–L4 por subsistema en `docs/foundation.md` §3.1; en S00 todo el
núcleo físico está en **L0 (semántico/declarado) o inexistente**, porque no
hay código de física.

## Etiquetas de credibilidad

`UNVERIFIED` · `VERIFIED_NUMERICALLY` · `CALIBRATED` · `VALIDATED_IN_DOMAIN` ·
`EXTRAPOLATED` · `UNSUPPORTED`. En S00: todo `UNVERIFIED` o `UNSUPPORTED`.

## No-soportado global en S00

- Todo solver termodinámico, cinemático, de flujos, combustión, fricción,
  ECU, sensores, aceite, refrigeración, desgaste, fatiga, dyno.
- Toda predicción de potencia, torque, consumo, temperaturas, presiones
  indicadas, knock, roturas, vida útil.
- Toda curva OEM (leva stock Y7 lift-vs-CA, flow OEM certificado, p(θ),
  dyno stock completo, mapas P2E, deadtime P2A-000, FMEP D16Y7).
- Toda UI/3D (Tauri/Three.js/WASM). Prohibida en S00 por brief.

## Estado tras S01–S05 (sin cambios físicos: sigue vigente lo anterior)

- S01 fijó variante y ficha con todo número en PENDING/DISPUTED.
- S02–S05 añaden infraestructura (toolchain, runner, unidades, grafo,
  historia) sin ningún solver: `physics::closed_cycle_placeholder`
  retorna `PhysicsNotImplemented(S09)` y `d16sim physics-status` lo
  demuestra. Las trazas/demos marcadas `runner_plumbing_only` no son
  resultados de motor.

## Dominios de extrapolación (recordatorio, no autorización)

- Proxy de leva D16A6, ratio I/E, Q(ΔP), deadtime analógico CBR600RR,
  priors Wiebe/Woschni/Chen-Flynn: todos `EXTRAPOLATED` o `ANALOGÍA` hasta
  S01–S25. Ver research §24.4 guardrails (no doble conteo de Cd, no ΔP
  manométrica como diferencial, no n único desde 184 psi sin IVC/absoluto).
- Compresímetro nominal 1270 kPa: fixture de **instrumento con condiciones
  250 rpm + WOT**, no presión de combustión. `REQUIRES_VERIFICATION` de
  edición/página/variante en S01.

## Matriz por subsistema (contrato de cobertura §4, estado S00)

Todos inician L0/inexistente a nivel código. La columna fase es sugerencia,
no autorización.

| Subsistema | Estado S00 |
|---|---|
| Block/crankcase … Herramientas/operaciones (25 filas §4) | `UNSUPPORTED` (sin representación ni mecanismo en código) |
| Turbo | L0 hasta expansión (declarado, sin modelo) |
| Instrumentación | `UNSUPPORTED` (sin probes ni scopes) |

## Compromiso

Sin datos medidos, el hito BASE-REPRODUCIBLE será *base computacional
validada solo parcialmente*. No etiquetar «predictivo de ingeniería».
