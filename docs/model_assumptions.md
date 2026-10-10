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

## Registro S16 (integración causal)

| ID | Hipótesis | Dominio | Fuente | Invalida si |
|---|---|---|---|---|
| H-S16-01 | Splitting explícito + Euler; convergencia medida | S16 | ADR-023 | Rigidez/implícito |
| H-S16-02 | Control open-loop (ηv 0.75, AFR 13.5, avance 15° fijos) | S16 | Supuesto declarado | Lazo cerrado S18 |
| H-S16-03 | AFR→quemado sin acoplar; eff=1, γ const | S16 | Limitación declarada | Modelo S12+quemado |
| H-S16-04 | Punto ralentí = posición de mariposa (experimento) | S16 | Supuesto declarado | Control S19 |
| H-S16-05 | Régimen emergente reportado tal cual, no stock | S16 | Gate S16 | Validación S25 |

## Registro S17 (triggers GENERIC_TRIGGER)

| ID | Hipótesis | Dominio | Fuente | Invalida si |
|---|---|---|---|---|
| H-S17-01 | Patrón 24/4/1 familiar, prohibido OEM | S17 | TRIG-2026-10-10 | Scope + página OEM |
| H-S17-02 | VR L1 (amplitud∝ω, ruido seed, histéresis) ilustrativo | S17 | Supuesto declarado | Medición VR |
| H-S17-03 | Fases TDC/CYP ilustrativas + offset de carcasa | S17 | Supuesto declarado | Diagrama OEM |
| H-S17-04 | CKF concepto aparte, presencia PENDING, no modelado | S17 | rusEFI serie B | Variante confirma |

## Registro S18 (ECU P2E-like, no OEM)

| ID | Hipótesis | Dominio | Fuente | Invalida si |
|---|---|---|---|---|
| H-S18-01 | Mapas/códigos propios, nunca OEM ni aftermarket | S18 | Decisión (MISSING P2E) | Dumps/estrategia OEM |
| H-S18-02 | Sinc CYP-gated 24+1 con respaldo 4+1 | S18 | TRIG + diseño | Diagrama OEM |
| H-S18-03 | Latencia fija 1 ms ilustrativa; 400 rpm clasifica | S18 | Supuesto declarado | Medición ECU |
| H-S18-04 | Dead-reckoning entre flancos a rpm constante | S18 | Supuesto declarado | Transitorios medidos |

## Registro S19 (sensores y cableado)

| ID | Hipótesis | Dominio | Fuente | Invalida si |
|---|---|---|---|---|
| H-S19-01 | Transferencias ilustrativas, no curvas OEM | S19 | Supuesto declarado | Tablas OEM/banco |
| H-S19-02 | NTC con pull-up 2k/2k y B=3435 ilustrativos | ECT S19 | Orden típico | Curva ECT OEM |
| H-S19-03 | O2 narrowband como conmutador con histéresis futura | S19 | Física de sensor | Dinámica con heater |
| H-S19-04 | Acción ECU sin camino a termodinámica (tipos) | S19 | Decisión ADR-026 | — |

## Registro S20 (instrumentos sin panel secreto)

| ID | Hipótesis | Dominio | Fuente | Invalida si |
|---|---|---|---|---|
| H-S20-01 | Multímetro 10 MΩ/20 V/1 mV ilustrativo | S20 | Orden típico | Hoja de instrumento |
| H-S20-02 | Scope sin antialias: alias declarado, no oculto | S20 | Decisión ADR-027 | Filtro modelado |
| H-S20-03 | Scanner espejo; manual por enlaces, sin contenido | S20 | Decisión (licencias) | Protocolo variante |
| H-S20-04 | Ambigüedad exige complementaria; verdad solo monta | S20 | Gate S20 | — |

## Registro S21 (térmico L1)

| ID | Hipótesis | Dominio | Fuente | Invalida si |
|---|---|---|---|---|
| H-S21-01 | Red opt-in; None = S16 exacto | S21 | Decisión ADR-028 | — |
| H-S21-02 | fw = 0.30 a paredes; fricción al aceite | S21 | Supuesto declarado | Reparto medido |
| H-S21-03 | hA/C ilustrativos de orden de magnitud | S21 | Supuesto declarado | Caracterización |
| H-S21-04 | Termostato 80–88 genérico; ECT lag 5 s | S21 | Supuesto declarado | Curvas OEM |
| H-S21-05 | Enriquecimiento ECT 1.25/1.05/1.0 (cadena L1) | S21 | S19 + supuesto | Closed-loop |

## Registro S22 (lubricación por red)

| ID | Hipótesis | Dominio | Fuente | Invalida si |
|---|---|---|---|---|
| H-S22-01 | Walther con 70/10.5 cSt ilustrativos | S22 | Orden comercial | Curvas de aceite |
| H-S22-02 | Bomba 8cc/rev + alivio 500 kPa ilustrativos | S22 | Supuesto declarado | Mapa de bomba |
| H-S22-03 | Fuga lateral L1 por holgura; resto de fugas no | S22 | Supuesto + límite | Inventario S25 |
| H-S22-04 | Película μ·N/p con k=1e5 + inanición (no HD) | S22 | Supuesto declarado | Calibración S26 |
| H-S22-05 | Pinneo en alivio declarado (sin factor cobertura) | S22 | Decisión ADR-029 | Consumidores S25 |

## Registro S23 (fugas con ley explícita)

| ID | Hipótesis | Dominio | Fuente | Invalida si |
|---|---|---|---|---|
| H-S23-01 | Junta cuadrática con p_ref de clamp completo | S23 | Forma ilustrativa + ancla | Medición S25 |
| H-S23-02 | Anillo geométrico × Cd 0.03 (no calibrado) | S23 | Supuesto declarado | Banco S25 |
| H-S23-03 | Suministro regulado + PCV + tapón 110 kPa | S23 | Protocolo declarado | Tester S24 |
| H-S23-04 | Magnitudes no predictivas; direcciones robustas | S23 | Decisión ADR-030 | Calibración |

## Registro S24 (instrumentos mecánicos)

| ID | Hipótesis | Dominio | Fuente | Invalida si |
|---|---|---|---|---|
| H-S24-01 | Cranking L1 con C=8e5 ilustrativa | S24 | Supuesto declarado | Banco S25 |
| H-S24-02 | Leak-down L1 con orificio 1 mm² | S24 | Supuesto declarado | Equipo real |
| H-S24-03 | Reparto como observación del harness | S24 | Decisión ADR-031 | Tester S25 |
| H-S24-04 | Umbrales de taller sin calibrar | S24 | Ayuda declarada | Manual/banco |
| H-S24-05 | Subapretada≈anillos con 2 tests (3º prescrito) | S24 | Límite documentado | 3er test |

## Registro S25 (calibración sin datos de campo)

| ID | Hipótesis | Dominio | Fuente | Invalida si |
|---|---|---|---|---|
| H-S25-01 | 6 params identificables con taller; 8 bloqueados | S25 | Decisión ADR-032 | Llegada de C1–C7 |
| H-S25-02 | Self-test sintético verifica fitter, no valida | S25 | Regla EVIDENCE | Mal uso (prohibido) |
| H-S25-03 | Ancla sellada rompe degenerescencia C/p_ideal | S25 | Análisis (corr −0.98→−0.54) | Banco C2 |
| H-S25-04 | Cierre NUMERICALLY_VERIFIED_ONLY; precisión bloqueada | S25 | Decisión ADR-032 | Manifiesto con datos |

## Registro S26 (desgaste persistente)

| ID | Hipótesis | Dominio | Fuente | Invalida si |
|---|---|---|---|---|
| H-S26-01 | Archard oiled K=1e-8 efectivo (rango 1e-9–1e-7) | S26 | Orden µm/50h ilustrativo | Tribología P1 |
| H-S26-02 | Cojinete Kb=5e-18 con (1−film); aceite A=5e4/Ea=80kJ | S26 | Supuesto declarado | Banco P1 |
| H-S26-03 | Creep logarítmico integrado en forma cerrada | S26 | Decisión ADR-033 | Modelo de junta |
| H-S26-04 | Fatiga/scuffing/fretting UNSUPPORTED | S26 | Sin ley | Ley constitutiva |
| H-S26-05 | Salidas TREND_CONDITIONED, nunca vida útil | S26 | Regla S26 | Evidencia S30 |

## Registro S27 (daño por mecanismos)

| ID | Hipótesis | Dominio | Fuente | Invalida si |
|---|---|---|---|---|
| H-S27-01 | Solape PV geométrico; doblado exige material PENDING | S27 | S07 + límite | Material válvula |
| H-S27-02 | Rp0.2 940 MPa clase 10.9 GENÉRICO (no D16Y7) | S27 | ISO 898-1 clase | Perno medido |
| H-S27-03 | Ruptura solo si todo el rango K supera; si cruza, nada | S27 | Decisión ADR-034 | K medido |
| H-S27-04 | Gripado/rosca UNSUPPORTED o riesgo sin criterio | S27 | Sin ley | Criterio medido |
| H-S27-05 | Chequeos puros: diagnosticar no altera trayectoria | S27 | Diseño | Acople S29 |
