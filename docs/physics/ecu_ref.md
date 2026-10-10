# ECU de referencia — S18

## Modelo

Decoder: RPM por mediana de 8 intervalos CKP, sinc CYP (720°) con
respaldo CKP-solo (360°), timeout 3× periodo, CYP contradictorio (≠24
CKP entre CYP) ⇒ SyncLost, fase por dead-reckoning. FSM con fallback
degradado. Mapas bilineales propios (7.7 mg/14° a ralentí;
saturación en borde). Scheduler θ con latencia visible + bitácora.
DTC `REF-*` por persistencia (3/2 ciclos) con curación.

## Verificación (2026-10-10)

Sync 720 + 750 rpm exactas; borde saltado degrada confianza (mediana
resiste amplitud); timeout ⇒ SyncLost; sin CYP ⇒ 360°; SyncLost inhibe;
latencia 1 ms = 4.5° visible; DTC confirma al 3.º y cura al 2.º sano.
Demos fault 0–3 con estados Running/Fallback/CrankingUnsynced + DTC.

## Dominio y deuda

Sin estrategia OEM, sin closed-loop ni trims, sin IACV/PWM, sin CAN/OBD,
códigos no P0xxx, avance fijo por mapa (transitorios S19), inyector con
hipótesis S13. Decoder a régimen constante entre flancos.
