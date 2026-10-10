# Protocolo de adquisición — S01 (lo faltante)

Ordenado por retorno (research v0.1 §7). Ninguno bloquea S00 ni un L1
honesto; todos bloquean su calibración correspondiente.

1. Perfil leva Y7 (duración/alzada/LSA/centerline): comparador + rueda
   graduada cada 2° en árbol real. Cierra S11.
2. Flow escape + gargantas adm/esc: banco propio si hay culata. Completa S12.
3. Banco inyectores P2A-000 Q(ΔP) + deadtime vs V: servicio comercial o
   caracterización propia. Domina PW ralentí. S13.
4. Compresión + leak-down protocolizados en motor real (250rpm WOT,
   rpm/batería/T/instrumento registrados). Valida S24.
5. CKP/TDC/CYP en scope + datalog MAP/IAT/ECT/TPS/O2. Cierra S17/S19.
6. Dyno stock con protocolo (marcha/corrección/ambiente/combustible). S25.
7. p(θ) con transductor in-cylinder. Salto de identificabilidad. S25.

## Campaña mínima con D16Y7 funcional (cuando haya acceso; no asumir)

Placa/motor/ECU/distribuidor + market/trim/manual; compresión por cilindro
(250rpm+WOT); leak-down con rutas; CKP/TDC/CYP + pulsos a
arranque/ralentí/RPM; datalog MAP/IAT/ECT/TPS/O2 + AFR externa + batería +
frío/caliente; rail con/sin vacío; aceite 80°C ralentí/3k; termostato/fan;
si hay medios: torque controlado, caudales, p(θ), banco inyectores.
Seguridad y procedimientos aplicables obligatorios.

## Ingestión

`research/raw/` (bruto) → normalizado con metadatos (hechos, no copias sin
licencia) → `research/oem_specs/` + `research/field_cases/` +
`research/unsolved_coefficients/`. Versionar por hash; `raw != normalized
!= validated`; revisión por segundo investigador/agente. Búsquedas
prioritarias: foundation §23.13 (instrumentos y trazas).
