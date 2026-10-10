# Herramientas diagnósticas — S20

## Modelo

Sondas (acceso/polaridad/masa), multímetro 10 MΩ con carga/rango/
resolución/guarda, osciloscopio (timebase/tasa/trigger/ganancia/offset,
aliasing declarado), scanner espejo (lecturas+DTC entregados), manual
indexado por enlaces, casos ambiguos con veredicto.

## Verificación (2026-10-10)

Polaridad/acceso, carga 1%/rango/guarda 330 Ω, trigger en arcsin,
alias 5 kHz→1 kHz declarado, scanner sin invenciones, manual por
keyword, MAP 1.00≈1.00 V inconcluso sin referencia y discriminado con
ella, jitter vs offset separados >10× por varianza.

## Dominio y deuda

Sin instrumentos mecánicos (S24), sin Bode completo, sin sonda de
presión in-cylinder, sin modo tutorial (Fase D), sin evaluador externo
(opcional futuro). Ninguna ayuda automática en exploración libre.

## Remediación R01-F02 (2026-10-10)

`mech-demo` construye el reparto de fuga (`LeakSplit`) a partir de la **red
física y de los caudales integrados** (nodos cárter/refrigerante), nunca del
identificador `LeakScenario`. Renombrar/reordenar escenarios con los mismos
inputs físicos preserva la observación; causas con la misma firma permanecen
ambiguas. El escape, sin puerto en la red, se declara `no instrumentado`
(frontera 0), no un porcentaje heurístico. La construcción de la avería y la
de la observación quedan desacopladas.
