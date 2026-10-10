# D16Y7 — RESEARCH DE DONANTES SOHC NON-VTEC v0.1

**Fecha:** 2026-10-09 · **Propósito:** cerrar por transferencia los huecos del D16Y7 usando hermanos de la familia SOHC non-VTEC. Complementa `D16Y7_RESEARCH_v0.1.md`; no rehace el spec. Reportes brutos en `workspace/engine_sandbox/research/raw/`.
**Niveles de transferencia:** `DIRECTA` (misma pieza/familia, justificar) · `ESCALADA` (requiere ajuste documentado) · `ANALOGÍA` (solo orden de magnitud). **[D]** = cálculo propio verificado.

## Donantes considerados y distancia al Y7

| Donante | Cilindrada | Bore×Stroke | CR | Potencia | BMEP pico pot. [D] | Distancia |
|---|---|---|---|---|---|---|
| D16Y7 (objetivo) | 1590 cc | 75×90 | 9.4 | 106 hp @ 6200 | 9.62 bar | — |
| **D16A6** (88–91) | 1590 cc | 75×90 | 9.1 | 108 hp @ 6000 | 10.13 bar | **Mínima**: geometría idéntica |
| D15B7 (92–95) | 1493 cc | 75×84.5 | 9.2 | 102 hp @ 5900 | 10.36 bar | Carrera −5.5 mm |
| D17A1 (01–05) | 1668 cc | 75×94.4 | 9.5 | 117 hp @ 6100 | 10.29 bar | Carrera +4.4 mm, gen. posterior |
| D15B8 (92–95, 8V) | 1493 cc | 75×84.5 | — | 70 hp @ 4500 | 8.71 bar | Arquitectura 8V distinta |

Clase de specs de catálogo: `FORUM-CLAIM` (OEM pendiente en S01). Conflicto registrado: una agregadora da el A6 en 110 hp @ 5600 (probable otro mercado) — no promediado, se usa el USDM.

---

# 0. RESUMEN EJECUTIVO

1. **Leva: el D16A6 es donante DIRECTO y acota el rango del Y7.** Lobe lift medido .225/.193" (`FORUM-MEASURED`), @.050" 194/196°, LSA 110°. El Y7 queda acotado: @.050" ∈ **[183°, 196°]** (cota inferior D17A1/B7, superior A6), lift adm ∈ **[.335", .374"]**. Si el sim con proxy A6 da potencia alta, la corrección va hacia el extremo manso, nunca hacia arriba.
2. **Escape: el hueco queda cerrado parcialmente.** Dataset público D14/Y7 (fundición P2A-06, la misma del Y7) con curvas I/E @ 28" H₂O: ratio **I/E 0.77–0.82**, misma forma de Cd que admisión pero decaimiento más rápido. Reemplaza la estimación 0.70±0.10 del v0.1.
3. **Inyectores: Q ∝ √ΔP confirmada empíricamente dentro del 2%.** El claim "180–190 sin ΔP" tiene ambigüedad intrínseca de ±6%. Deadtime OEM del P2A-000 sigue `MISSING-DATA`; mejor prior: OEM Honda/Keihin 220 cc (CBR600RR) 0.62 ms @14V ±0.3 ms, etiquetado `ANALOGÍA`. **El deadtime es el MISSING-DATA más caro del subsistema:** ±8% de error de dosificación a ralentí vs ±2.5% a plena carga.
4. **Combustión/fricción: la familia D es termodinámicamente consistente.** BMEP en pico de potencia: 9.62–10.36 bar (dispersión ±3.7%); en pico de torque: 11.03–11.24 bar (±2.4%). **Rango de cordura para el simulador: IMEP_bruto 11.5–13.0 bar a plena carga.** p(θ) publicada de la familia entera: `MISSING-DATA`. Dyno completo stock: hueco sistémico (tampoco existe para A6/B7/D17A1).
5. **Descomposición del n politrópico [D]:** el cierre tardío de admisión (CR efectiva 8.67 con advertised) explica ~1/3 de la brecha 269→184 psi; el resto (heat transfer + blow-by a 250 rpm) deja n≈1.17 contra CR efectiva. El modelo del compresímetro debe usar **IVC por duración advertised**, no @.050".

---

# 1. LEVAS DONANTES

## 1.1 D16A6 — mejor donante (geometría idéntica 75×90)

| Parámetro | Valor | Fuente | Clase |
|---|---|---|---|
| Adv. duration | 222° / 224° (adm/esc) | método no declarado | `FORUM-CLAIM` |
| Duration @ .050" | **194° / 196°** | estándar USA | `FORUM-CLAIM` |
| **Lobe lift (medido)** | **.225" / .193"** | base 1.220/1.262, lóbulo 1.445/1.455, comparador | `FORUM-MEASURED` (Mista Bone, honda-tech 2001) |
| Valve lift derivado | .360" / .347" (× rocker 1.60/1.80 documentado) | cálculo [D] | [D] |
| LSA | 110° | — | `FORUM-CLAIM` |
| Intake centerline | ~110° (straight-up asumido, declarar) | — | `MISSING-DATA` |
| Válvulas | 29.5 / 25.4 mm | Endyn (mide culatas) | `MFR-TECH` |
| Tabla lift-vs-CA | — | — | `MISSING-DATA` |

**Conflicto resuelto (no promediar):** valve lift .333/.362 (fuente A) vs .374/.353 (medido, fuente B). El lobe lift medido es el dato primario; el valve lift depende del rocker usado. **Fijar lobe lift .225/.193 como `FORUM-MEASURED` y computar valve lift con el rocker documentado.**

## 1.2 D15B7 — leva medida directamente, cota inferior de agresividad

| Parámetro | Valor | Clase |
|---|---|---|
| Lobe lift (leva DX=B7) | **.202" / .166"** (mismo instrumento que el A6) | `FORUM-MEASURED` (Mista Bone, honda-tech 2003) |
| Valve lift derivado | .335" / .304" | `FORUM-MEASURED` (derivado) |
| Delta vs A6 | **+.025" / +.030" de lobe en el A6** (comparativa directa) | `FORUM-MEASURED` |
| Fundición culata | **PM3 = misma que D16A6** ("the only difference is the cam") | `FORUM-CASE` |
| Duration / LSA / centerline | — | `MISSING-DATA` |

## 1.3 D17A1 — non-VTEC moderno, cota inferior de duración

| Parámetro | Valor | Clase | Nota |
|---|---|---|---|
| Adv. duration | 238° / 229° | `FORUM-MEASURED` (epcivic 2009) | Atribución exacta A1-vs-A2 **requiere confirmación** |
| Duration @ .050" | **183° / 171°** | `FORUM-MEASURED` | — |
| Gross lift | .338" / .310" (rocker ~1.55) | `FORUM-MEASURED` | — |
| Válvulas | 30 / 26 mm | `FORUM-CLAIM` | = Y7 |

**Lección metodológica [D]:** advertised más largo que el A6 (238 vs 222) pero @.050 más corto (183 vs 194) → **rampas más lentas**. Prueba de que advertised no transfiere entre métodos: solo el @.050 es comparable.

## 1.4 Precaución VTEC (obligatoria)

Lóbulos primarios (non-VTEC) de Y8: 211°/215° adv, 183°/187° @.050", .300"/.319" — **solo `ANALOGÍA` con etiqueta `VTEC-PRIMARY`**. Prohibido usar el tercer lóbulo (VTEC: 247° adv / 217° @.050" / .397") o su centerline/timing para el Y7. D15B8 (8V) y D14/D13: sin datos de leva → `ANALOGÍA`/`MISSING-DATA`.

## 1.5 Contraste teórico [D] (verificado)

**IVC y CR efectiva** (leva A6 como proxy, LSA 110°, instalada straight-up → ICL ≈ 110° ATDC):

| Caso | IVC | CR efectiva (sobre 9.4) |
|---|---|---|
| @.050" (194°) | 207° ATDC = **27° ABDC** | **9.09** |
| Advertised (222°) | 221° ATDC = **41° ABDC** | **8.67** |

Overlap @.050" = (194+196)/2 − 2×110 = **−25°** (sin overlap, típico de leva stock mansa); overlap advertised ≈ 3°.

**Descomposición del n politrópico** (compresímetro Y7: 184 psi @ 250 rpm WOT):

| Escenario | Presión |
|---|---|
| Ideal ingenua (CR 9.4, n=1.3) | 271 psi |
| Solo cierre tardío (CR_eff 8.67, n=1.3) | 244 psi → el IVC explica ~1/3 de la brecha |
| Resto hasta 184 psi | heat transfer + blow-by → **n ≈ 1.17** contra CR efectiva |

**Consecuencia para el simulador:** a 250 rpm el reflujo persiste hasta el asentamiento real — el modelo del compresímetro debe usar **IVC por duración advertised**, no @.050".

## 1.6 Proxy recomendado (L1, `EXTRAPOLATED`)

**Perfil D16A6:** adv 222/224° · @.050" 194/196° · LSA 110° · ICL 110° (asumido, declarar) · lobe lift **.225/.193"** `FORUM-MEASURED` · valve lift = lobe × rocker documentado · válvulas 29.5/25.4.
**Rango acotado Y7 [D]:** @.050" ∈ **[183°, 196°]** · lift adm ∈ **[.335", .374"]**. Dirección de corrección: hacia el extremo manso (D17A1/B7), nunca hacia arriba.

---

# 2. FLUJO DE ESCAPE — HUECO CERRADO PARCIALMENTE

**Hallazgo principal:** existe UN dataset público con curvas I/E @ 28" H₂O para la fundición del Y7 — hilo "D14 /y7 head flow numbers", dodo-upgrades.nl (abr-2010), banco casero. El posteador afirma que la culata D14 es "exactamente la misma fundición que la Y7 (P2A-06)"; P2A es efectivamente el código de culata del D16Y7 (12100-P2A-000 en EPC). Clase: `FORUM-MEASURED`, caveat banco casero.

## 2.1 Dataset Y7/D14 (P2A-06) @ 28" H₂O, por cilindro

| Lift (in) | Admisión (CFM/cil) | Escape (CFM/cil) | Ratio I/E |
|---|---|---|---|
| 0.100 | 75.3 | 61.1 | **0.811** |
| 0.150 | 111.5 | 91.1 | **0.817** |
| 0.200 | 137.5 | 111.3 | **0.809** |
| 0.250 | 144.7 | 122.1 | 0.844 ⚠️ |
| 0.300 | 163.7 | 125.7 | **0.768** |
| 0.350 | 165.2 | 127.9 | **0.774** |
| 0.400 | 166.4 | 128.6 | **0.773** |
| 0.450 | 167.6 | 129.4 | **0.772** |

⚠️ El 0.844 @ 0.250" está inflado por un dip anómalo de la curva de admisión de este banco a ese lift. **Ratio robusto: 0.77–0.82.** Reemplaza la estimación 0.70±0.10 del v0.1 (era deducción de literatura externa).

**Prueba por-cilindro [D] (independiente):** 61.1 CFM @ 0.100" no pueden ser por puerto — una válvula de 26 mm a 0.100" tiene cortina de 0.3215 in² → Q_max isentrópico = 45.5 CFM < 61.1. **Imposible ⇒ por cilindro.** Esto valida además la interpretación por-cilindro del shootout Bad Guys de admisión.

## 2.2 Cd implícito de escape por válvula [D] vs admisión Y7

Válvula escape 26 mm; Cd = (Q_pub/2) / (141.6 × π·D·L):

| Lift | Cd escape | Cd admisión Y7 | Δ |
|---|---|---|---|
| 0.050 | 0.628 | 0.641 | −0.013 |
| 0.100 | 0.671 | 0.669 | +0.002 |
| 0.150 | 0.667 | 0.704 | −0.037 |
| 0.200 | 0.611 | 0.678 | −0.067 |
| 0.300 | 0.460 | 0.547 | −0.087 |
| 0.400 | 0.353 | 0.425 | −0.072 |
| 0.450 | 0.316 | 0.381 | −0.065 |

**Lectura [D]:** el escape sigue la **misma forma de curva** que la admisión (pico ≈0.67 a bajo lift, decaimiento monótono), pero decae más rápido desde 0.200" — el puerto/garganta de escape estrangula antes. De 0.250"→0.450" el escape solo gana +7.3 CFM vs +22.9 en admisión. Cruce cortina→garganta estimado: con garganta Ø22–24 mm, L* = 0.183–0.218" — coherente con el aplanamiento observado.

## 2.3 Cross-check entre bancos — cota de incertidumbre

Admisión dodo (casero) vs Bad Guys Y7 (SuperFlow): +7.1% @ 0.100", +0.4% @ 0.150", −3.4% @ 0.200", −11.5% @ 0.250" (dip anómalo), −5/−7% en alto lift. **Cota para el dataset de escape: ±10%.** El ratio I/E, al venir del mismo banco y sesión, cancela gran parte del sesgo → más robusto que los CFM absolutos.

## 2.4 Cotas de transferencia por variante (precaución obligatoria)

Admisión Bad Guys, mismo banco, promedio: Y7 140.8 · A6 141.5 (+0.5%) · Y8 142.4 (+1.1%) · **Z6 148.9 (+5.8%)**.

| Variante | Veredicto | Nota |
|---|---|---|
| **D16Y7** (P2A) | **DIRECTA** | Misma fundición P2A-06; ±10% por banco casero |
| D16A6 | **ESCALADA** | Puerto casi idéntico (+0.5% admisión); aplicar curva Y7 ±6%, corregir válvula 25.4 vs 26 mm |
| D16Y8 | **ESCALADA con cautela** | Admisión +1.1% pero arquitectura "swirl" distinta; el escape podría diferir más; ±10% |
| D16Z6 | **ANALOGÍA** | Mejor puerto (+5.8%); la curva Y7 subestima — solo como cota inferior |
| D17A1 | `MISSING-DATA` | — |

Ø válvula escape (`MFR-TECH`): Y7 26 · Y8 26 · Z6 26.4 · A6 25.4 mm. Ø OEM en documento Honda: `MISSING-DATA`. No existe flow de escape público para Z6/Y8/A6/D17A1 ni throat diameters publicados de ninguna variante D (10 búsquedas + 4 aperturas, 2026-10-09).

## 2.5 Reglas de modelado

1. **Fixture utilizable hoy** (`FORUM-MEASURED`, ±10%): tabla Cd(L) de escape §2.2 para válvula de 26 mm; Q_esc = Cd_esc(L)·min(A_cortina, A_garganta)·f(Δp); sobre ~0.20" domina la garganta.
2. A lift de leva stock (~0.300"): Q_esc ≈ 125.7 CFM/cil @ 28" (62.9/válvula).
3. **Prohibido:** usar la curva Y7 para Z6 sin corrección al alza; citar el 0.844 @ 0.250" sin el caveat del dip de banco.

---

# 3. INYECTORES DONANTES

## 3.1 Tabla por PN

| PN / Aplicación | Caudal | ΔP declarada | R | Clase | Transferencia |
|---|---|---|---|---|---|
| `06164-P2A-000` D16Y7 (objetivo) | 180–190 cc/min | **null** | — (`MISSING-DATA`) | `FORUM-CLAIM` | **DIRECTA** (objetivo, incompleto) |
| `06164-P2J-000` D16Y8 | 240 | **null** | — | `FORUM-CLAIM` | **DIRECTA** |
| `06164-P8A-A00` D16Y8 99–00 | 276 | **3 bar, heptano** | 11.7 Ω | `MFR-TECH` vendor | **ESCALADA** (otra página del vendor dice 300: inconsistencia interna) |
| D16Z6 / D15B7 / D16A6 | 235 (tabla DE) / 240 (chart Accel) | **null** | — | `FORUM-CLAIM` | **DIRECTA** (conflicto 235 vs 240 sin ΔP: no promediar) |
| D15B8 / D15B6 (HF) | 180 | **null** | — | `FORUM-CLAIM` | **DIRECTA** |
| B18B1 / B20B4 (saturated) | 240 | **null** | ~12 Ω | `FORUM-CLAIM` | **ANALOGÍA** (B-series, misma época) |
| D17A1 (16450-PLC-003) | ~204 @ 3 bar (escala de potencia [D]) | — | — | `MISSING-DATA` (estimación, no medición) | **ESCALADA** |
| Reman OEM Honda (B7/Z6/B16A3) | — | — | **12.5 Ω** | `MFR-TECH` vendor | **ANALOGÍA** (típico saturated Honda) |

Ningún claim de foro declara ΔP. El patrón 235 (A6/Z6/B7) vs 180–190 (Y7/B8) es consistente entre dos fuentes independientes, pero sin ΔP ninguno es nominal.

## 3.2 Q vs ΔP — confirmación empírica [D]

Bosch 0280158124 (banco, `FORUM-MEASURED`): 340@2.5, **380@3.0**, 410@3.5 bar. Predicción √ΔP desde 380@3.0: @2.5→346.9 vs 340 (**+2.0%**); @3.5→410.4 vs 410 (**+0.1%**). **Q ∝ √ΔP confirmada dentro del 2%: `Q(ΔP) = Q_ref·√(ΔP/ΔP_ref)` utilizable directamente.**

**Ambigüedad intrínseca del claim [D]:** con Q_ref=185 @ 3.0 bar → **168.9 @ 2.5 bar** (ralentí con vacío) y **191.1 @ 3.2 bar**. El claim "180–190 sin ΔP" cubre un **rango intrínseco de ±6%** solo por la presión de referencia. Un rating sin ΔP no es un nominal: es un intervalo.

## 3.3 Deadtime vs voltaje — datos reales publicados

| Fuente | Dato | Clase |
|---|---|---|
| **Honda CBR600RR OEM Keihin 220.66 cc/min, 12.06 Ω** | **1.29@10V · 0.93@12V · 0.74@13V · 0.62@14V · 0.50@15V ms** | `FORUM-MEASURED` — **mejor prior analógico**: OEM Honda, saturated, tamaño cercano al Y7 |
| RC PL4-1200 (aftermarket) | 1.236@10V · 1.087@12V · 0.926@14V · 0.916@16V @43.5 psi; grid V×P completo | `MFR-TECH` — el deadtime **crece con la presión** |
| ID1050X (aftermarket) | 1425@10V · 1125@12V · 925@14V · 795@16V µs @43.5 psid | `MFR-TECH` — confirma dependencia de presión |
| Bosch 0280158124 | @3.0 bar: 0.909@10V · 0.619@12V · 0.405@14V ms; @2.5 bar baja 2–9% | `FORUM-MEASURED` — **la ΔP también entra al modelo** |

**Deadtime OEM del P2A-000: `MISSING-DATA` confirmado** (no es falta de búsqueda). Distinguir siempre: un deadtime de ID/RC **no** es el del P2A-000.

## 3.4 Por qué el deadtime domina a baja carga [D] (verificado)

Con prior analógico CBR600RR (0.62 ms @14V, 0.93 ms @12V):

| Condición | PW efectivo | PW total | Error por ±0.3 ms deadtime |
|---|---|---|---|
| Ralentí | 3.37 ms | 3.99 ms | **±7.5–8.9%** |
| Plena carga | ~12 ms | ~12.6 ms | **±2.4–2.5%** |

- Caída a 12V sin corrección de voltaje → **~8% pobre en ralentí**. La tabla de corrección por voltaje no es opcional.
- Ignorar ΔP con regulador referenciado → **8.7% de error** a ralentí (2.5 vs 3.0 bar).
- **Conclusión [D]:** a ralentí el deadtime (~±8%) y la ΔP real (±8.7%) dominan; el Q nominal es tercer orden. El deadtime del P2A-000 es el `MISSING-DATA` más caro del subsistema.

**Bajo PW:** banco con Link G4 (`FORUM-MEASURED`): lineal hasta ~2.0–2.2 ms; debajo, codo agresivo (short-pulse adder). El PW efectivo del Y7 a ralentí (3.37 ms) está en **zona lineal** — el codo no es preocupación de primer orden en ralentí stock; sí en arranque (voltaje bajo), DFCO e inyectores sobredimensionados.

## 3.5 Prior recomendado para usar hoy

| Parámetro | Prior | Incertidumbre | Etiqueta |
|---|---|---|---|
| Q_nominal | **185 cc/min @ 3.0 bar** | ±6% (ambigüedad ΔP) | `FORUM-CLAIM` provisional |
| Q(ΔP) | **185·√(ΔP/3.0)** | ±2% | [D] confirmada |
| deadtime(V) | **1.29@10 · 0.93@12 · 0.74@13 · 0.62@14 · 0.50@15 ms** | **±0.3 ms** | `ANALOGÍA` (OEM Honda/Keihin 220cc — NO es el P2A-000) |
| deadtime(ΔP) | −2–9% al bajar 3.0→2.5 bar | ±5% | `ANALOGÍA` (Bosch) |
| R_bobina | **12.5 Ω** típico | ±1 Ω (P2A-000: `MISSING-DATA`) | `MFR-TECH`/analogía |
| Low-PW | lineal sobre ~2.2 ms efectivo; adder bajo 2 ms | cualitativo | `FORUM-MEASURED` |

**Regla de modelado:** `PW_total = PW_efectivo + deadtime(V, ΔP)` con `Q = Q_ref·√(ΔP/ΔP_ref)`. **Prohibido:** usar deadtime de ID/RC como si fuera del P2A-000; fijar Q sin ΔP de referencia.

---

# 4. COMBUSTIÓN Y FRICCIÓN DONANTES

## 4.1 BMEP de la familia [D] (verificado)

| Motor | Pico de potencia | Pico de torque |
|---|---|---|
| D16Y7 | **9.62** bar @ 6200 | **11.03** bar @ 4600 |
| D16A6 | 10.13 @ 6000 | 10.71 @ 5000 |
| D15B7 | 10.36 @ 5900 | 11.18 @ 5000 |
| D17A1 | 10.29 @ 6100 | 11.24 @ 4500 |

Dispersión: ±3.7% en pico de potencia (media 10.10), ±2.4% en pico de torque. El Y7 queda ~5% bajo el cluster en pico de potencia — coherente con su leva más suave (limitado por duración, §1). **Ningún BMEP de la familia contradice el rating del Y7.**

## 4.2 p(θ) / IMEP / FMEP en literatura

| Fuente | Dato | Transferencia |
|---|---|---|
| **Pipitone, SAE 2009-01-1984** (1.24L 4-cil NA SI, banco) | FMEP medido **0.5–2.1 bar** @ 1500–4500 rpm; Chen-Flynn error medio 13%, máx 35%; sin relación unívoca FMEP–Pmax | **ESCALADA** — el FMEP medido más cercano al Y7 en literatura abierta |
| MDPI Energies 2020 (0.9L 2-cil **turbo** SI) | IMEP 12.7–17.1 bar alta carga; COV_IMEP <2%, COV_pmax 5–10% | **ANALOGÍA débil** (turbo) |
| MDPI 2019 (Lombardini 2-cil NA, metano, Kistler) | COV_IMEP 1.83%, COV_pmax 7.74% | **ANALOGÍA** — prior de variabilidad cíclica |
| p(θ) publicada de **cualquier D-series** o 4-cil NA 1.5–1.8L gasolina con protocolo | — | `MISSING-DATA` — el hueco es de la familia entera |

**pmax estimada Y7 [D]:** IMEP_bruto ≈ 12 bar × ratio pmax/IMEP 4–5 → **48–60 bar**, pico ~15–16° ATDC. Prior `EXTRAPOLATED`.

## 4.3 BSFC y propagación a inyectores [D]

Fuentes: SciELO 2024 (BSFC **275 g/kWh** real, `ANALOGÍA`); Sierra Research 1999 vía AutoSpeed (mapa de islas 2V NA, `ANALOGÍA`); Eng-Tips (~240 g/kWh @ 8 bar, `FORUM-CLAIM`).

| BSFC | Rating necesario @ 80% duty | Duty con 180 cc/min |
|---|---|---|
| 250 g/kWh | 139.1 cc/min | 61.8% |
| 270 | 150.2 | 66.8% |
| 280 | 155.8 | 69.2% |

**El sizing 180–190 cc/min cierra en todo el rango 250–280** — el BSFC de literatura refuerza, no corrige, la conclusión de inyectores.

## 4.4 Cadena IMEP/BMEP [D]

Con η_mecánica ≈ 81%: Y7 → IMEP_bruto **11.9 bar**; donantes → **12.5–12.8 bar**. La familia pide ~0.6–0.9 bar más de IMEP que el Y7 (leva más suave). **Rango de cordura para el simulador: IMEP_bruto 11.5–13.0 bar a plena carga** — fuera de ahí, el bug está en el modelo. FMEP de Pipitone extrapolado a 6200 rpm ≈ 2.5–2.7 bar, sin contradicción con la estimación propia (2.4 bar @ 6600).

## 4.5 Temperaturas y dynos de donantes

- **EGT con protocolo en la familia D: `MISSING-DATA`.** Solo claims débiles (~1000 °F en colector sin condiciones; 1400–1650 °F en comunidad turbo). Prior [D] para L1: EGT de puerto NA a plena carga **750–900 °C**, `EXTRAPOLATED`.
- **Dyno completo stock: hueco sistémico.** No existe publicado para D16A6, D15B7 ni D17A1 (hilo honda-tech 2007 lo confirma para el D17A1). Los picos de catálogo + BMEP son el sustituto honesto.

---

# 5. FIXTURES UTILIZABLES HOY — actualización vs v0.1

Qué cambió con los donantes (todo con incertidumbre declarada, nada como `calibration_golden`):

| Hueco v0.1 | Estado tras donantes |
|---|---|
| Leva Y7 | **Proxy DIRECTO A6** + rango acotado @.050" [183°,196°], lift [.335",.374"]; IVC por advertised para el compresímetro |
| Flujo de escape | **Fixture nuevo**: Cd_esc(L) §2.2, ±10%; ratio I/E 0.77–0.82 reemplaza 0.70±0.10 |
| Inyectores | **Ley Q(ΔP)** confirmada ±2%; **prior de deadtime analógico** OEM Honda/Keihin ±0.3 ms; R 12.5 Ω ±1 |
| Combustión/fricción | **Envelope familiar BMEP** 9.6–10.4 bar; **rango de cordura IMEP_bruto** 11.5–13.0 bar; FMEP anclado en Pipitone |

Además, durante el research se generaron fixtures legibles por máquina en `workspace/engine_sandbox/research/fixtures/`: `d16y7_intake_cd_vs_lift.yml` (tabla Cd de admisión) y `d16y7_service_fixtures.yml` (fixtures de servicio OEM) — verificar contra este documento antes de fijarlos como datos de producción.

**Sigue prohibido fijar como nominal:** duración/LSA/centerline del Y7, deadtime OEM del P2A-000, flow de escape de Z6/Y8/A6, p(θ), mapas P2E, "100–105 whp stock".

---

# 6. MISSING-DATA PERSISTENTE TRAS DONANTES

- Tabla lift-vs-CA digitalizada (cualquier D non-VTEC) — adquisición física #1 pendiente
- Duración/LSA/centerline del D15B7; centerline del A6
- Throat diameters publicados de cualquier variante D
- Deadtime y resistencia OEM del P2A-000
- p(θ) de la familia D entera
- Dyno completo stock de cualquier D (hueco sistémico)
- Atribución exacta del dato de leva D17A1 (A1 vs A2)

---

# 7. FUENTES CLAVE (URLs directas)

- Leva A6 medida (Mista Bone, 2001): https://honda-tech.com/forums/honda-crx-ef-civic-1988-1991-3/specs-stock-sohc-non-vtec-cam-94750/
- Comparativa A6 vs B7/DX (2003, fundición PM3): https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/d16a6-vs-d15b7-camshaft-626196/page2/?ispreloading=1
- Leva D17 OEM (epcivic, 2009): https://www.epcivic.com/index.php?/topic/4387-d17-cams-in-d16v1/
- Dataset I/E Y7/D14 (dodo-upgrades, 2010): https://dodo-upgrades.nl/phpBB3/viewtopic.php?t=363
- Tabla de inyectores Honda (DE): https://prelude-ba4.de/wp-content/uploads/2020/06/Honda-Einspritzd%C3%BCsen-Durchlass.pdf
- Five-O Motorsport P8A-A00 (@3 bar): https://www.fiveomotorsport.com/honda-accord-civic-12195/
- Q vs ΔP + deadtime V×P (Bosch, ms4x): https://www.ms4x.net (ver reporte bruto)
- Pipitone, SAE 2009-01-1984 (vía academia.edu) · SciELO 2024 (BSFC 275 g/kWh)
- mywikimotors D17A / D15B (specs agregadas): http://mywikimotors.com/honda-d17a/ · http://mywikimotors.com/honda-d15b-d15a/

**Notas de copyright:** cifras = hechos publicados; textos solo resumidos/parafraseados. No redistribuir scans del manual Honda ni fotos de foros como assets.

---

*FIN — D16Y7 Research Donantes v0.1. Reportes brutos: `workspace/engine_sandbox/research/raw/`.*
