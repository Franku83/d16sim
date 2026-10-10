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
