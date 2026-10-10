# ADR-040 — Validación integral de fronteras y compatibilidad semántica de snapshots

- **ID:** ADR-040
- **Fecha:** 2026-10-10
- **Estado:** ACEPTADO (remediación R01–R02-C2)
- **Decisión propuesta:**
  1. **Validación integral de Engine (C1-01):** `validate_inputs` cubre control
     open-loop (AFR, ηv, avance), combustible (LHV, densidad, AFR
     estequiométrico), eléctrico/arranque (tensión, dwell, cutout, motor,
     batería), inercia (J > 0, masa ≥ 0), Wiebe y térmico; `resume` valida
     además nodos (volumen/cv), ledgers, coherencia de red térmica y confirma
     solo un candidato válido. Los errores del starter se propagan; no se
     sustituyen por par cero.
  2. **Energía térmica derivada (C1-02/C1-04):** se valida el producto T·C y la
     suma del ledger, no solo los operandos; `stored_j_checked` y `step`
     rechazan capacidades finitas extremas cuyo estado derivado desborda.
     `restore`/`initial_state`/`run_engine` rechazan configuraciones y
     auditorías no representables antes de devolver `Ok`.
  3. **Dominio diagnóstico (C1-03):** `LeakSplit` es una **descomposición
     completa normalizada** (finita, en [0,1], Σ ≈ 1 con tol 1e-6);
     compresión/leak-down/umbral deben ser finitos y de dominio válido. Una
     observación parcial se expresa con `None`; con datos inválidos o parciales
     el veredicto es `Inconclusive`, nunca una causa.
  4. **Identidad semántica de snapshots (C1-05):** se separa la versión de
     esquema de la convención de contabilidad química mediante `model`
     (`schema_version` + `combustion_accounting`). Un snapshot sin `model`
     (pre-C2) es ambiguo entre R02 y C1: se migra con procedencia aproximada,
     nunca como nativo exacto. Convención desconocida ⇒ rechazo.
  5. **Alcance de integridad (C1-G01):** `state_hash` autentica solo el estado;
     `model` y `provenance` son metadata no autenticada y así se documentan. No
     se firma criptográficamente ni se reescriben hashes históricos.
- **Contexto/motivo:** la verificación independiente C1
  (`a0088ff`, `C1_REMEDIATION_REQUIRED`) reprodujo configuraciones inválidas
  aceptadas, `restore` con capacidad cero/NaN, overflow del ledger térmico,
  diagnóstico discriminando con observaciones inválidas/parciales y un
  snapshot R02 genuino durante combustión sin migración semántica.
- **Alternativas consideradas:** (a) inferir el productor por valores
  (`pmax`/`burn_accounted`) — rechazada por la spec (deducción arbitraria);
  (b) restar 1157 J del ledger para «cuadrar» — rechazada (maquillaje);
  (c) exigir Σ = 1 sin tocar los fixtures históricos — imposible con los
  tests S24/S29 existentes, que se normalizan sin cambiar su causa esperada;
  (d) autenticar la metadata con un hash nuevo — fuera de alcance y
  incompatible con lectores históricos.
- **Consecuencias:** sin cambio numérico en la trayectoria nominal
  (ralentí 8 s dt 50 µs idéntico a C1: 1047.3038737 rpm); solo se rechazan
  entradas/estados inválidos. Los snapshots pre-C2 quedan etiquetados
  aproximados; los C2 nativos son bit-exactos.
- **Trigger de revisión:** nueva evidencia experimental (S25), cambio de
  convención de contabilidad, o adopción de autenticación de metadata.
- **Referencias:** `C1_FINDINGS.md` C1-01…C1-05, C1-G01, C1-D01;
  `C1_GATE_DECISION.md`; `docs/remediation/R01_R02_C2_BOUNDARY_MATRIX.md`;
  `docs/remediation/R01_R02_C2_BATCH_REPORT.md`; ADR-038, ADR-039.
