# HANDOFF — punto de entrada al siguiente agente

Fecha: 2026-10-10 · Sprints: **S00–S21 APROBADOS (batch S21–S25)**.
Siguiente: **S22**. **S26 prohibido sin autorización.**

## Estado científico/arquitectónico actual

- Spec normativa: `docs/foundation.md` (v0.2). Research intactos en raíz.
- 12 crates en workspace (`crates/`): tipos, runner, unidades, grafo de
  montaje, cinemática, contactos, pernos, termo, combustión, `physics`
  aún stub, simulación, CLI. Sin UI. Numéricos con etiqueta de
  verificación (`VERIFIED_NUMERICALLY`), nunca validación.
- `model_assumptions.md` con H-S06..H-S10 registradas. Números OEM D16Y7
  siguen `PENDING` / `MISSING-DATA`; presets ilustrativos etiquetados.
- Evidencia: `docs/EVIDENCE_POLICY.md`. Riesgos: `docs/RISK_MATRIX.md`.
- DoD y rechazo: `docs/definition-of-done.md`. Gate S00:
  `docs/sprints/S00.md`.

## Fuentes principales

- Foundation R01–R20, CF01–CF62 indexados en
  `data/provenance_registry.yaml` (índice, no transcripción de valores).
- Research: `../D16Y7_RESEARCH_v0.1.md`,
  `../D16Y7_RESEARCH_DONANTES_v0.1.md` (rutas desde `docs/`).
- Discrepancias conocidas → `REQUIRES_VERIFICATION`, no promediar
  (ej. 184 vs 135 psi = nominal vs mínimo; P2E genérico vs sufijo completo;
  TD-80U vs TD-98U; 180 vs 240 cc/min sin ΔP).

## Blockers

1. ~~Gate S00 sin verificación~~ — resuelto: S00 APROBADO por propietario
   2026-10-10.
2. ~~Licencia sin decidir~~ — resuelto: Apache-2.0 solo código propio
   (propietario 2026-10-10); `LICENSE` + `THIRD_PARTY_NOTICES.md` creados.
3. S01–S21 APROBADOS 2026-10-10 (S20: veredicto por complementaria;
   S21: ECT 20→20.33 °C, auditoría ramificada). Deuda: viscosidad y
   galerías en S22; mapas/banco pendientes.

## Roles y reglas de cambio (S00)

- Arquitectura / física / validación / datos-CAD / agentes: un responsable
  por PR que toque su área; sin responsable = rechazo.
- Cambios de reglas (charter, DoD, evidence, gates): PR dedicado + ADR.
- Aporte de ecuación: `docs/physics/<subsistema>.md` (nuevo en su sprint)
  + `ModelCard` + fuente/página/dominio + test que falla sin mecanismo.
- Vetting de código de agentes: provenance mandatory, revisión humana,
  CI verde, sin `unwrap` en rutas de simulación sin revisión, sin copiar
  GPL/scans/CAD sin derechos.
- Code review: 1 revisor distinto del autor;/docs + `PROGRESS.md` +
  `HANDOFF.md` actualizados en cada sprint físico (§12.3).

## Cómo reproducir (S00)

```bash
git log --oneline -5
git status --short
ls docs/foundation.md docs/SPRINTS.md docs/PROGRESS.md docs/HANDOFF.md
cat docs/sprints/S00.md
```

## Brief S01 (COMPLETADO)

S01 APROBADO 2026-10-10. Ficha en `docs/variants/`, catálogos en
`data/d16y7/`. Todo número sigue PENDING/DISPUTED/IDENTITY_ONLY.

## Brief S02 (APROBADO 2026-10-10)

Clon fresco + replay MATCH. Detalle en `docs/sprints/S02.md`.

## Brief S03 (APROBADO 2026-10-10)

Tipos SI + `Table1` + `dimensions.md` + `units-demo`. Ver
`docs/sprints/S03.md`.

## Brief S04 (APROBADO 2026-10-10)

Crate `assembly` + catálogo semilla + `assembly-demo`. Ver
`docs/sprints/S04.md`.

## Brief S05 (APROBADO 2026-10-10)

`simulation::history` + `history-demo` (0 divergencias). Ver
`docs/sprints/S05.md`.

## Brief S06 (APROBADO 2026-10-10)

Crate `kinematics` + CLI + ADR-014. Ver `docs/sprints/S06.md`.

## Brief S07 (APROBADO 2026-10-10)

Crate `contacts` + `contact-check`. Ver `docs/sprints/S07.md`.

## Brief S08 (APROBADO 2026-10-10)

Crate `fastener` + `fastener-seal-test` + ADR-015. Ver
`docs/sprints/S08.md`.

## Brief S09 (APROBADO 2026-10-10)

Crate `thermo` + `closed-cycle` + ADR-016. Ver `docs/sprints/S09.md`.

## Brief S10 (APROBADO 2026-10-10)

Crate `combustion` + `closed-cycle-burn` + ADR-017. Ver
`docs/sprints/S10.md`.

## Brief S11 (APROBADO 2026-10-10)

Crate `valvetrain` + `valvetrain` CLI + ADR-018. Ver `docs/sprints/S11.md`.
Nota S12: `lift_at` ahora envuelve a (−360°, 360°] (bug corregido,
re-verificado).

## Brief S12 (APROBADO 2026-10-10)

Crate `flow` + `gas-exchange` CLI + ADR-019. Ver `docs/sprints/S12.md`.

## Brief S13 (APROBADO 2026-10-10)

Crate `fuel` + `fuel-supply-test` + ADR-020. Ver `docs/sprints/S13.md`.

## Brief S14 (APROBADO 2026-10-10)

Crate `crank` + `crank-dynamics` + ADR-021. Ver `docs/sprints/S14.md`.

## Brief S15 (APROBADO 2026-10-10)

Crate `electrical` + `cold-crank` + ADR-022. Ver `docs/sprints/S15.md`.

## Brief S16 (APROBADO 2026-10-10)

Crate `engine` + `engine-run` + ADR-023. Ver `docs/sprints/S16.md`.

## Brief S17 (APROBADO 2026-10-10)

Crate `trigger` + `trigger-waveform` + ADR-024 + raw TRIG-2026-10-10.
Ver `docs/sprints/S17.md`.

## Brief S18 (APROBADO 2026-10-10)

Crate `ecu` + `ecu-demo` + ADR-025. Ver `docs/sprints/S18.md`.

## Brief S19 (APROBADO 2026-10-10)

Cadena sensorial + `sensor-demo` + ADR-026. Ver `docs/sprints/S19.md`.

## Brief S20 (APROBADO 2026-10-10)

Crate `diagnostics` + `diagnose-demo` + ADR-027. Ver `docs/sprints/S20.md`.

## Brief S21 (APROBADO 2026-10-10)

Crate `thermal` + acople + `warmup`/`export-trace` + ADR-028. Ver
`docs/sprints/S21.md`.

## Brief S22 (siguiente en batch autorizado)

Aceite y cojinetes: bomba/alivio, viscosidad vs T, nivel y galerías por
holgura, presión local (no solo global), fricción con lubricación
dependiente del régimen, `film_quality` hacia desgaste futuro. Gate:
fallo de aceite por ruta presión/film/fricción, nunca bandera.
