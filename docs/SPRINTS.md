# SPRINTS — dependencias, estado y gates

Fuente normativa: `docs/foundation.md` §14–§16. Detalle por sprint:
`docs/sprints/S00.md`–`S10.md`. Estados: `PENDIENTE` · `EN_CURSO` ·
`APROBADO`. S00–S10 APROBADOS; S11+ pendientes.

## FASE A — FOUNDATION-KERNEL (S00–S13)

| Sprint | Dep. | Objetivo | Gate (resumen) | Estado |
|---|---|---|---|---|
| S00 | — | Charter, alcance, reglas repo | Colaborador explica verificado/calibrado/validado, localiza fuente, explica control contrafactual; sin código física | **APROBADO** (2026-10-10, propietario) |
| S01 | S00 | Variante + manual OEM + ficha maestra | Cero numéricos esenciales sin fuente o `PENDING`; ECU/sensores ligados a variante | **APROBADO** (2026-10-10, gate verificado; ver `docs/sprints/S01.md`) |
| S02 | S00 | Toolchain, CI, runner headless | Segundo equipo clona y obtiene mismo hash bajo entorno documentado | **APROBADO** (2026-10-10; clon fresco + replay MATCH; ver `docs/sprints/S02.md`) |
| S03 | S02 | Unidades, constantes, álgebra | Compilador/validadores impiden mezclas dimensionales | **APROBADO** (2026-10-10; 27 tests; ver `docs/sprints/S03.md`) |
| S04 | S01–S03 | Catálogo estructural e instancias | Grafos persistentes y operaciones trazables; sin simular física | **APROBADO** (2026-10-10; reload match; ver `docs/sprints/S04.md`) |
| S05 | S02–S04 | Eventos, snapshots, replay | 100 secuencias controladas sin divergencia estructural | **APROBADO** (2026-10-10; 0 divergencias; ver `docs/sprints/S05.md`) |
| S06 | S03–S05 | Cinemática cigüeñal/pistón/biela | Conservación geométrica + tests analíticos; sin RPM simuladas | **APROBADO** (2026-10-10; 8 tests; ver `docs/sprints/S06.md`) |
| S07 | S04,S06 | Herramientas y contactos | Sin daño sin material/criterio; errores detectables por estado | **APROBADO** (2026-10-10; 5 tests; ver `docs/sprints/S07.md`) |
| S08 | S01,S03–S05,S07 | Pernos, precarga, junta | Torque/lubricante cambian precarga; calibración pendiente | **APROBADO** (2026-10-10; 6 tests; ver `docs/sprints/S08.md`) |
| S09 | S03,S06 | Gas ideal / cilindro sellado | Coincide con ideal en tolerancia justificada; no «D16 validado» | **APROBADO** (2026-10-10; err 7e-11, orden 3.9; ver `docs/sprints/S09.md`) |
| S10 | S09 | Combustión reducida | Retraso desplaza p/trabajo explicablemente; sin dyno al freno | **APROBADO** (2026-10-10; barrido monótono; ver `docs/sprints/S10.md`) |
| S11 | S06–S07,S09 | Distribución SOHC | 4 cilindros derivan de cam+crank compartidos; sin daños automáticos | **APROBADO** (2026-10-10; 6 tests; ver `docs/sprints/S11.md`) |
| S12 | S09,S11 | Flujos adm/esc 0D | Efecto válvula por caudal calculado; declarar faltantes 1D | **APROBADO** (2026-10-10; 8 tests; ver `docs/sprints/S12.md`) |
| S13 | S03,S10,S12 | Combustible rail/inyector | Sin rendimiento desde AFR sin entrega real; sin DTC por bomba sin presión | **APROBADO** (2026-10-10; 5 tests; ver `docs/sprints/S13.md`) |

## FASE B — RUNNABLE-ENGINE (S14–S22)

| Sprint | Dep. | Objetivo | Gate | Estado |
|---|---|---|---|---|
| S14 | S06,S10–S13 | Dinámica cigüeñal + pérdidas | Sin energía gratis; aceleración = torques/inercia en error numérico | **APROBADO** (2026-10-10; 6 tests; ver `docs/sprints/S14.md`) |
| S15 | S13–S14 | Starter, batería, encendido | Crank sin fuel/spark no arranca; trazas eléctricas inspeccionables | **APROBADO** (2026-10-10; 5 tests; ver `docs/sprints/S15.md`) |
| S16 | S10–S15 | Arranque + ralentí end-to-end | Ralentí sin clamp ni torque mágico; puede no coincidir con Honda | **APROBADO** (2026-10-10; 7 tests; ver `docs/sprints/S16.md`) |
| S17 | S01,S05–S06,S15–S16 | CKP/TDC/CYP exactos | Patrón trazado a fuente; si no, `GENERIC_TRIGGER`, prohibido OEM | **APROBADO** (2026-10-10; 7 tests + TRIG-2026-10-10; ver `docs/sprints/S17.md`) |
| S18 | S13,S15–S17 | ECU trigger decoder + scheduler | Señales erróneas alteran sync/timing causalmente; visible solicitado vs efectivo | **APROBADO** (2026-10-10; 7 tests; ver `docs/sprints/S18.md`) |
| S19 | S12–S13,S15,S18 | Sensores, cableado | ECT falso cambia ECU sin cambiar T física; MAP falso no cambia p real | **APROBADO** (2026-10-10; 12 tests; ver `docs/sprints/S19.md`) |
| S20 | S05,S07,S17–S19 | Herramientas diagnósticas | Sin panel secreto; tests conocen causa solo para evaluación | **APROBADO** (2026-10-10; 8 tests; ver `docs/sprints/S20.md`) |
| S21 | S09–S16,S19 | Térmico L1 | Todo calor tiene fuente/sumidero; T no cambia por comando de sensor | **APROBADO** (2026-10-10; thermal 1 + engine 9; ver `docs/sprints/S21.md`) |
| S22 | S14,S21 | Aceite/galerías/cojinetes | Fallo aceite por ruta presión/film/fricción, nunca bandera | **APROBADO** (2026-10-10; 3 tests; ver `docs/sprints/S22.md`) |

## FASE C — CAUSAL-DIAGNOSTIC-BASE (S23–S30)

| Sprint | Dep. | Gate | Estado |
|---|---|---|---|
| S23 | S08–S12,S21–S22 | Sin multiplicador de potencia ni rutina de síntomas; fidelidad etiquetada | **APROBADO** (2026-10-10; 4 tests; ver `docs/sprints/S23.md`) |
| S24 | S20,S23 | Diagnóstico con ≥2 mediciones independientes, no flag de daño | PENDIENTE |
| S25 | S14–S24 | Real vs simulado + train vs holdout; no inventar dyno | PENDIENTE |
| S26 | S21–S25 | Desgaste por leyes/provenance, no FPS; sin evidencia solo tendencias | PENDIENTE |
| S27 | S07–S08,S11,S22,S26 | Sin instante de fallo real sin material/geometría/calibración | PENDIENTE |
| S28 | S14–S27 | HP desde torque integrado, no circular desde spec | PENDIENTE |
| S29 | S23–S28 | Cada síntoma remonta a ecuaciones; sin `if fault_count>1` | PENDIENTE |
| S30 | S00–S29 | Sin «predictivo» sin evidencia; incompleta pero honesta puede pasar | PENDIENTE |

## FASE D — Taller 3D y expansión (S31–S37, tras base)

Prototipado CAD/UI en paralelo permitido, sin bloquear ni «validar»
ecuaciones con animaciones. Ver foundation §16. Todos PENDIENTE.

## Regla de avance

S00–S23 APROBADOS. Batch S21–S25 autorizado, siguiente: S24.
**S26 prohibido sin autorización.** Gates universales §12.3 +
`docs/definition-of-done.md` aplican.
