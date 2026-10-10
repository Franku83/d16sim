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
