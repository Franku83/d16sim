# PROGRESS — log objetivo de cambios ejecutados

Formato: fecha UTC · sprint · cambio · tests/referencia · commit.
Solo hechos constatables; sin cifras improvisadas.

## 2026-10-10 · S00 · Inicialización documental

- `git init` en `/home/franku/projects/d16sim` (antes no era repo).
- Copia verbatim `ENGINE_ENGINEERING_SANDBOX_D16Y7_RESEARCH_FOUNDATION.md`
  → `docs/foundation.md` (fuente normativa v0.2, 2026-10-09).
- Research conservados íntegros en raíz, sin reescritura:
  `D16Y7_RESEARCH_v0.1.md`, `D16Y7_RESEARCH_DONANTES_v0.1.md`.
- Creados: `README.md`, `docs/charter.md`, `docs/definition-of-done.md`,
  `docs/EVIDENCE_POLICY.md`, `docs/limitations.md`,
  `docs/model_assumptions.md`, `docs/RISK_MATRIX.md`, `docs/BACKLOG.md`,
  `docs/SPRINTS.md`, `docs/sprints/S00.md`, `docs/PROGRESS.md` (este),
  `docs/HANDOFF.md`, `docs/LICENSE_STATUS.md`,
  `docs/adr/ADR-001..012 + 000-template`,
  `data/provenance_registry.yaml`, `.github/workflows/docs-check.yml`.
- Tests: verificación documental (listado de archivos + `git status`);
  sin tests numéricos (sin física en S00, por diseño).
- Deuda: licencia por decidir; toolchain sin pin (S02); raw vacío (S01).
- Commits: ver `git log --oneline`.

## 2026-10-10 · S01 · Variante y ficha maestra (APROBADO)

- Variante congelada `US_1997_CIVIC_DX_2DR_5MT_OBD2A_P2E_FAMILY`
  (`data/d16y7/reference_variant.yaml`); distribuidor/ECU-sufijo en
  DISPUTED/REQUIRES_VERIFICATION, política P2E-like-no-OEM.
- Creados: `docs/variants/D16Y7_DATA_DICTIONARY.md`,
  `data/d16y7/D16Y7_OEM_SPECS.csv`,
  `data/d16y7/OEM_CATALOG_1997_DX_5MT.csv`,
  `data/d16y7/parts_inventory.csv`, `data/d16y7/unverified_values.csv`,
  `docs/variants/bibliography.md`, `docs/variants/discrepancies.md`,
  `docs/variants/acquisition_protocol.md`, `docs/sprints/S01.md`.
- Gate S01: PASS — 20 filas OEM_SPECS con PENDING/REQUIRES, 18 PN con
  IDENTITY/DISPUTED, 0 numéricos sin fuente; sin proxies como nominal;
  ECU/sensores ligados a variante; sin física añadida.
- Deuda: edición/página OEM exactas pendientes; torque/ángulos de apriete
  PENDING; licencia sigue por decidir (bloquea S02).

## 2026-10-10 · Licencia (decisión del propietario, desbloquea S02)

- Apache-2.0 exclusivamente al código propio. Creados `LICENSE`,
  `THIRD_PARTY_NOTICES.md` (tras S02); `Cargo.toml`s declaran
  `license = "Apache-2.0"`. Research/datos terceros bajo sus condiciones.
  Sin código GPL sin revisión+autorización.

## 2026-10-10 · S02 · Toolchain, CI y runner headless (APROBADO)

- Creados: `rust-toolchain.toml` (1.99.0), `Cargo.toml` workspace
  (`engine-types`, `physics` stub, `simulation`, `cli`), `Cargo.lock`,
  `.github/workflows/rust-ci.yml` (Linux+Windows), `THIRD_PARTY_NOTICES.md`
  (44 paquetes, todo permisivo MIT/Apache/Unlicense/Unicode, sin GPL),
  `docs/adr/ADR-013-trazas-jsonl.md`, `docs/sprints/S02.md`, `.gitignore`.
- Tests: `cargo test --locked --workspace` → 9 passed, 0 failed
  (unit + tamper-replay + schema + validación datos S01).
  `cargo fmt --check` OK; clippy: 0 errores `unwrap_used` (avisos
  `missing_errors_doc` como warn, a documentar en S03).
- Demo: `validate-data` OK; `physics-status` confirma stub S09;
  `init/run-scenario` ×2 → hashes idénticos
  (`manifest=b6030259…`, `traces=dd830da7…`, `metrics=3e2d97c1…`);
  `replay: MATCH`; `report` OK.
- Gate S02: PASS — clon fresco `/tmp/opencode/d16sim-clone` reproduce
  `traces=dd830da7…` idénticas + `replay: MATCH` (misma toolchain/Linux).
  Windows-CI pendiente de runner (deuda).
- Corrección: commit inicial arrastró `target/`; excluido con `.gitignore`
  + `git rm --cached` (commit `84d3390`).
- Deuda: avisos clippy `missing_errors_doc`; CI Windows sin ejecutar;
  `cargo-deny` evaluado y no adoptado (auditoría manual documentada).

## 2026-10-10 · S03 · Unidades y álgebra (APROBADO)

- Crates `units` (tipos SI + abs/gauge/delta + AFR/lambda + banco/rueda +
  política f64) y `math` (`Table1` con dominio); `docs/physics/dimensions.md`;
  `d16sim units-demo` (750 rpm → 78.539816 rad/s; rail 270 kPa abs →
  168.675 kPa gauge; ECT 80 °C → 353.15 K; lambda 0.897959; tabla 20.000).
- Tests: 27 passed, 0 failed (un test falló primero por constante manual
  errónea lb·ft — corregido contra el factor definitorio, no contra el
  código). `fmt` OK; clippy sin errores `unwrap_used`.
- Gate S03: PASS por construcción (tipos disjuntos, campo privado, sin
  impls cruzados) + validadores testeados. Deuda: suite `trybuild`
  diferida; constantes físicas (R, composiciones) en S09.

## 2026-10-10 · S04 · Catálogo y grafo de montaje (APROBADO)

- Crate `assembly` (tipos + `EngineGraph` + ops tri-estado + catálogo
  semilla 10 definiciones EPC + `block-001` inicial); `d16sim
  assembly-demo` (bare `4e9010e7…` → montado `1799d767…`, reload match,
  3 instancias / 1 joint / 3 registros).
- Tests: 5 passed (fuera de orden, ausente/duplicados, serial+roundtrip,
  incompatibles/versión rancia, esquema futuro fail-closed). Total
  workspace: 32 passed, 0 failed. Hashes S02 intactos tras higiene de
  lints (`traces=dd830da7…`).
- Gate S04: PASS — persistencia + trazabilidad verificadas; sin física.
- Deuda: compatibilidad fina en S07; `mates_with` sin chequeo cruzado.

## 2026-10-10 · S05 · Eventos, snapshots y replay (APROBADO)

- `simulation::history` (`HistoryEvent` + orden total + `SnapshotFile`
  inmutable/versionado + `replay_history` + generador LCG); `d16sim
  history-demo` (100 secuencias, 0 divergencias; first `b2141ef5…`,
  last `af7eb4c1…`; `events.jsonl` idéntico entre invocaciones
  `d600541e…`).
- Tests: 3 passed historia (100×40 ops por dos caminos con save/load
  intermedio) + resto workspace OK (13 objetivos ok, 0 fallos). Sin
  `std::time`/rng en `src/` (grep). `fmt` OK; clippy 0 errores.
- Gate S05: PASS — 0 divergencias estructurales; tolerancias numéricas N/A
  (sin física) documentado.
- Deuda: migraciones de esquema futuras (stub fail-closed, sin migrador
  v0→v1 porque v0 no existe); CI Windows sin ejecutar.

## 2026-10-10 · S06 · Cinemática biela-manivela (APROBADO)

- Crate `kinematics` (geometría analítica + fases 720° + preset
  ilustrativo); `d16sim kinematics` (Cyl1 TDC: V=47.3343 cm³=Vc;
  BDC: 444.9421 cm³; v=0 en puntos muertos); ADR-014;
  `docs/physics/kinematics.md`; H-S06-01..04.
- Tests: 8 passed (posiciones, simetría, FD4, límites λ/CR, fases,
  compañeras, escalado ω). 1 error de tipos en test corregido.
- Gate S06: PASS — verificación matemática, no validación; ω es
  parámetro, sin RPM simuladas.
- Deuda: offset de bulón, masas/inercia (S14), contacto (S07/S11).

## 2026-10-10 · S07 · Herramientas y contactos (APROBADO)

- Crate `contacts` (interferencias + medición L1 + herramientas);
  `d16sim contact-check` (interferencia 2 mm a PMS con lift impuesto,
  `damage: NotAssessed`); `docs/physics/contacts.md`; H-S07-01..04.
- Tests: 5 passed (huelgo exacto, cant, biela/rosca, no-daño incl.
  penetración profunda, medición). Build sin warnings nuevos.
- Gate S07: PASS — daño jamás decretado; interferencias cuantificadas.
- Deuda: compatibilidad fina S04 sigue nominal; dinámica de válvula en
  S11; material en S27.

## 2026-10-10 · S08 · Pernos, precarga y junta L1 (APROBADO)

- Crate `fastener` (L1 `T=K·F·d` + junta por segmentos +
  `SealConductance::Unsupported`); `d16sim fastener-seal-test`
  (seco 25.0 kN vs aceitado 33.3 kN a 50 N·m; 208.33 MPa/anillo;
  `SealingIndeterminate`); ADR-015; `docs/physics/fastener.md`;
  H-S08-01..04.
- Tests: 6 passed (lubricante, monotonía/propagación, rechazo
  ángulo-solo, fluencia-estado sin rotura, campo uniforme/subapriete,
  costura S23).
- Gate S08: PASS — torque y lubricante cambian F según modelo;
  calibración pendiente declarada en reporte y física.
- Deuda: secuencia OEM sin transcribir; interacción elástica; fuga en
  S23; K medido en S25.

## 2026-10-10 · S09 · Cilindro sellado ideal (APROBADO)

- Crate `thermo` (mezcla R/γ + `ThermoState` consistente + ciclo cerrado
  RK4/Simpson); `d16sim closed-cycle` (W −153.3826 J err 1.4e−10,
  W_net 8.6e−9 J, orden 3.888); ADR-016; `docs/physics/closed_cylinder.md`;
  H-S09-01..04.
- Tests: 3 passed (ideal+reversibilidad+positividad, orden RK4,
  rechazos). 1 fallo intermedio honesto: cuadratura trapezoidal O(h²)
  limitaba el trabajo → Simpson (documentado). Build 0 warnings.
- Gate S09: PASS — coincide con ideal en tolerancia medida; rotulado no
  validado.
- Deuda: γ(T,mezcla), fugas, pared, residuales, combustión (S10/S12/S21).

## 2026-10-10 · S10 · Combustión Wiebe y presión indicada (APROBADO)

- Crate `combustion` (Wiebe validado + zona única + auditoría GL);
  `d16sim closed-cycle-burn` (W 562.5 J, IMEP 14.1 bar ill., pmax 5852
  kPa @ 377.8°, Q err 5.5e−8); ADR-017; `docs/physics/wiebe.md`;
  H-S10-01..04.
- Tests: 4 passed (forma/completitud, barrido monótono W+pmax, cero-fuel
  ⇒ motored, rechazos). 1 fallo intermedio honesto: Simpson global del
  calor hereda O(h·salto) del borde Wiebe (1.4e−4) → GL por tramos
  (5.5e−8), analizado en ADR-017.
- Gate S10: PASS — retraso explicable y monótono; sensibilidad revisada
  (barrido 340/350/360), no calibrada; sin dyno al freno.
- Deuda: γ(T,mezcla), residuales, pared L1, ciclo-a-ciclo, knock
  (UNSUPPORTED), gas exchange (S12), p(θ) medida (S25).

## 2026-10-10 · Revisión de consistencia pre-batch S11–S15 (sin cambios científicos)

- Cruzados `HANDOFF.md`, `SPRINTS.md`, `PROGRESS.md`: gates S00–S10,
  fechas, hashes y deuda coinciden; historial `git log` 1 commit/sprint
  (S02 con fixup `target/`); sin contradicciones científicas.
- Corregidas solo declaraciones obsoletas de estado: `README.md`
  (sprints, árbol, licencia), `SPRINTS.md` (línea S01+), `HANDOFF.md`
  (crates/código/hipótesis), `BACKLOG.md` (estados S00–S10), índice ADR
  (014–017). Intactos: historial PROGRESS, gates, ADRs, research,
  `docs/sprints/S00.md`.

## 2026-10-10 · S11 · Distribución SOHC (APROBADO)

- Crate `valvetrain` (proxy A6 etiquetado + sen² ilustrativa + correa 2:1
  + eventos por fase desde árbol único); `d16sim valvetrain` (Cyl1: IO
  719, IC 221, EO 498, EC 2; Vtr 410.40 cm³ idéntico; min 5.99 mm; con 5
  dientes: Vtr 322.69 cm³ + solape −2.04 mm informado, sin daño);
  ADR-018; `docs/physics/valvetrain.md`; H-S11-01..05.
- Tests: 6 passed (aritmética donantes 3°/−25°, autoconsistencia de forma,
  fases/compañeras, desfase causal). 1 bug real corregido: doble conteo
  del centro (IC 331→221), detectado por V atrapado inverosímil.
- Gate S11: PASS — 4 cilindros de cam+crank compartidos; daño jamás
  decretado.
- Deuda: tabla lift-vs-CA medida, dinámica de muelle/flotación, holgura
  térmica, VTEC (otra arquitectura); perfil Y7 sigue MISSING-DATA.

## 2026-10-10 · S12 · Flujos 0D con Cd medido (APROBADO)

- Crate `flow` (orificio compresible + Cd Y7/P2A-06 + nodos 2 especies +
  ciclo 720° con auditoría); `d16sim gas-exchange` (trapped 519.9 mg,
  residual 0.0000, cierres ~1e-15, p 96/2235 kPa, 2881 puntos);
  ADR-019; `docs/physics/gas_exchange.md`; H-S12-01..05.
- Tests: 8 passed (choke/incompresible, monotonía, Cd, cierre masa,
  reflujo+especies, dosificación mariposa 0.79×, refinamiento <1%,
  rechazos). 3 bugs reales corregidos: literal crítico, Pa-como-kg,
  envoltura de leva (este último también tocó S11, re-verificado 6/6).
- Gate S12: PASS — caudal calculado, sin VE; faltantes 1D declarados.
- Deuda: garganta explícita, puertos/colector medidos, red 1D,
  multicilindro y periódico en S16, γ(T), lift medido.

## 2026-10-10 · S13 · Inyección L1 comandado vs entregado (APROBADO)

- Crate `fuel` (rail/regulador/bomba + inyector Q√ΔP/deadtime/tensión +
  agenda por estrategia); `d16sim fuel-supply-test` (ralentí 3.37+0.62
  ms = 7.69 mg exacto al research; 12 V −9%; WOT 11.58 ms; NoPressure y
  Saturated como estados); ADR-020; `docs/physics/fuel_supply.md`;
  H-S13-01..04.
- Tests: 5 passed (cadena ralentí, ley √ΔP + tensión, comandado vs
  entregado con 4 casos, rail/vacío, agenda). Sin tipo DTC en el crate.
- Gate S13: PASS — sin AFR→rendimiento; sin DTC por bomba.
- Deuda: banco P2A-000, atomización/wall-wetting, curva de rail,
  scheduler S18.

## 2026-10-10 · S14 · Dinámica de cigüeñal (APROBADO)

- Crate `crank` (J media + Chen–Flynn c2=0 + torque de traza S12 real +
  RK4 temporal); `d16sim crank-dynamics` (J 0.15052; 750→0 rpm;
  W_ind 0.44 J vs W_fric 464.7 J; cierre 2.7e−8); ADR-021;
  `docs/physics/crank_dynamics.md`; H-S14-01..04.
- Tests: 6 passed (giro, auditoría+convergencia, rotor bloqueado,
  forma FMEP, J, trabajo nulo). 1 corrección honesta: monotonía
  estricta ⇒ neto + auditoría (rizo = volante).
- Gate S14: PASS — sin energía gratis; torques/inercia en error numérico.
- Deuda: J(θ), FMEP medido, accesorios/arranque S15, combustión S16.

## 2026-10-10 · S15 · Arranque arrastrado sin bootstrap (APROBADO)

- Crate `electrical` (batería + starter/reductora + bobina/dwell +
  `cold_crank` con `spin_with_assist`); `d16sim cold-crank` (started
  false; 236.8 rpm; rizo 295 por compresión; chispa 134 mJ; cierre
  2e−15); ADR-022; `docs/physics/cranking.md`; H-S15-01..05.
- Tests: 5 passed (colapso V, chispa V², η≤1, giro+rizo, 4 fallos sin
  arranque). 1 bug real: par sin acoplo a V ⇒ acoplo L1 documentado.
  Refactor S14 (`spin_with_assist`) con resultados idénticos (6/6).
- Gate S15: PASS — sin fuel/chispa no arranca (ni con ellos, por diseño);
  trazas V·I inspeccionables.
- Deuda: integración con combustión S16, gas exchange vivo, starter
  medido, Bendix, fricción fría.

## 2026-10-10 · S16 · Motor integrado end-to-end (APROBADO)

- Crate `engine` (4 cámaras + nodos + control open-loop + arranque +
  snapshot reanudable bit-idéntico); `d16sim engine-run` (ralentí ~1026
  rpm con mariposa 0.05+bypass 0.02; speed-density 1920/3102/5552;
  0/0/699 en fallos; masa 1e−11, energía 7e−5); ADR-023;
  `docs/physics/integrated_engine.md`; H-S16-01..05.
- Tests: 7 passed (ralentí sin clamp, orden causal, corte coherente,
  snapshot bit-idéntico, determinismo+convergencia, flancos, sellado).
- Bugs reales corregidos (4+2 auditoría): rebanado bajo deadtime,
  doble disparo floor, −p·dV perdido, Q vs E_chem y ΔKE en auditoría.
- Gate S16: PASS — ralentí emergente reportado tal cual; sin magia.
- Deuda: lazo cerrado, AFR→quemado, multicilindro diferenciado,
  periódico largo, control S19, arranque parametrizado.

## 2026-10-10 · S17 · Triggers CKP/TDC/CYP GENERIC_TRIGGER (APROBADO)

- Pre-S17: `research/raw/trigger-distributor-2026-10-10.md` (hondabase +
  rusEFI leídos; carmanualsonline 403; sin corroboración TD-80U) +
  TRIG-2026-10-10 en provenance + C-19/C-20 en discrepancias:
  correspondencia con la variante NO verificable ⇒ GENERIC_TRIGGER.
- Crate `trigger` (reluctores 24/4/1 + VR/noise/histéresis + 3 canales
  con fallos + muestreo); `d16sim trigger-waveform` (24/4/1; 23 con
  borde menos; 0 con TDC abierto); ADR-024; `docs/physics/trigger_hw.md`;
  H-S17-01..04.
- Tests: 7 passed (conteos, espaciado 30°, fallos, offset 10°,
  alineación TDC/CYP, amplitud∝ω). 1 corrección: sensibilidad ×20
  (ruido ahogaba umbral).
- Gate S17: PASS — trazado a fuente + GENERIC_TRIGGER imposible de
  etiquetar OEM por construcción.
- Deuda: geometría/fase/pines OEM (scope + manual), CKF, transitorios.

## 2026-10-10 · S18 · ECU P2E-like con decoder y scheduler (APROBADO)

- Crate `ecu` (decoder 24+1/4+1 + FSM + mapas propios + scheduler con
  latencia visible + DTC REF-* por persistencia); `d16sim ecu-demo`
  (Running 14.0/9.5° + 7.70 mg; Fallback batch; inhibido + REF-CKP-LOSS);
  ADR-025; `docs/physics/ecu_ref.md`; H-S18-01..04.
- Tests: 7 passed (sync/RPM, borde→confianza, timeout/360°, nodos/clamp,
  inhibición/degradado, req vs eff, persistencia/curación).
- Gate S18: PASS — causalidad señal→actuación; solicitado vs efectivo.
- Deuda: estrategia OEM, closed-loop/trims, IACV/PWM, P0xxx, transitorios.

## 2026-10-10 · S19 · Sensores y cableado (APROBADO)

- `electrical::{sensors,wiring}` (canal + transductores + cable con
  fallos + ADC/filtro + monitor REF-* + acción ECU testeable);
  `d16sim sensor-demo` (sano/falso-frío/abierto/corto con físicos
  fijos); ADR-026; `docs/physics/sensors_wiring.md`; H-S19-01..04.
- Tests: 12 passed (escalón 63%, ruido seed, NTC/O2, cable, monitor,
  2 de gate: ECT y MAP falsos cambian acción sin tocar físico).
- Gate S19: PASS — separación por construcción + demos + tests.
- Deuda: curvas OEM, heater/retardo O2, PWM IACV, closed-loop, Bode
  completo, corrosión/diafonía.

## 2026-10-10 · S20 · Instrumentos sin panel secreto (APROBADO)

- Crate `diagnostics` (sondas + multímetro + scope + scanner espejo +
  manual por enlaces + casos ambiguos; SIN dep de `engine`/`assembly`);
  `d16sim diagnose-demo` (Inconclusive→Discriminated en map y ckp);
  ADR-027; `docs/physics/diagnostic_tools.md`; H-S20-01..04.
- Tests: 8 passed (sonda, multímetro, trigger/alias, scanner, manual,
  2 de ambigüedad con crates reales). 1 tolerancia corregida
  (cuantización 1 mV).
- Gate S20: PASS — sin panel secreto por construcción; verdad del
  harness solo para montar escenarios.
- Deuda: instrumentos mecánicos S24, Bode, in-cylinder, tutorial/evaluador.

## 2026-10-10 · S21 · Red térmica L1 (APROBADO)

- Crate `thermal` (4 nodos + termostato + ECT/enrich) acoplada a `engine`
  como opt-in (S16 bit-idéntico sin ella); fricción→aceite; auditoría
  ramificada; snapshot esquema 2 con migración v1; `d16sim warmup` +
  `export-trace` (contrato trace-bundle-v1 solo lectura, sin Blender/UI
  en core); ADR-028; `docs/physics/thermal.md` + `docs/export-contract.md`;
  H-S21-01..05.
- Tests: thermal 1 + engine 9 passed (calentamiento, termostato, lag ECT,
  enrich causal, sin-quemado, migración con hash falso rechazado).
- Gate S21: PASS — todo calor con fuente/sumidero; ECT emerge de la red.
- Deuda: hA/C medidos, termostato/bomba reales, viscosidad S22, S19
  completo, calentamiento largo fuera de horizonte de test.

## 2026-10-10 · S22 · Lubricación por red (APROBADO)

- Crate `lube` (Walther + bomba/alivio + 9 ramas por holgura + película
  con inanición + enlace a fricción); `d16sim lubrication-test` (sano
  525 kPa/film 0.56; caliente film 0.19/mult 2.62; casos por ruta);
  ADR-029; `docs/physics/lubrication.md`; H-S22-01..05.
- Tests: 3 passed (Walther, película/enlace, continuidad + respuesta +
  inanición no vacua). k_ref corregido 1e-9→1e5 (absurdo detectado).
- Gate S22: PASS — ruta causal sin banderas; película hacia S26.
- Deuda: mapa de bomba, consumidores distribuidos (pinneo declarado),
  caída de línea, HD completa, k calibrada.

## 2026-10-10 · S23 · Rutas de fuga generales (APROBADO)

- Crate `leak` (dominios gas/aceite/refrigerante + orificio/laminar +
  suministro regulado + PCV); junta desde SealContactState con p_ref de
  clamp completo (ancla autoconsistente); anillo gap×prof×Cd 0.03;
  4 escenarios (sano/subapretada/ausente/anillo×2); `d16sim leak-test`;
  ADR-030; `docs/physics/leaks.md`; H-S23-01..04.
- Tests: 4 passed (ley junta/anillo, suministro+reparto 1:3, órdenes
  robustos, sensibilidades reportadas). Bugs reales: desigualdad
  invertida, Pa-como-kg, falta de respiradero; rediseño a suministro
  regulado (vaciado saturaba).
- Gate S23: PASS — cero referencias a potencia; todo UNCALIBRATED.
- Deuda: p_ref y Cd sin calibrar (S25), sin dinámica de anillo, sin
  aceite→exterior (retén S26), sin acople al motor vivo.

## 2026-10-10 · S24 · Diagnóstico mecánico (APROBADO)

- `diagnostics::mech` (compresímetro/cranking L1 + leak-down +
  galgas + procedencia obligatoria + protocolo de 2 mediciones);
  `d16sim mech-demo` cableado a S23 (sano sin falta; anillo×2 bajo
  umbral; subapretada ambigua con anillos + 3er test prescrito;
  ausente a junta); ADR-031; `docs/physics/mech.md`; H-S24-01..05.
- Tests: 4 passed (FIELD-07/08, inconclusividad simple, separación
  doble). Bug real: respiradero sumado al área (pico sano 50 kPa);
  corregido (1400 kPa).
- Gate S24: PASS — ≥2 mediciones, sin flags, sin dependencia del truth.
- Deuda: causa válvula sin modelo S23, curvas reales, umbrales, 3er test.

## 2026-10-10 · S25 · Calibración sin datos de campo (APROBADO, NUMERICALLY_VERIFIED_ONLY)

- Crate `calibration` (params con priors + manifiesto anti-leakage +
  Gauss-Newton con escalado/backtracking/covarianza + Morris);
  `d16sim calibration-demo` (recuperación exacta, SYNTHETIC rotulado);
  `docs/calibration/` (reporte, C1–C7, manifiesto vacío, model cards);
  ADR-032; H-S25-01..04.
- Tests: 4 passed (set, split —incl. rechazo de sintético en holdout—,
  recuperación exacta, ranking). Bugs reales: lockfile, `max` ambiguo,
  GN sin backtracking, covarianza singular → ancla sellada.
- Gate S25: PASS por ausencia declarada — cero dyno inventado, cero
  validación con sintéticos. 8 vacíos numerados; reapertura con C1–C6+M2.
- DETENCIÓN honesta: sin evidencia externa indispensable (P1), el batch
  termina aquí como ordenó el propietario.

## 2026-10-10 · Fase 0 · Reconciliación documental (sin cambios de física/gates)

Auditoría previa a S26 contra repo real (25 crates, 32 ADRs, H-S00–H-S25,
`docs/sprints/S00–S25.md`, 1 commit/sprint S00–S25 + extras conocidos:
`84d3390`orius S02, `98d95cf` revisión pre-S11; historia intacta).
Inconsistencias corregidas: (1) HANDOFF decía 12 crates → 25;
(2) HANDOFF citaba solo H-S06–H-S10 → H-S00–H-S25; (3) SPRINTS encabezado
S00–S10/S11+ pendientes → S00–S25; (4) CONFLICTO S26: HANDOFF Brief S26
describía calibración (copia de S25); Foundation §15 + SPRINTS asignan
S26 a desgaste — se adopta desgaste como definición correcta y la
calibración permanece en S25 (corrección explícita, no sustitución);
(5) duplicaciones eliminadas (doble prohibición S26, estados S23
heredados, tabla BACKLOG congelada en S00 → actualizada con puntero
autoritativo a SPRINTS); (6) HANDOFF extendido con Briefs S27–S30
ausentes. Sin contradicciones materiales irresolubles: no se requirió
detención. Batch S26–S30 autorizado; S31 prohibido.

## 2026-10-10 · S26 · Motor persistente (APROBADO, TREND_CONDITIONED)

- Crate `wear` (slow-state + 3 leyes efectivas + creep + 5 servicios +
  acople gap→fuga S23); `d16sim wear-demo` (50 h: gap +2.4 µm, cojinete
  +0.9 µm, oxidación 0.057); ADR-033; `docs/physics/wear.md`; H-S26-01..05.
- Tests: 4 passed (Archard/oiled, subdivisión, replay+snapshot,
  servicios). Fallos reales: K seco→metros; creep Euler dependiente del
  paso → forma cerrada; test de bits → acuerdo 1e-9.
- Gate S26: PASS — leyes/provenance, escala libre, cero vidas útiles.
- Deuda: tribología P1, fatiga/scuffing/fretting UNSUPPORTED, acople
  motor vivo (S29), K efectivo pendiente de mapa de película.

## 2026-10-10 · S27 · Daños por montaje con dominio (APROBADO)

- Crate `damage` (PV contacto/riesgo + perno con Rp0.2 genérico +
  inanición riesgo + rosca UNSUPPORTED, todo puro); `damage-demo`
  (6 casos); ADR-034; `docs/physics/damage.md`; H-S27-01..05.
- Tests: 3 passed (negativa PV + pureza, perno elástico/incierto/
  plástico, inanición riesgo + rosca). Corrección: 100 Nm sí fluye
  M10 aceitado (física corrigió al test); 80 Nm = incertidumbre.
- Gate S27: PASS — sin instantes de fallo sin material; 1 ruptura
  genérica declarada, resto riesgo/no soportado.
- Deuda: material válvula/asiento, criterio gripado, rosca medida,
  acople dinámico (S29).

## 2026-10-10 · S28 · Dyno auditable (APROBADO, ilustrativo)

- Crate `dyno` (freno declarado + corrección explícita + validez con
  Unsettled + manifiesto); `dyno-demo` (3 Covered, 1.67–2.88 kW);
  ADR-035; `docs/physics/dyno.md`; H-S28-01..05.
- Tests: 4 passed (corrección, identidad global <1 %, causalidad carga,
  rechazo Unsettled). Hallazgo: cola carga-8 con residuo 60 % pese a
  cierre 1e-5 (muestreo/ inestabilidad) → rechazado, no ajustado.
- Gate S28: PASS — potencia desde τ integrado; cero spec circular.
- Deuda: dyno real C1, corrección certificada, inestabilidad (S29).

## 2026-10-10 · S29 · Suite adversarial (APROBADO, ilustrativo)

- Crate `adversarial` (10 deltas + 4 pares + MC con seed + top);
  `adversarial-demo` (interacciones +0/+132/−0/−71, top-5 con hash);
  ADR-036; `docs/physics/adversarial.md`; H-S29-01..05.
- Tests: 4 passed (cierres bajo pares, enmascaramiento + conmuta,
  observabilidad, MC reproducible). Correcciones: starter motora sin
  chispa (no 0); hash canónico ordenado.
- Gate S29: PASS — síntomas desde ecuaciones; cero gobierno de fallos.
- Deuda: blow-by motor (par junta parcial), carga-8, N pequeños, C1.

## 2026-10-10 · S30 · Auditoría final (APROBADO, research foundation)

- Crate `audit` (4 cadenas + 3 perfiles + env + bench);
  `d16sim release-check`; `FOUNDATION_RELEASE_REPORT.md`;
  `capability_matrix.md`; `snapshot-contract.md`; `CHANGELOG.md`;
  ADR-037; H-S30-01..05.
- Tests: 3 passed (4 cadenas, perfiles+determinismo, env+bench).
  Corrección: acoplo fill∝alzada descartado (el calado no entra en
  `lift_at`); cadena leva→motor directa.
- Gate S30: PASS honesto — predictivo denegado, PENDING numerados.
- Batch S26–S30 COMPLETO. S31 prohibido. Base congelada pendiente de
  C1–C6+M2 + experto + 2ª máquina.

## 2026-10-10 · Remediación R01–R02 (IMPLEMENTED_PENDING_INDEPENDENT_REVIEW)

- Rama `remediation/r01-r05` desde S30 (`ff09a0d`). Auditoría independiente
  (`25d528e`) preservada intacta en su worktree. Reporte:
  `docs/remediation/R01_R02_BATCH_REPORT.md`; ADR-038.
- **R01**: F01 transporte de especies (cilindros + nodos, frontera emitida,
  `inyectado = almacenado + quemado + emitido`); F02 observación diagnóstica
  desde flujos físicos (no la etiqueta del escenario); F08 advección de
  especie fresca en la red de fugas; F09 conversión volumen→masa con densidad
  de líquido; F10 calor al ambiente en el ledger térmico integrado.
- **R02**: F05 parseo f64 lossless (`serde_json/float_roundtrip`); F06
  verificación de hash v1 **antes** de migrar; F07 `pmax_pa` persistido
  (replay dividido = continuo con `c2 ≠ 0`); F15 fronteras fail-closed por API
  (engine/thermal/dyno/decoder/calibration) y validación física en `restore`.
- Tests: `cargo test --locked --workspace` 183 passed / 0 failed (163 previos +
  20 nuevos de regresión R01/R02); fmt OK; clippy exit 0 bajo
  `-D clippy::unwrap_used`. Harness independiente: **23 → 10 fallos**, y los
  10 restantes son de R03/R04 (fuera del batch autorizado).
- Cambios numéricos: las cifras nominales de RPM/trabajo cambian (especies +
  presión de fricción); no se reajustaron parámetros para conservarlas.
- **STOP**: no se inicia R03 hasta autorización explícita. S31 prohibido.

## 2026-10-10 · Remediación R01–R02-C1 (IMPLEMENTED_PENDING_INDEPENDENT_REVIEW)

- Rama `remediation/r01-r02-c1` desde `8a7e868` (R01–R02 verificado con
  `REMEDIATION_REQUIRED_BEFORE_R03_R04`). Reporte:
  `docs/remediation/R01_R02_C1_BATCH_REPORT.md`; ADR-039.
- **RV01/F01**: presupuesto químico transitorio (la energía consume el
  inventario en el mismo paso); exceso 1156.15→1.5e-11 J.
- **RV02/F02**: toda ruta de gas instrumentada entra en el reparto; reparto
  nulo ⇒ Inconclusive (no Rings).
- **RV03/F05**: `-0.0` conserva bits en los campos opcionales.
- **RV04/F06**: `MigrationProvenance` (origen, transformación, campos
  ausentes, `exact_neutral`/`approximate_hot`).
- **RV05–RV09/F15**: validación atómica fail-closed en engine, thermal,
  decoder/ECU, fitter y dyno; sin panic ni mutación parcial.
- Tests: workspace **198 passed / 0 failed** (183 + 15); fmt/clippy OK.
  Harness adversarial **19/19** (196 predicados; antes 9/19). Harness original
  **22/32** con los mismos 10 fallos de R03/R04. F07/F08/F09/F10 conservados.
- **STOP**: no se inicia R03/R04/R05 ni S31 sin autorización explícita.

## 2026-10-10 · Remediación R01–R02-C2 (IMPLEMENTED_PENDING_INDEPENDENT_REVIEW)

- Rama `remediation/r01-r02-c2` desde `caad1cb` (C1 verificado con
  `C1_REMEDIATION_REQUIRED`). Matriz de fronteras:
  `docs/remediation/R01_R02_C2_BOUNDARY_MATRIX.md`; ADR-040; reporte:
  `docs/remediation/R01_R02_C2_BATCH_REPORT.md`.
- **C1-01/F15**: validación integral de EngineInputs (control, combustible,
  eléctrico/arranque, inercia, Wiebe, térmico), nodos (v/cv), ledgers,
  coherencia térmica y confirmación del candidato; errores de starter
  propagados; auditoría finita obligatoria.
- **C1-02/C1-04/F15**: energía térmica derivada (T·C) validada en
  `stored_j_checked`, `step`, `restore`, `initial_state` y `run_engine`.
- **C1-03/F02**: `LeakSplit` como descomposición completa normalizada;
  observaciones inválidas/parciales ⇒ `Inconclusive`.
- **C1-05/F06**: identidad semántica `model` (esquema + contabilidad);
  snapshots pre-C2 migran como aproximados, nunca nativos exactos;
  metadata de procedencia documentada como no autenticada (C1-G01).
- Tests: workspace **204 passed / 0 failed** (199 + 5 C2); fmt/clippy OK.
  Controles C2 **15/15** (150 predicados; antes 10/15). Harness adversarial
  **19/19**; harness original **22/32** con los mismos 10 fallos de R03/R04.
  Ralentí nominal idéntico a C1 (1047.3038737 rpm): solo validación.
- **STOP**: no se inicia R03/R04/R05 ni S31; no se fusiona a `r01-r05`.

## 2026-10-10 · Remediación R01–R02-C3 (IMPLEMENTED_PENDING_INDEPENDENT_REVIEW)

- Rama `remediation/r01-r02-c3` desde `a0040c5` (C2 verificado con
  `C2_REMEDIATION_REQUIRED`). Matriz:
  `docs/remediation/R01_R02_C3_ACCEPTANCE_MATRIX.md`; ADR-041; reporte:
  `docs/remediation/R01_R02_C3_BATCH_REPORT.md`.
- **C2-01/F15**: validación de inyector (q_ref/p_ref/deadtime/incertidumbre) y
  levas (duración/at050/alzada/flanco/centerline/LSA).
- **C2-02/F15**: T=U/(m·cv) y p/ρ derivadas validadas en restore/resume
  (incl. no-op) y run_engine.
- **C2-03/F15**: energía térmica evaluada con el estado real, no con cold(20).
- **C2-04/F06**: `model` desconocido/incoherente rechazado en todos los
  esquemas; histórico sin `model` conservado.
- **C2-05/F06/F07**: nativo exacto exige `pmax` cuando c2≠0; si falta,
  `approximate_hot`; control c2=0 exacto.
- **C2-06/F15**: temperatura de batería negativa legítima admitida.
- Tests: workspace **209 passed / 0 failed** (204 + 5); fmt/clippy OK.
  Checker C2 **11/11** (1442 predicados; antes 6/11). Oráculos previos:
  S30 **22/32**, adversarial **19/19**, C1 **15/15**. Ralentí nominal
  idéntico (1047.3038737 rpm): solo validación.
- **STOP**: no se inicia R03/R04/R05 ni S31; no se fusiona a `r01-r05`.
