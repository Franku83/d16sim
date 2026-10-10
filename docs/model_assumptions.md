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

## Registro S07 (contactos sin material)

| ID | Hipótesis | Dominio | Fuente | Invalida si |
|---|---|---|---|---|
| H-S07-01 | Huelgo axial coaxial con corona plana; offset lateral irrelevante en axial | Geometría S07 | Supuesto declarado + test | Corona no plana o válvula angulada real |
| H-S07-02 | Biela–bloque simétrico L0 | Entradas S07 | Supuesto declarado | Geometría de cárter real |
| H-S07-03 | Ningún contacto implica daño sin modelo material | Todo S07 | Decisión de diseño (gate) | S27 con criterio validado |
| H-S07-04 | Medición L1 aditiva | Instrumentos S07 | Supuesto declarado | Caracterización de instrumento |

## Registro S08 (precarga y junta L1)

| ID | Hipótesis | Dominio | Fuente | Invalida si |
|---|---|---|---|---|
| H-S08-01 | `F = T/(K·d)` con K por condición | L1 S08 | Modelo torque-factor (texto) | Curva clamp-load medida |
| H-S08-02 | Rangos K ilustrativos genéricos (NO OEM) | Sensibilidad S08 | Lit. genérica, cal. pendiente | Medición de K |
| H-S08-03 | Reparto por pesos de rigidez, secuencia sin efecto en L1 | Junta S08 | Supuesto declarado | Modelo elástico S08+ |
| H-S08-04 | Sellado indeterminado sin ley de fuga | Junta S08 | Decisión (gate S23) | Solver S23 |
