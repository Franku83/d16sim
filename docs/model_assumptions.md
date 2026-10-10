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

## Registro S06 (cinemática ideal)

| ID | Hipótesis | Dominio | Fuente | Invalida si |
|---|---|---|---|---|
| H-S06-01 | Biela-manivela ideal sin offset de bulón | λ<1, θ finito | Cinemática de mecanismos (texto) | Se mide offset o se requiere |
| H-S06-02 | Régimen estacionario (`dω/dt=0`) en `state()` | Evaluación S06 | Supuesto declarado | S14 integra dinámica |
| H-S06-03 | Preset ilustrativo 75×90/137/CR9.4 solo para verificación | Tests/demos S06–S10 | S01 PENDING + ADR-014 | Auditoría OEM lo sustituye |
| H-S06-04 | Compañeras (1,4)/(2,3) y orden 1-3-4-2 ilustrativos | Fases S06 | S01 PENDING | Manual variante fija topología |
