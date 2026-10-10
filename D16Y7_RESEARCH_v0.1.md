# D16Y7 — RESEARCH DE CAMPO Y CONTRASTE TEÓRICO v0.1

**Fecha:** 2026-10-09 · **Naturaleza:** solo research. No rehace ni sustituye el spec (`ENGINE_ENGINEERING_SANDBOX_D16Y7_RESEARCH_FOUNDATION.md`); lo complementa. Los reportes brutos de los 5 frentes viven en `workspace/engine_sandbox/research/raw/`.
**Regla:** ningún número sin fuente + condiciones + clase de evidencia. Los cálculos marcados **[D]** son deducción propia verificada aritméticamente, no datos medidos.

**Clases de evidencia** (las de la adenda 23.1 del foundation): `OEM-TEXT` · `OEM-EPC` · `MFR-TECH` · `FORUM-MEASURED` · `FORUM-CASE` · `FORUM-CLAIM` · `MISSING-DATA`.

---

# 0. RESUMEN EJECUTIVO — qué cambió vs. Foundation v0.1

**Hallazgos nuevos que cierran huecos declarados:**

1. **Flow bench de culata D16Y7 SÍ existe públicamente.** La adenda 23.11 lo declaraba "no encontrado"; se localizó un shootout completo (Bad Guys, 2020) con curvas CFM vs lift @ 28" H₂O para Y7/Y8/Z6/A6 stock y porteadas. Hallazgo físico clave **[D]**: las cifras publicadas son **por cilindro (ambas válvulas)**, no por puerto — demostrado por violación del límite isentrópico bajo la otra interpretación. Para el modelo por-válvula: **dividir entre 2**.
2. **El claim "inyectores Y7 = 180–190 cc/min" cierra con la física.** A 106 hp con BSFC 270 g/kWh se necesitan 150 cc/min @ 80% duty; 180 cc/min implica 66.8% duty — sizing OEM típico. El lore "todos son 240" no cierra (50% duty, sobredimensionado).
3. **El rating de catálogo (106 hp @ 6200) es termodinámicamente coherente.** BMEP 9.62 bar en pico de potencia, 11.03 bar en pico de torque — dentro del rango normal NA SOHC de los 90s. El dyno medido defendible (91.1 whp) cierra con 106 bhp al 12–15% de pérdida; los claims de "100–105 whp stock" **no cierran** (implicarían 114–124 bhp).
4. **El compresímetro nominal (184 psi) implica n politrópico ≈ 1.13, no 1.3.** Con n=1.3, la CR efectiva sería ~7.0:1. La brecha la explican cierre tardío de admisión a 250 rpm + heat transfer + blow-by. Consecuencia: predecir el instrumento exige IVC efectivo + fuga + heat transfer; `CR × p_atm` falla 14–30% por arriba.
5. **Rocker ratio confirmado por 3 fuentes:** 1.60:1 admisión / 1.80:1 escape (`MFR-TECH`). Lash 0.18–0.22/0.23–0.27 mm confirmado por segunda fuente independiente.
6. **Base timing 12° BTDC** con doble confirmación independiente. **Rail 40–47 psi sin vacío / 28–36 psi con vacío** (`OEM-TEXT`).

**Bloqueos estructurales (no es falta de búsqueda, es que no existe públicamente):**

- **No hay dumps ni mapas de la ECU P2E en la comunidad.** Razón estructural: la escena grassroots bypassea OBD2 con conversión a OBD1 (P28 chipeada). La ECU del simulador debe ser de referencia propia, etiquetada no-OEM.
- **Specs stock del árbol Y7 (duración/alzada/LSA): MISSING-DATA.** Hay de A6/Z6/Y8, del Y7 no. Proxy honesto: D16A6 documentado, etiquetado `EXTRAPOLATED`.
- **Flow bench de escape D-series: MISSING-DATA total.** Estimación por ratio I/E 0.70±0.10 como deducción, nunca como fixture.
- **Deadtime vs voltaje del inyector P2A-000: MISSING-DATA** (domina la incertidumbre de PW a ralentí).
- **Curva completa de dyno stock, p(θ), FMEP medido: siguen MISSING-DATA** (confirma el hueco CF59).

---

# 1. FLOW BENCH — CULATAS HONDA D-SERIES

## 1.1 Diámetros de válvula

| Motor | Admisión | Escape | Fuente | Clase |
|---|---|---|---|---|
| D16Y7 | 30 mm (medido por porter) | — | Bad Guys shootout 2020 | `FORUM-MEASURED` |
| D16Y5/Y7/Y8/Z6 | 30 mm std | 26 mm std | Ferrea F5515/F5512, Brian Crower BC3070/BC3071, Supertech ("Standard size") | `MFR-TECH` ×3 |
| D16Z6 | 30.5 mm | 26.4 mm | Endyn (mide culatas para fabricar) | `MFR-TECH` |
| D16A6 | 29 mm (medido) / 29.5 | 25.4 | Bad Guys / Endyn | `FORUM-MEASURED` / `MFR-TECH` |

Conflicto menor registrado: catálogos redondean a 30.0/26.0; Endyn mide 30.5/26.4 en Z6. Diferencia ≤0.5 mm — irrelevante para Cd. Usar 30.0/26.0 como nominal. Diámetro OEM en documento Honda: `MISSING-DATA`.

## 1.2 Flow numbers admisión @ 28" H₂O — stock (Bad Guys, banco SuperFlow)

Cifras **por cilindro** (ver §1.4). Culatas stock con válvulas/asientos de serie.

| Lift (in) | D16Y7 | D16Y8 | D16Z6 | D16A6 |
|---|---|---|---|---|
| 0.050 | 33.66 | 35.84 | 35.60 | 33.90 |
| 0.100 | 70.33 | 72.79 | 73.28 | 71.31 |
| 0.150 | 111.03 | 111.90 | 111.90 | 111.90 |
| 0.200 | 142.41 | 142.25 | 143.58 | 143.56 |
| 0.250 | 163.43 | 159.92 | 168.10 | 163.43 |
| 0.300 | 172.52 | 171.00 | 181.79 | 172.52 |
| 0.350 | 175.63 | 180.25 | 189.49 | 175.63 |
| 0.400 | 178.71 | 183.33 | 192.57 | 178.71 |
| 0.450 | 180.25 | 183.33 | 195.65 | 180.25 |
| 0.500 | 180.25 | 183.33 | 197.17 | 183.33 |
| **Promedio** | **140.82** | **142.39** | **148.91** | **141.45** |

Clase: `FORUM-MEASURED` (porter profesional, protocolo declarado, sin incertidumbre publicada). Caveats: Bad Guys vende culatas CNC (posible sesgo comercial; los baselines stock tienen menos incentivo de inflación); variación banco-a-banco típica ±5%.

Notas del porter: el Y7 es ~clon tardío del A6 con válvula de admisión mayor (30 vs 29 mm) pero **garganta casi igual** — por eso fluye casi lo mismo pese a la válvula más grande. El Y8 usa diseño "swirl" (peor para potencia). El Z6 es la mejor culata stock (+~6% vs non-VTEC).

Porteadas (misma fuente, lifts pares): Y7 DIY +4 CFM prom. (144.7), Y8 DIY +7 (148.9), Z6 DIY 157.5, Z6 pro 159.8–177.5 (Bad Guys CNC). Ganancia casera modesta; pro hasta +26%.

## 1.3 Hallazgo: CFM por cilindro — prueba por límite isentrópico [D]

A 28" H₂O (6974 Pa), el flujo isentrópico máximo es **141.6 CFM por pulg²**; velocidad máxima teórica **103.6 m/s**.

- Si 70.33 CFM @ 0.100" fueran **por puerto** (una válvula de 30 mm): área de cortina = π·1.181·0.100 = 0.371 in² → velocidad implicada **138.7 m/s > 103.6 m/s. Físicamente imposible.**
- A 0.500" por puerto: 180.25 CFM exigiría garganta de Ø ≥ 33.4 mm — **mayor que la propia válvula. Imposible.**
- Cross-check independiente: el mismo patrón (violación del límite bajo interpretación por-puerto) se repite en datasets Volvo B234, Ford Duratec y K24 drag.

**Conclusión [D]: el dataset es por cilindro (ambas válvulas de admisión). Para el simulador: Q_válvula = Q_publicado / 2.**

## 1.4 Contraste teórico: Cd implícito por válvula — D16Y7 stock [D]

Cortina A = π·D_v·L (D_v = 30 mm); Q_max = 141.6 × A(in²); Cd = (Q_pub/2) / Q_max.

| Lift (in) | Q/cil | Q/válv | A cortina (in²) | Q_max teór. | **Cd** |
|---|---|---|---|---|---|
| 0.050 | 33.66 | 16.83 | 0.1855 | 26.3 | **0.641** |
| 0.100 | 70.33 | 35.16 | 0.3711 | 52.5 | **0.669** |
| 0.150 | 111.03 | 55.52 | 0.5566 | 78.8 | **0.704** |
| 0.200 | 142.41 | 71.20 | 0.7421 | 105.1 | **0.678** |
| 0.250 | 163.43 | 81.72 | 0.9276 | 131.4 | **0.622** |
| 0.300 | 172.52 | 86.26 | 1.1132 | 157.6 | **0.547** |
| 0.350 | 175.63 | 87.81 | 1.2987 | 183.9 | **0.478** |
| 0.400 | 178.71 | 89.36 | 1.4842 | 210.2 | **0.425** |
| 0.450 | 180.25 | 90.12 | 1.6697 | 236.4 | **0.381** |
| 0.500 | 180.25 | 90.12 | 1.8553 | 262.7 | **0.343** |

Cross-check: Y8 → 0.693/0.542/0.349; Z6 → 0.697/0.577/0.375; A6 (D_v=29) → 0.702/0.566/0.361 (a 0.100/0.300/0.500). Las cuatro culatas colapsan a la misma curva — coherente con arquitectura de puerto/garganta compartida. **Lectura:** Cd pico ≈ 0.70 @ 0.150", decayendo monótonamente — forma clásica de culata stock: la cortina domina a bajo lift, el puerto/garganta estrangula arriba.

**Cruce cortina→garganta [D]:** igualando A_cortina = A_garganta, con garganta estimada 24–26 mm (testimonio del porter: Y7 ≈ A6), el cruce ocurre a **0.19–0.22" de lift** — exactamente donde la curva medida se aplana. Regla de modelado: Q = Cd(L)·min(A_cortina, A_garganta)·f(Δp), con Cd(L) de esta tabla.

## 1.5 Escape — estado de datos

| Dato | Estado |
|---|---|
| CFM vs lift de escape D-series (cualquier fuente) | **MISSING-DATA** — 6 rondas de búsqueda en foros y porters (Endyn, 4Piston, Portflow, RLZ) sin resultado |
| Cortina teórica escape (26 mm) @ 28" H₂O, por válvula [D] | 0.100"→45.5 · 0.200"→91.1 · 0.300"→136.6 · 0.400"→182.1 · 0.500"→227.7 CFM |
| Estimación [D] | Ratio I/E típico 4V stock: 0.65–0.80 → Q_esc_cil ≈ Q_adm_cil × (0.70 ± 0.10). **Deducción, no medición — no usar como fixture de calibración.** |

---

# 2. LEVAS Y TREN DE VÁLVULAS

## 2.1 Datos confirmados

| Dato | Valor | Fuente | Clase |
|---|---|---|---|
| Rocker ratio admisión / escape | **1.60:1 / 1.80:1** | Crower 63441Z + Brian Crower BC0072 + aritmética Blox Stage 1 (3 fuentes consistentes) | `MFR-TECH` ×3 |
| Lash frío admisión / escape | 0.18–0.22 / 0.23–0.27 mm | Manual Honda Civic 1996 SOHC (CF06) + hoja de builder dkgoodrich (2ª fuente independiente) | `OEM-TEXT` + `FORUM-MEASURED` |
| Cámara Y7 | 34.6 cc (= Z6; Y8 = 32.8) | dkgoodrich | `FORUM-MEASURED` |
| Biela | 137 mm | Arias Pistons | `MFR-TECH` |

## 2.2 Specs stock de la familia D — el Y7 no aparece

| Leva | Adv. dur. (°) | @.050" (°) | Alzada bruta (in) | LSA |
|---|---|---|---|---|
| D16A6 stock | 222/224 | 194/196 | .333/.362 | 110 |
| D16Y8 stock (dkgoodrich) | 211/215/247 ; 235 | 183/187/217 ; 204 | .300/.319/.397 ; .370 | 115/113/114 ; 110 |
| D16Y8 stock (honda-tech 2015) | 288/292 | 218/204 | .397/.367 | — |
| D16Z6 stock (honda-tech 2015) | 304/303 | 212/204 | .394/.373 | — |
| **D16Y7 stock** | — | — | — | — (`MISSING-DATA`) |

**Conflicto metodológico importante:** para el Y8, una fuente da advertised 247° y otra 288°, pero ambas coinciden en @.050" (217–218°) y alzada. La discrepancia es la **altura de medición del "advertised"** (punto de lash vs 0.006"). Regla: advertised sin método declarado no es comparable; el @.050" sí. **Afirmación descartada:** mechanicask.com atribuye al Y7 "246/230°" — dos agregadoras asignan ese valor al Y8 (coincide con su lóbulo VTEC). Probable atribución errónea Y8→Y7. No usar.

## 2.3 Contraste teórico: ¿limita la leva al Y7? [D]

Entradas: B=75 mm, S=90 mm, biela 137 mm, D_válv=30 mm, N=6000 rpm, a≈370 m/s.

| Alzada supuesta | A_cortina | Z medio | Z a v_pico | L/D |
|---|---|---|---|---|
| 7.62 mm (.300", proxy Y8) | 718 mm² | 0.150 | 0.247 | 0.254 |
| 8.46 mm (.333", A6 stock) | 798 mm² | 0.135 | 0.222 | 0.282 |

Choke (Z≈0.5) exigiría alzada < 3.8 mm — margen >2×. A max lift, cortina (718–810 mm²) > garganta del asiento (~531 mm²): **la restricción local es la garganta del puerto, no la cortina**.

**Veredicto [D]:** a 6000 rpm el Y7 stock **no está limitado por área de cortina** (Z≈0.13–0.25 ≪ 0.5). El límite es **time-area: la leva en duración, no en alzada** — leva suave ⇒ VE cae tras el pico de torque a 4600 rpm, coherente con 106 hp @ 6200. Caveat: Cd del puerto es `MISSING-DATA`, pero la conclusión es robusta para alzadas 7.6–9.5 mm.

**Proxy honesto para L1:** perfil stock del **D16A6** (222/224° adv, 194/196° @.050", .333/.362", LSA 110), etiquetado `EXTRAPOLATED` — nunca como dato Y7. **Medición de mayor retorno:** digitalizar un árbol Y7 real con comparador + rueda graduada (alzada vs CA cada 2°) — una tarde, coste ~cero, cierra duración/alzada/LSA/centerline de una vez.

---

# 3. ELECTRÓNICA: ECU P2E, INYECTORES, ENCENDIDO, SENSORES

## 3.1 ECU P2E — lo que la comunidad sabe y el bloqueo estructural

| Hallazgo | Clase | Fuente |
|---|---|---|
| P2E = 96–00 Civic CX/DX/LX non-VTEC OBD2; sufijos ej. 37820-P2E-L02 | `FORUM-CASE` | d-series.org (2006), civicforumz (2005) |
| OBD2a (96–98) y OBD2b (99–00) **no intercambiables** → `EcuVariant` debe incluir subvariante | `FORUM-CASE` | d-series.org |
| **No existen ROM dumps / XDF / mapas P2E en la comunidad** (pgmfi.org, Honda-Tech, Xenocron, Hondata: sin resultados) | `MISSING-DATA` | búsqueda exhaustiva 2026-10-09 |
| Razón estructural: la escena grassroots **solo trabaja OBD1** (P06/P28 chipeadas con Crome/Uberdata); para OBD2 la práctica estándar es conversion harness OBD2→OBD1 | `FORUM-CLAIM` (consenso multisitio) | pgmfi.org (archive), ek9.org, civic5.com |
| Basemaps "D16Y7" de Crome/Hondata = calibraciones **aftermarket**, no P2E stock | `MISSING-DATA` (OEM) | — |

**Implicación:** no hay mapa OEM P2E recuperable. La ECU del simulador debe ser de **referencia propia** con estrategia P2E-like y mapas etiquetados no-OEM. Nunca atribuir un basemap aftermarket a Honda.

## 3.2 Inyectores — part numbers y el claim 180 vs 240

| Parte | Aplicación | Clase |
|---|---|---|
| `06164-P2A-000` (set) | 96–00 DX/CX/LX (D16Y7) | `OEM-EPC`/`FORUM-CASE` |
| `06164-P2J-000` | 96–98 EX (D16Y8) | `FORUM-CASE` |
| `06164-P8A-A00` | 99–00 EX (D16Y8) | `FORUM-CASE`/`MFR-TECH` |

DX/LX/CX comparten PN; EX usa otro → **Y7 ≠ Y8 en inyectores** (debilita el lore "todos 240").

| Claim | Valor | ΔP declarada | Clase |
|---|---|---|---|
| Y7 ≈ 180 cc/min | 180 | **No declarada** | `FORUM-CLAIM` (tabla alemana + honda-tech 2003, 4 fuentes convergen en 180–190) |
| Y7 ≈ 190 cc/min | 190 | **No declarada** | `FORUM-CLAIM` (honda-tech 2008) |
| Y8 (P2J) = 240 cc/min | 240 | **No declarada** | `FORUM-CLAIM` |
| Y8 (P8A-A00) = 276 cc/min | 276 | **3 bar, heptano** | `MFR-TECH` (fiveomotorsport) |
| Y8 (P8A-A00) = 300 cc/min | 300 | **3 bar, heptano** | `MFR-TECH` — **misma vendor discrepa con su propia página** (276 vs 300). No usar sin resolver. |

**Regla de modelado derivada:** todo claim de caudal sin ΔP declarada entra como `FORUM-CLAIM` con `reference_pressure: null`, nunca como nominal.

### Contraste teórico — sizing de inyectores [D] (verificado)

Supuestos: BSFC 270 g/kWh @ plena carga · gasolina 0.74 g/cm³ · 106 hp · ΔP ≈ 3 bar a WOT.

| Paso | Resultado |
|---|---|
| Flujo másico combustible | 355.7 g/min → **480.6 cm³/min** total |
| Por inyector (continuo) | 120.2 cm³/min → **150.2 cm³/min @ 80% duty** |

| Claim | Duty implicado @ 106 hp | Veredicto |
|---|---|---|
| 180 cc/min | **66.8%** | ✅ Cierra — duty ~2/3 es sizing OEM típico |
| 190 cc/min | 63.3% | ✅ Cierra igual |
| 240 cc/min | 50.1% | ⚠️ Sobredimensionado; no es sizing OEM creíble para 106 hp |

**Conclusión [D]: el claim 180–190 cc/min es el único consistente con la física.**

**Ralentí [D]** (750 rpm, MAP 35 kPa abs, T 300 K, VE 0.7, AFR 14.7, inyector 185 cc/min): m_aire = 0.113 g/cil/ciclo → PW efectivo **≈ 3.37 ms** (+ deadtime ~0.8–1.0 ms @14V, **no verificado** → `MISSING-DATA`). Sensibilidad VE 0.6–0.8 → PW 2.9–3.9 ms. El deadtime exacto domina la incertidumbre a ralentí.

Resistencia bobina P2A-000 (Y7): `MISSING-DATA` (P8A-A00: 11.7 Ω `MFR-TECH`). Deadtime vs voltaje OEM: `MISSING-DATA`.

## 3.3 Encendido, combustible, ralentí

| Dato | Valor | Clase | Fuente |
|---|---|---|---|
| Base timing | **12° BTDC** (jumper SCS puesto, motor caliente) | `FORUM-MEASURED` + `FORUM-CLAIM` (doble confirmación) | honda-tech 2017; smogtips cita guía MOTOR |
| Nota | 16°±2° es spec de era OBD1 (Z6); para Y7 OBD2 el spec de servicio es 12° | `FORUM-CLAIM` | mismo Q&A |
| Rail, vacío desconectado | **270–320 kPa (40–47 psi)** | `OEM-TEXT` | doc Honda techinfo |
| Rail, vacío conectado | **200–250 kPa (28–36 psi)** | `OEM-TEXT` | CF07 (manual Honda) |
| Mecanismo [D] | Regulador referenciado a múltiple mantiene ΔP ≈ cte (~43 psi nominal); el rail cae con el vacío. A WOT ΔP ≈ 3 bar → ratings a 3 bar comparables a plena carga | [D] consistente con ambos rangos | — |
| IACV | **3 cables** en cuerpo de mariposa; ralentí objetivo 750±50 rpm | `FORUM-CASE` ×2 | honda-tech 2025/2026 |
| Ralentí M/T USA (caliente, sin carga) | **670 ± 50 rpm** | `OEM-TEXT` (manual Helm 96–00) | thehondaforums |
| Decel fuel cut P2E | Inyectores a **PW=0 sobre 1100 rpm**, mariposa cerrada; reactivan bajo 1100 | `FORUM-MEASURED` (scan tool) | ecomodder |
| O2 narrowband | 0–1 V, conmuta ~0.45 V; sin señal → open loop | `MFR-TECH` (física de sensor + OBD2 genérico) | — |
| Datalogs/trims y condiciones de closed-loop del P2E | — | `MISSING-DATA` | — |
| Curva duty→caudal IACV, frecuencia PWM | — | `MISSING-DATA` | — |

**Nota de modelado [D]:** el O2 debe modelarse como sensor conmutado con retardo térmico + heater, no como "AFR medido" — un narrowband no mide lambda, solo indica rico/pobre.

---

# 4. FRICCIÓN Y COMBUSTIÓN — PRIORS DE LITERATURA

Honestos como priors `EXTRAPOLATED`, nunca como calibración D16Y7.

## 4.1 Chen-Flynn (FMEP)

Forma `FMEP = c1 + c2·pmax + c3·Sp + c4·Sp²` confirmada (Chen & Flynn, SAE 650625, 1965 — desarrollado en motor de encendido por compresión; **su uso en SI es extrapolación consagrada**). **No existe en literatura abierta un set canónico de coeficientes para SI pequeño** — son ajustes por motor. Coeficientes D16Y7: `MISSING-DATA`.

Dato incómodo y valioso: Pipitone (SAE 2009-01-1984, 1.24L SI medido en banco) ajustó Chen-Flynn y publicó el error: **13% medio, 35% máximo (0.668 bar)**. Y en ese motor **no hay relación unívoca FMEP–Pmax** (el FMEP a veces sube al bajar Pmax) — **el término de carga es el eslabón débil**. FMEP medidos en SI pequeños: 0.5–2.1 bar (1500–4500 rpm).

**Estimación anclada [D]** (Sp = 2·carrera·N/60; carrera 90 mm → 9.0 m/s @ 3000, 19.8 m/s @ 6600):

| Régimen | FMEP supuesto | Pf resultante | Incertidumbre |
|---|---|---|---|
| 3000 rpm | 1.2 bar | **4.8 kW** | ±35% (Pipitone) |
| 6600 rpm | 2.4 bar | **21.0 kW** | ±35% |

Cadena de cordura **[D]**: 106 hp @ 6200 → BMEP 9.62 bar → IMEP_bruto ≈ 12.1 bar, η_mecánica ≈ 81%. **Test de cordura para el simulador: si con priors honestos el BMEP a plena carga sale fuera de 9–11 bar, el bug está en el modelo.**

## 4.2 Woschni (heat transfer)

`h = 129.8·B^−0.2·p^0.8·T^−0.55·w^0.8` (p en bar); `w = C1·vp + C2·(Vd·Tr/(pr·Vr))·(p−pm)`; C1 = 2.28+0.308·(cu/vp) en ciclo cerrado, 6.18+0.417·(cu/vp) en intercambio; C2 = 3.24×10⁻³ (original diésel — **recalibrar en SI**). Alternativa Hohenberg: `h = 130·V^−0.06·p^0.8·T^−0.4·(vp+1.4)^0.8` — la literatura confirma que Woschni sobreestima el pico. Paredes como prior publicado: 450/400/500 K (culata/camisa/pistón).

## 4.3 Wiebe (combustión)

`a=5, m=2` = default de libro de texto (a=6.908 ↔ 99.9% completitud). Duraciones publicadas: 30–90 °CA total; 40–60 °CA ángulo de quemado; fast-burn moderno a carga parcial 0–90% ≈ 35 °CA; Δθ ∝ rpm^0.2. MBT: pmax ≈ 15–16° ATDC. **Prior propuesto D16Y7: Δθ = 50–60 °CA plena carga, 60–70 °CA parcial, etiquetado `EXTRAPOLATED`.**

## 4.4 Livengood-Wu (knock)

Incertidumbre publicada: cinética detallada predice borde de knock con error medio 1 °CA / máx 3 °CA de avance; se degrada con combustibles NTC (doble etapa/cool flame); el supuesto clásico "no hay knock tras un MFB fijo" es falso (SAE 2018-01-0855). **Uso honesto: predictor de tendencia con 2–3 °CA de incertidumbre, jamás para decretar daño.**

## 4.5 Eficiencia Otto [D] (verificado)

CR 9.4:1 → η = **59.2%** (γ=1.4) / **54.3%** (γ=1.35). PMEP ilustrativo: ≈ −0.1 bar a WOT, ≈ −0.55 bar a crucero, ≈ −0.75 bar en ralentí (`PMEP ≈ pe−pi`; magnitudes medidas: `MISSING-DATA`).

---

# 5. DYNO Y DATOS DE CAMPO

## 5.1 Specs de catálogo (verificación cruzada, OEM pendiente en S01)

| Parámetro | Valor | Clase |
|---|---|---|
| Potencia | 106 hp (79.0 kW) @ **6200 rpm** | `FORUM-CLAIM` (consenso de 5 agregadores) |
| Torque | 103 lb-ft (140 N·m) @ **4600 rpm** | `FORUM-CLAIM` |
| CR / Bore×Stroke | 9.4:1 / 75×90 mm (1590.4 cm³) | `FORUM-CLAIM` |
| Redline / rev limiter | 6800 / 7200 rpm | `FORUM-CLAIM` |

Conflictos (no promediar): un post afirma 6400 rpm y limiter 8000 (contradice consenso); una agregadora publica carrera 89.4 mm (errata); otra inconsistente en rpm de torque (4300 vs 4600).

## 5.2 Dyno stock — el baseline que cierra

| Dato | Valor | Condiciones | Clase | Fuente |
|---|---|---|---|---|
| Baseline | **91.1 whp / 92.3 wtq** | Test de paquete de piezas ~2001 citado en 2011; dyno/marcha/corrección **no reportados** | `FORUM-MEASURED` | honda-tech 2011 |
| Bolt-ons (mismo test) | AEM 97.1 → Tanabe 101.8 → header **109.8 whp** | idem | `FORUM-MEASURED` | misma |
| Claim "100–105 whp stock, curva plana" | sin protocolo | — | `FORUM-CLAIM` | carphrases 2025 |

**Curva completa whp/wtq vs rpm: NO ENCONTRADA = `MISSING-DATA`** (confirma hueco CF59 del foundation).

### Contraste teórico [D] (verificado)

**BMEP** (Vd = 1.5904 L): 106 hp @ 6200 → **9.62 bar**; 103 lb-ft @ 4600 → **11.03 bar**. Ambos dentro del típico 10–12.5 bar NA SOHC 90s. Velocidad media de pistón: 18.6 m/s @6200, 21.6 m/s @7200 — coherente con serie.

**whp → bhp:**

| whp | ÷0.85 (15%) | ÷0.88 (12%) | ¿Cierra con 106? |
|---|---|---|---|
| **91.1** (medido) | **107.2** | 103.5 | ✅ ±2.5% |
| 100 (claim) | 117.6 | 113.6 | ❌ overshoot 8–11% |
| 105 (claim) | 123.5 | 119.3 | ❌ overshoot 13–16% |

**Lectura [D]: el dato que cierra es 91.1 whp.** "100–105 whp stock" implicaría 114–124 bhp → dyno optimista o bolt-ons no declarados. Caveat metodológico: la pérdida % no es fija (varía con torque/rpm/marcha) y calcularla asumiendo el bhp de catálogo es razonamiento circular — usar como envelope, no como identidad.

**Compresión politrópica [D]:** `p2 = 101 × 9.4^n`:

| n | p2 | vs nominal 1270 kPa (184 psi) |
|---|---|---|
| 1.2 | 1485 kPa (215 psi) | medición **14% por debajo** |
| 1.3 | 1856 kPa (269 psi) | 32% por debajo |
| **n implicado por nominal** | — | **n ≈ 1.13** |
| n implicado por mínimo (930 kPa) | — | **n ≈ 0.99** |

Con n=1.3, el nominal implicaría **CR efectiva ≈ 7.0:1**. La brecha: (1) cierre tardío de admisión a 250 rpm de arranque, (2) heat transfer en arranque lento, (3) blow-by. **Consecuencia para el simulador: el modelo del compresímetro debe incluir IVC efectivo + heat transfer + fuga; `CR × p_atm` falla 14–30% por arriba.**

## 5.3 Datos OEM de servicio adicionales (fixtures listos para S01/S24)

| Dato | Valor | Condiciones | Clase | Fuente |
|---|---|---|---|---|
| Aceite mín. | **70 kPa (10 psi)** @ ralentí · **340 kPa (50 psi)** @ 3000 rpm | Aceite 80 °C | `OEM-TEXT` | doc técnica Honda (mirror PDF techinfo) |
| Juego rotor bomba aceite | 0.02–0.14 mm nuevo / 0.18 mm límite | — | `OEM-TEXT` | misma |
| Compresión | **1270** / 930 / 200 kPa nominal/mín/var-máx (184/135/28 psi) | 250 rpm, WOT | `OEM-TEXT` | Service Manual 96–98 (mirror PDF) |
| Bobina (distribuidor) | primario 0.63–0.77 Ω · secundario 12.8–19.2 kΩ | @ 68 °F | `FORUM-CLAIM` | honda-tech |

## 5.4 Casos de diagnóstico nuevos (no están en el foundation)

| ID | Síntoma → Medición → Causa | Estado | Clase |
|---|---|---|---|
| CASE-D01 | P0301+P0302; cables **medían bien en R-DC**, todo nuevo, inyectores cruzados sin cambio → **cables nuevos lo curaron** | Autodeclarada | `FORUM-CASE` |
| CASE-D02 | Misfire + P0420 + header rajado; tras reemplazar manifold/ICM/bujías "running good" | Causa ambigua | `FORUM-CASE` |
| CASE-D03 | **Misfire aleatorio solo en ralentí**, todo el encendido nuevo; MIL se autoapagó pero el miss **persiste** | **Sin resolver** | `FORUM-CASE` |
| CASE-D04 | Parado 3 años: cilindros 1-2 muertos, hay chispa, inyectores OK → hipótesis **válvulas pegadas en guías** | Sin resolución | `FORUM-CASE` |
| CASE-D05 | Arranque intermitente; **main relay PGM-FI clickea pero no arranca** (soldaduras agrietadas) | Consejo experto | `FORUM-CASE` |
| CASE-D06 | Sin chispa: **la bobina (dentro del distribuidor) falla antes que el ignitor** | Consejo moderador | `FORUM-CASE` |
| CASE-D07 | **P1399** + P0300–P0304 (misfire específico Honda) | Artículo dealer | `FORUM-CLAIM` |

**Lecciones de modelado:** D01 → R-DC del cable no predice integridad bajo pulso HV (separar R_dc de dieléctrico/intermitencia). D05 → relay que "clickea" puede no conducir (modelar estado de contacto, no solo bobina). D03 → admitir diagnósticos **no concluyentes**. D04 → *storage degradation* (válvulas pegadas) como condición inicial del slow-state, no solo desgaste por uso.

---

# 6. CONTRASTE TEÓRICO CONSOLIDADO — predicción vs medición

| Fenómeno | Teoría ingenua | Medición / literatura | Brecha y lectura [D] |
|---|---|---|---|
| Compresión de arranque | CR×p_atm con n=1.3 → 269 psi | 184 psi nominal | **n≈1.13**; IVC tardío + heat transfer + blow-by. El instrumento debe modelarse, no imponerse. |
| Potencia de catálogo | — | 106 hp → BMEP 9.62/11.03 bar | Coherente con NA SOHC 90s. Sin inconsistencia termodinámica. |
| Dyno stock | — | 91.1 whp medido | Cierra con 106 bhp al 12–15%. Claims 100–105 whp no cierran. |
| Inyectores | — | claim 180–190 cc/min | Duty 63–67% @ 106 hp ✅. 240 cc/min → 50%, no-OEM. |
| Flujo culata | cortina ideal | Cd 0.70→0.34 por válvula | Culata stock limitada por garganta/puerto sobre ~0.22" lift. |
| Leva | — | Z≈0.13–0.25 @ 6000 | No limita por área; limita por duración (time-area). |
| FMEP | Chen-Flynn | 1.2–2.4 bar est. (±35%) | Pf 4.8–21 kW. Término de carga = eslabón débil (Pipitone). |
| Combustión | Wiebe a=5,m=2 | Δθ 50–60° WOT (prior) | Etiquetar `EXTRAPOLATED` hasta p(θ) real. |

---

# 7. MISSING-DATA CONSOLIDADO Y PRIORIDADES DE ADQUISICIÓN

| # | Hueco | Retorno | Vía más barata |
|---|---|---|---|
| 1 | **Perfil de leva Y7** (duración/alzada/LSA/centerline) | Cierra S11 de una vez | Digitalizar árbol real: comparador + rueda graduada, una tarde, coste ~0 |
| 2 | **Flow de escape D-series** (hueco público más grande) | Completa S12 | Banco propio si hay acceso a culata; medir Ø garganta adm/esc de paso |
| 3 | **Banco de inyectores P2A-000**: Q(ΔP) + deadtime vs V | Domina incertidumbre de PW a ralentí/baja carga | Banco de inyectores (servicio comercial) o caracterización propia |
| 4 | **Compresión + leak-down protocolizados** en motor real | Valida el modelo del instrumento (S24) | Motor de junkyard + compresímetro + leak-down tester |
| 5 | **CKP/TDC/CYP en osciloscopio + datalog** (MAP/IAT/ECT/TPS/O2) | Cierra S17/S19 | Motor en marcha + scope de 2 canales |
| 6 | **Dyno stock con protocolo completo** (marcha, corrección, ambiente) | Hueco #1 para S25 | Comunidad con dyno + D16Y7 stock verificado |
| 7 | **p(θ) con transductor in-cylinder** | El salto de identificabilidad (burn model) | Banco instrumentado / colaboración universitaria |

---

# 8. FIXTURES UTILIZABLES HOY (no-calibración, con incertidumbre declarada)

Pueden entrar al simulador **hoy** como `FORUM-MEASURED`/`MFR-TECH`/`OEM-TEXT` con su incertidumbre — plausibilidad y tests adversariales, nunca como `calibration_golden`:

- **Cd(L) admisión stock Y7** (tabla §1.4), ±7% banco-a-banco + sesgo de vendor desconocido; Q por válvula = Q_pub/2; restricción = min(cortina, garganta) sobre ~0.22".
- **Tren de válvulas:** rocker 1.60/1.80, lash 0.18–0.22/0.23–0.27 mm frío, válvulas 30/26 mm.
- **Encendido/combustible:** base timing 12° BTDC (jumper); rail 40–47 psi sin vacío / 28–36 con vacío; inyectores 180–190 cc/min con `reference_pressure: null` hasta banco.
- **Servicio:** ralentí 670±50 rpm M/T; aceite 10/50 psi @ 80 °C; compresión 1270/930/200 kPa @ 250 rpm WOT; decel fuel cut >1100 rpm; bobina 0.63–0.77 Ω / 12.8–19.2 kΩ.
- **Dyno:** 91.1 whp / 92.3 wtq como envelope de plausibilidad (no curva).
- **Priors:** Chen-Flynn (sin coeficientes canónicos — anclar en Pipitone ±35%), Woschni (recalibrar C2 en SI), Wiebe a=5/m=2 Δθ 50–60° WOT, Livengood-Wu como tendencia (±2–3 °CA).

**Prohibido fijar como nominal:** specs de leva Y7 (usar proxy A6 `EXTRAPOLATED`), flow de escape (ratio 0.70±0.10 es deducción), deadtime (inventar 0.9 ms contamina todo el ralentí), mapas P2E (no existen), "100–105 whp stock".

---

# 9. FUENTES CLAVE (URLs directas)

- Bad Guys, "D SERIES CYLINDER HEAD SHOOT OUT!" (2020-04-13): https://the-bad-guys.com/2020/04/13/d-series-cylinder-head-shoot-out/?v=84de8e2b14bb
- Endyn, tabla de válvulas: http://www.theoldone.com/components/valves/
- Ferrea STD D16 (30/26 mm): https://www.realstreetperformance.com/ferrea-competition-plus-valves-std-honda-d16z6-y5-y7-y8.html
- Chen & Flynn, SAE 650625 (1965) · Pipitone, SAE 2009-01-1984 · Heywood, *Internal Combustion Engine Fundamentals*
- Honda techinfo mirrors (aceite, fuel pressure, service manual 96–98): http://downloads.laboratoryb.org/insight/other_manuals/Specs/Engine/
- honda-tech: baseline dyno (2011) https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/what-kind-power-should-i-expect-out-modded-d16y7-2965368/page2/ · base timing (2017) https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/ignition-timing-d16y7-3304844/ · inyectores (2003/2008) https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/stock-d-series-injector-size-656591/
- Casos D01–D07: hilos honda-tech 2004–2010, justanswer, westwoodhonda.com (ver reportes brutos)
- Specs agregadas (contraste, OEM pendiente): https://www.nthefastlane.com/honda-d-series-engine-specs-info · https://hondatheotherside.com/d16y7/

**Notas de copyright:** los números son hechos medidos publicados; los textos solo se resumen/parafrasean. No redistribuir scans del manual Honda ni fotos de foros como assets.

---

*FIN — D16Y7 Research v0.1. Reportes brutos: `workspace/engine_sandbox/research/raw/`.*
