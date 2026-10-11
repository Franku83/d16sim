# Sistema térmico L1 — S21

## Modelo

Nodos metal/pistón/aceite/refrigerante + ambiente BC; quemado→paredes
(fw = 0.30) + resto al gas; fricción→aceite; pistón↔metal, metal↔
refrigerante/aceite/ambiente, aceite→ambiente, radiador con termostato
80–88 °C; ECT con lag 5 s → enriquecimiento 1.25/1.05/1.0.
Capacitancias/conductancias ilustrativas de orden de magnitud.

## Verificación (2026-10-10)

Sin quemado no hay calentamiento de paredes; fricción→aceite exacta;
termostato 0/0.5/1 en banda; ECT sigue con lag (63% en τ); arranque en
frío 30 s: ECT 20→20.33 °C, aceite 22.64, termostato cerrado,
enriquecido inyecta más; masa 4e−10, energía 3.7e−4; S16 intacto.

## Dominio y deuda

Sin hA/C medidos, sin curva real de termostato/bomba, sin ECT con S19
completo, sin viscosidad (S22), calentamiento real en minutos (fuera de
horizonte de test: se verifica inicio + lógica + auditoría).

## Remediación R01 (2026-10-10)

- **Frontera ambiente en el ledger (R01-F10):** `ThermalState::step`
  devuelve el calor neto cedido al ambiente (pared + aceite + radiador con
  termostato) y el motor lo acumula (`th_q_amb_j`) como sumidero declarado
  del balance energético integrado. El residuo que persistía con
  refinamiento (≈−0.0039 a 120 s) ya no es un término omitido.
- **Fail-closed (R02-F15):** las capacidades deben ser estrictamente
  positivas; una capacidad nula produce error tipado antes de mutar, no un
  estado no finito.

## Remediación R02-C1/RV06 (2026-10-10)

`step` valida las temperaturas de entrada y el ambiente (finitud y Kelvin
sobre el cero absoluto) **antes** de calcular, computa el estado candidato en
locales y solo lo confirma si es finito y físico. Temperaturas imposibles,
ambiente bajo cero absoluto y overflow por capacidad extrema devuelven error
tipado **sin mutar** el estado (atomicidad). El calor `q_amb` es energía (J) y
puede ser negativo si el ambiente está más caliente; solo se exige finitud.

## Remediación R02-C3/C2-03 (2026-10-10)

La energía térmica representable se evalúa contra el **estado real** de cada
API (`validate_physical_state`, `initial_state`), no contra una temperatura de
fábrica a 20 °C. Un estado con capacidad grande y temperatura baja cuya energía
actual es finita se acepta; el mismo par capacidad/temperatura que desborda
T·C se rechaza de forma atómica. `ThermalParams::validated` sigue comprobando
la validez intrínseca (capacidades > 0, fracción y bandas coherentes).
