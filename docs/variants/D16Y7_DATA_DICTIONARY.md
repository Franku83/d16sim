# D16Y7 — Diccionario de datos maestro (S01)

Variante: `US_1997_CIVIC_DX_2DR_5MT_OBD2A_P2E_FAMILY`
(`data/d16y7/reference_variant.yaml`).
Regla S01: **cero numéricos esenciales sin fuente o etiqueta PENDING**.
Canales: `nominal` · `service_limit` · `measurement` · `distribution` ·
`unknown`. Los service limits **no** son tolerancias de fabricación.
Ningún candidato abajo es `[F]` definitivo: falta edición/página Honda.

## 1. Geometría y catálogo (identidad EPC, sin dinámica)

| parameter_id | candidate (NO nominal) | unit | channel | class | status | source |
|---|---|---|---|---|---|---|
| d16y7.identity.head_assembly | 12100-P2A-000 | — | nominal-identity | OEM-EPC | IDENTITY_ONLY | CF15 |
| d16y7.identity.head_gasket | 12251-P2J-004 | — | nominal-identity | OEM-EPC | IDENTITY_ONLY | §23.3 |
| d16y7.identity.head_bolt | 90008-PZ1-003 x10, 10x120.5 washer | — | nominal-identity | OEM-EPC | IDENTITY_ONLY | §23.3 |
| d16y7.identity.piston_std | 13010-P2E-000 | — | nominal-identity | OEM-EPC | IDENTITY_ONLY | CF16 |
| d16y7.identity.piston_os | 13020-P2E-000 (+0.25), 13030-P2E-000 (+0.50) | mm | nominal-identity | OEM-EPC | IDENTITY_ONLY | §23.3 |
| d16y7.identity.ring_set | 13011-P07-004 RIKEN STD (variantes proveedor/mercado) | — | nominal-identity | OEM-EPC | IDENTITY_ONLY | §23.3/CF16 |
| d16y7.identity.injector_set | 06164-P2A-000 x4 | — | nominal-identity | OEM-EPC | IDENTITY_ONLY | CF17 |
| d16y7.identity.fuel_regulator | 16740-P2E-A01 | — | nominal-identity | OEM-EPC | IDENTITY_ONLY | §23.3 |
| d16y7.identity.intake_manifold | 17100-P2F-A00 | — | nominal-identity | OEM-EPC | IDENTITY_ONLY | §23.3 |
| d16y7.identity.oil_pump | 15100-P7A-013 | — | nominal-identity | OEM-EPC | IDENTITY_ONLY | CF18 |
| d16y7.identity.oil_relief | 15231-PM3-000 + spring 15232-PM3-000 | — | nominal-identity | OEM-EPC | IDENTITY_ONLY | §23.3 |
| d16y7.identity.distributor_A | 30100-P2E-A02 (TD-80U) | — | nominal-identity | OEM-EPC | DISPUTED_VS_B | CF19 |
| d16y7.identity.distributor_B | 30100-P2F-A01 (TD-98U) | — | nominal-identity | OEM-EPC | DISPUTED_VS_A | CF19 |
| d16y7.identity.water_pump | 19200-P2A-A01 | — | nominal-identity | OEM-EPC | IDENTITY_ONLY | CF20 |
| d16y7.identity.thermostat | 19301-P08-316 / reemplazo 19301-PAA-306 | — | nominal-identity | OEM-EPC | IDENTITY_ONLY | CF20 |
| d16y7.geometry.bore | 75 (FORUM-CLAIM) | mm | nominal | FORUM-CLAIM | PENDING_VERIFICATION | research v0.1 §5.1 |
| d16y7.geometry.stroke | 90 (FORUM-CLAIM) | mm | nominal | FORUM-CLAIM | PENDING_VERIFICATION | research v0.1 §5.1 |
| d16y7.geometry.displacement | 1590.4 [D] si bore/stroke confirmados | cm3 | nominal | [D] | PENDING_VERIFICATION | cálculo |
| d16y7.geometry.compression_ratio | 9.4 (FORUM-CLAIM) | — | nominal | FORUM-CLAIM | PENDING_VERIFICATION | research v0.1 §5.1 |
| d16y7.geometry.firing_order | 1-3-4-2 [P] | — | nominal | [P] | PENDING_VERIFICATION | Foundation §2.1 |
| d16y7.geometry.valve_dia_int/exh | 30.0/26.0 nominal catálogos (Ø OEM en doc Honda: MISSING-DATA) | mm | nominal | MFR-TECH | PENDING_VERIFICATION | research v0.1 §1.1 |
| d16y7.geometry.rod_length | 137 (Arias) | mm | nominal | MFR-TECH | PENDING_VERIFICATION | research v0.1 §2.1 |
| d16y7.geometry.combustion_chamber | 34.6 (=Z6; Y8 32.8, dkgoodrich) | cc | measurement | FORUM-MEASURED | PENDING_VERIFICATION | research v0.1 §2.1 |

## 2. Servicio OEM candidato (reproducciones HTML; edición/página por confirmar)

| parameter_id | candidate | conditions | channel | class | status | source |
|---|---|---|---|---|---|---|
| d16y7.service.compression_nominal | 1270 kPa / 184 psi | 250 rpm, WOT | service-nominal-instrument | OEM-TEXT-candidate | PENDING_VERIFICATION | CF01 |
| d16y7.service.compression_min | 930 kPa / 135 psi | idem | service_limit | OEM-TEXT-candidate | PENDING_VERIFICATION | CF01 |
| d16y7.service.compression_spread_max | 200 kPa / 28 psi entre cilindros | idem | service_limit | OEM-TEXT-candidate | PENDING_VERIFICATION | CF01 |
| d16y7.service.oil_p_idle | 69 kPa / 10 psi mín. | aceite 80 °C, ralentí | service_limit | OEM-TEXT-candidate | PENDING_VERIFICATION | CF02/CF03 |
| d16y7.service.oil_p_3000 | 340 kPa / 50 psi mín. | aceite 80 °C, 3000 rpm | service_limit | OEM-TEXT-candidate | PENDING_VERIFICATION | CF02/CF03 |
| d16y7.service.main_clearance_J1J5 | 0.018–0.036 nuevo; límite 0.05 | por bancada | nominal+limit | OEM-TEXT-candidate | PENDING_VERIFICATION | CF04 |
| d16y7.service.main_clearance_J2J3J4 | 0.024–0.042 nuevo; límite 0.05 | por bancada | nominal+limit | OEM-TEXT-candidate | PENDING_VERIFICATION | CF04 |
| d16y7.service.rod_clearance | 0.020–0.038 nuevo; límite 0.05 | por muñequilla | nominal+limit | OEM-TEXT-candidate | PENDING_VERIFICATION | CF05 |
| d16y7.service.lash_intake_cold | 0.18–0.22 | motor frío | nominal | OEM-TEXT-candidate | PENDING_VERIFICATION | CF06 |
| d16y7.service.lash_exhaust_cold | 0.23–0.27 | motor frío | nominal | OEM-TEXT-candidate | PENDING_VERIFICATION | CF06 |
| d16y7.service.fuel_p_vac_off | 260–310 kPa / 38–46 psi (rango taller citado; doc base 270–320/40–47) | ralentí, vacío desconectado | nominal-range | OEM-TEXT-candidate | REQUIRES_VERIFICATION | CF07/research §3.3 |
| d16y7.service.fuel_p_vac_on | 200–250 kPa / 28–36 psi | ralentí, vacío conectado | nominal-range | OEM-TEXT-candidate | PENDING_VERIFICATION | CF07 |
| d16y7.service.cam_holder_clearance | 0.050–0.089 nuevo (publ. DX 1999) | — | nominal | OEM-TEXT-candidate | PENDING_VERIFICATION | CF08 |
| d16y7.service.idle_mt_hot | 670±50 rpm | caliente, sin carga, M/T | nominal | OEM-TEXT-candidate | PENDING_VERIFICATION | research v0.1 §3.3 |
| d16y7.service.base_timing | 12° BTDC (jumper SCS, caliente) | ajuste | nominal | FORUM-MEASURED+CLAIM | PENDING_VERIFICATION | research v0.1 §3.3 |
| d16y7.service.coil_primary/secondary | 0.63–0.77 Ω / 12.8–19.2 kΩ @68°F | — | nominal | FORUM-CLAIM | PENDING_VERIFICATION | research v0.1 §5.3 |
| d16y7.service.decel_fuel_cut | PW=0 sobre 1100 rpm, mariposa cerrada | — | nominal | FORUM-MEASURED | PENDING_VERIFICATION | research v0.1 §3.3 |

Presión de compresión ≠ `CR × atm`: validar como instrumento (IVC + fuga +
heat transfer + protocolo). Presión rail: tipar `p_rail_abs`,
`p_manifold_abs`, `Δp_injector`. Galería ≠ película local.

## 3. Señales / ECU (existencia vs parámetros)

| parameter_id | estado S01 |
|---|---|
| d16y7.ecu.family_P2E | FORUM-CASE; full-ID+sufijo PENDING; mapas/ROM MISSING-DATA; política P2E-like-no-OEM |
| d16y7.ecu.obd | OBD2a (1997); OBD2b no intercambiable (FORUM-CASE) |
| d16y7.trigger.CKP_TDC_CYP | existencia en manual aplicable [F-existencia]; R/L/amplitud/forma/dientes/polaridad/estrategia [P]; resistencia DC ~331–345 Ω es FORUM-MEASURED y NO prueba señal útil |
| d16y7.trigger.CKF | presencia por año PENDING; no inferir |
| d16y7.sensors.MAP_TPS_IAT_ECT_O2 | PENDING (tablas/curvas/wiring pendientes) |
| d16y7.actuators.IACV | 3 cables candidato (FORUM-CASE x2); curva duty→caudal y PWM MISSING-DATA |
| d16y7.actuators.injector_P2A | identidad EPC; Q sin ΔP = FORUM-CLAIM con reference_pressure:null; Q_nominal 185@3bar = hipótesis, NO medición; deadtime y R OEM MISSING-DATA |
| d16y7.ignition.distributor | TD-80U vs TD-98U DISPUTED; avance base ≠ avance dinámico |

## 4. Proxies explícitamente NO nominales (sensibilidad futura, nunca S01)

Leva Y7 stock (duración/alzada/LSA/centerline): MISSING-DATA; proxy A6
EXTRAPOLATED. Flow escape: fixture comunitario P2A-06 ±10% propuesto, no
OEM. Cd admisión Y7: FORUM-MEASURED con incertidumbre banco-a-banco.
Deadtime analógico CBR600RR: ANALOGÍA, no P2A-000. Priors
Wiebe/Woschni/Chen-Flynn: EXTRAPOLATED. Dyno 91.1 whp: envelope sin
protocolo, no curva. "100–105 whp stock": prohibido fijar.
