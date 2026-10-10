# Tabla de discrepancias — S01

Política: no sobrescribir OEM; crear ConflictRecord; si no se explica por
régimen/T/variante/instrumento, bloquear calibración. No promediar.

| # | Parámetro | Claim A | Claim B | Condiciones A/B | Decisión S01 |
|---|---|---|---|---|---|
| C-01 | compresión | 184 psi nominal | 135 psi mínimo | 250rpm WOT ambas (CF01) | Campos distintos, no promedio |
| C-02 | compresión campo | 148–153 / 187–197 psi (CF09/CF10) | 120 psi sin WOT (CF11) | protocolo distinto | Método afecta fixture; registrar condiciones |
| C-03 | rail | foro 34–36 vac (CF13) | manual 28–36 vac (CF07) | vacío conectado | Priorizar OEM + curva regulador; REQUIRES_VERIFICATION |
| C-04 | rail vac-off | 38–46 psi (CF07) | 40–47 psi (doc base) | ralentí vac-off | REQUIRES_VERIFICATION edición |
| C-05 | inyectores | Y7 180–190 sin ΔP | Y8 240 sin ΔP (CF14) | ΔP null ambas | PENDING banco; reference_pressure:null |
| C-06 | inyectores vendor | P8A-A00 276 @3bar | P8A-A00 300 @3bar (mismo vendor) | heptano vs ? | No usar sin resolver |
| C-07 | A6 valve lift | .333/.362 | .374/.353 medido | rocker distinto | Fijar lobe .225/.193 FORUM-MEASURED; valve=[D] |
| C-08 | Y8 advertised | 247° | 288° | altura medición distinta | Solo @.050 comparable |
| C-09 | Y7 specs leva | (ausente) | 246/230 atribuido (mechanicask) | probable Y8→Y7 erróneo | Descartado; MISSING-DATA |
| C-10 | distribuidor | TD-80U P2E-A02 | TD-98U P2F-A01 (CF19) | distinta aplicación | DISPUTED; congelar mercado/sufijo |
| C-11 | ECU | P2E genérico | sufijos mercado/transmisión (CF48–CF50) | OBD2a vs OBD2b | ID completo obligatorio |
| C-12 | aceite ralentí | foro 5–10 psi caliente (CF61) | manual mín 10 psi@80°C (CF02) | desgaste/sensor? | Caso límite, no normal OEM |
| C-13 | torque pernos | torque correcto pero rosca gira (CF24) | clamp esperado | rosca dañada | Torque ≠ precarga; diagnosticar stripping |
| C-14 | PTW | 0.003–0.004 in reportado (CF22) | (sin límite OEM) | medición usuario | No límite servicio |
| C-15 | dyno stock | 91.1 whp medido sin protocolo | 100–105 whp claim / 117 whp bolt-ons | protocolo ausente | Envelope, no curva; prohibido fijar 100–105 |
| C-16 | potencia catálogo | 106hp@6200 / 103lb-ft@4600 | variantes 6400/8000, 4300, 89.4mm | agregadores | FORUM-CLAIM; OEM pendiente |
| C-17 | I/E escape | ratio 0.77–0.82 P2A-06 | 0.844@0.250 (dip banco) | banco casero | Citar con caveat; cota ±10% propuesta |
| C-18 | bore სიტვ | 75×90 consenso | 89.4 carrera errata | agregadora | No promediar; OEM pendiente |
| C-19 | trigger 24/4/1 | hondabase+rusEFI: 24/4/1 a ½ velocidad (95+) | TD-80U/98U no nombrados; 16° es era OBD1 | Fuerte candidato familiar; GENERIC_TRIGGER, prohibido OEM hasta scope+página |
| C-20 | CKF | rusEFI: separado en bomba, 12 dientes, algunos OBD2 USA (serie B) | D16Y7-1997 sin confirmar | PENDING presencia; concepto registrado |

Matriz CONFIRMED / DISPUTED / UNKNOWN: nada es CONFIRMED en S01
(edición/página pendientes). DISPUTED = tabla arriba. UNKNOWN =
`unverified_values.csv`.
