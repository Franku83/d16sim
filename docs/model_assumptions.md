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

## Registro S09 (cilindro sellado ideal)

| ID | Hipótesis | Dominio | Fuente | Invalida si |
|---|---|---|---|---|
| H-S09-01 | Gas ideal, R/γ constantes | Ciclo cerrado S09 | Supuesto declarado | cp(T,mezcla) en S10+ |
| H-S09-02 | Q_pared = 0, sin fugas ni residuales | Ciclo cerrado S09 | Supuesto declarado | Modelos S12/S21/S23 |
| H-S09-03 | Entradas p/T BDC ilustrativas (101.325 kPa/350 K) | Demo S09 | Supuesto declarado | Medición en motor |
| H-S09-04 | RK4+Simpson verifica contra adiabática, no valida motor | Tests S09 | ADR-016 | — |

## Registro S10 (Wiebe zona única)

| ID | Hipótesis | Dominio | Fuente | Invalida si |
|---|---|---|---|---|
| H-S10-01 | Wiebe a=5/m=2/Δθ=60° ilustrativos, no universales | Demo S10 | Default de libro + research | p(θ) medida |
| H-S10-02 | Zona única, γ const, Q_pared=0, eff=1, sin residuales | Ciclo S10 | Supuesto declarado | S12/S21, γ(T) |
| H-S10-03 | Combustible 25 mg y LHV44 ilustrativos (no P2A-000) | Demo S10 | Supuesto declarado | Banco P2A-000 |
| H-S10-04 | IMEP ilustrativo ≠ validación (overshoot esperado) | Reporte S10 | ADR-017 | Calibración S25 |

## Registro S11 (distribución proxy A6)

| ID | Hipótesis | Dominio | Fuente | Invalida si |
|---|---|---|---|---|
| H-S11-01 | Sin perfil Y7 stock (MISSING-DATA); solo proxy A6 | S11 | Research donantes | Medición lift-vs-CA |
| H-S11-02 | ICL 110° straight-up asumido y declarado | Fases S11 | Supuesto (donantes) | Medición centerline |
| H-S11-03 | Ley sen² simétrica ilustrativa (050 subestimado) | Alzada S11 | Supuesto declarado | Tabla medida |
| H-S11-04 | Correa 2:1, dientes ilustrativos, solo desfasa eventos | S11 | Supuesto declarado | Conteo OEM + patrón |
| H-S11-05 | Contacto informado, daño pospuesto a material | S11 | Decisión (gate S27) | Modelo S27 |

## Registro S12 (gas exchange 0D)

| ID | Hipótesis | Dominio | Fuente | Invalida si |
|---|---|---|---|---|
| H-S12-01 | Cd medido por válvula, sin garganta (no doble conteo) | Flujo S12 | Research + §24.4 | Cd con banco + garganta |
| H-S12-02 | Dos especies; residual quemado exige combustión | Nodos S12 | Decisión + física | Acople S16 |
| H-S12-03 | Volúmenes/áreas ilustrativos; mariposa = área efectiva | Demo S12 | Supuesto declarado | Geometría medida |
| H-S12-04 | Primer ciclo transitorio; periódico en S16 | Ciclo S12 | Supuesto declarado | Operación continua |
| H-S12-05 | Euler explícito; masa exacta contable, energía auditada | S12 | ADR-019 | Acoplamiento implícito |

## Registro S13 (inyección L1)

| ID | Hipótesis | Dominio | Fuente | Invalida si |
|---|---|---|---|---|
| H-S13-01 | Q_ref 185 cc/min @3 bar = hipótesis, no medición | Inyector S13 | Claim sin ΔP | Banco P2A-000 |
| H-S13-02 | Deadtime analógico CBR600RR ±0.3 ms (no P2A-000) | Inyector S13 | Analogía | Deadtime P2A medido |
| H-S13-03 | Regulador ideal ΔP cte; bomba off ⇒ rail 0 | Rail S13 | Supuesto declarado | Curva regulador/bomba |
| H-S13-04 | Gasolina ilustrativa (740 kg/m³, LHV44, 14.7) | S13 | Orden comercial | Lote medido |

## Registro S14 (dinámica de cigüeñal)

| ID | Hipótesis | Dominio | Fuente | Invalida si |
|---|---|---|---|---|
| H-S14-01 | Indicado = ciclo completo (bombeo dentro); fricción aparte | S14 | Decisión ADR-021 | Esquema alternativo doc. |
| H-S14-02 | J constante por media energética; J(θ) deuda | S14 | Supuesto declarado | J(θ) medido/modelado |
| H-S14-03 | Chen–Flynn c2=0, coeficientes ilustrativos del llamante | S14 | Pipitone + supuesto | FMEP medido |
| H-S14-04 | Traza S12 replicada en 4 cilindros idénticos | Demo S14 | Supuesto declarado | Diferenciación S16 |

## Registro S15 (arranque arrastrado)

| ID | Hipótesis | Dominio | Fuente | Invalida si |
|---|---|---|---|---|
| H-S15-01 | Batería/starter/bobina ilustrativos 12 V (no medidos) | S15 | Supuesto declarado | Curvas medidas |
| H-S15-02 | Acoplo τ·(V/V0) lineal L1; eficiencia diagnóstico | S15 | Supuesto + test η≤1 | Modelo V explícito |
| H-S15-03 | Reductora 12:1 ilustrativa explícita | S15 | Supuesto declarado | Relación OEM |
| H-S15-04 | Compresión = traza S12 (velocidad pendiente) | S15 | Supuesto declarado | Gas exchange vivo S16 |
| H-S15-05 | `started=false` por diseño; bootstrap prohibido | S15 | Decisión ADR-022 | Integración S16 |
