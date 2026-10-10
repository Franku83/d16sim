# Libro de hipótesis — S00

`model_assumptions.md`. En S00 **no hay modelos**, por tanto no hay
hipótesis numéricas que registrar. Este archivo existe para que futuros
sprints no tengan dónde esconderse.

## Formato obligatorio (desde S06 en adelante)

Por hipótesis: `id`, ecuación/aproximación, dominio (RPM/MAP/T),
fuentes, parámetros afectados, vigencia, condición que la invalida,
etiqueta `[H]`, responsable, fecha.

## Registro S00

| ID | Hipótesis | Estado |
|---|---|---|
| — | (vacío intencional) | Sin modelos en S00 |

Prohibido añadir constantes «de conveniencia» silenciosas en sprints
futuros: toda aproximación pasa por aquí + `ModelCard` (§11.4).
