# HANDOFF — punto de entrada al siguiente agente

Fecha: 2026-10-10 · Sprints: **S00–S26 APROBADOS (batch S26–S30 en curso)**.
**S31 prohibido sin autorización explícita.**

## Estado científico/arquitectónico actual

- Spec normativa: `docs/foundation.md` (v0.2). Research intactos en raíz.
- 25 crates en workspace (`crates/`, reconciliado 2026-10-10; antes
  decía 12 por desactualización): assembly, calibration, cli, combustion,
  contacts, crank, diagnostics, ecu, electrical, engine, engine-types,
  fastener, flow, fuel, kinematics, leak, lube, math, physics,
  simulation, thermal, thermo, trigger, units, valvetrain. Sin UI.
  Numéricos con etiqueta de verificación (`VERIFIED_NUMERICALLY`),
  nunca validación.
- `model_assumptions.md` con H-S00 y H-S06–H-S25 registradas (antes
  decía solo H-S06–H-S10 por desactualización). Números OEM D16Y7 siguen
  `PENDING` / `MISSING-DATA`; presets ilustrativos etiquetados.
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
3. S01–S25 APROBADOS 2026-10-10 (reconciliado; antes decía S01–S22
   por desactualización). S23: suministro regulado + leyes explícitas.
   S24: protocolo de 2 mediciones. S25: cierre NUMERICALLY_VERIFIED_ONLY
   con 8 vacíos P1 (sin dyno inventado). Deuda: desgaste (S26), daños
   por montaje (S27), dyno auditable (S28), suite adversarial (S29),
   auditoría final (S30).

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

## Brief S22 (APROBADO 2026-10-10)

Crate `lube` + `lubrication-test` + ADR-029. Ver `docs/sprints/S22.md`.

## Brief S23 (APROBADO 2026-10-10)

Crate `leak` + `leak-test` + ADR-030. Ver `docs/sprints/S23.md`.

## Brief S24 (APROBADO 2026-10-10)

`diagnostics::mech` + `mech-demo` + ADR-031. Ver `docs/sprints/S24.md`.

## Brief S25 (APROBADO 2026-10-10, NUMERICALLY_VERIFIED_ONLY)

Crate `calibration` + `calibration-demo` + `docs/calibration/` + ADR-032. Ver `docs/sprints/S25.md`.

## Brief S26 (APROBADO 2026-10-10, TREND_CONDITIONED)

Crate `wear` + `wear-demo` + ADR-033. Ver `docs/sprints/S26.md`.

**Reconciliación explícita Fase 0 (no sustitución silenciosa):** este brief
decía «calibración stock…» por un copia-pega de S25. La fuente normativa
(Foundation §15, S26 «Motor persistente: desgaste y mantenimiento bajo
incertidumbre») y `SPRINTS.md` (S26 = desgaste) concuerdan: **S26 es
desgaste**, y la calibración queda en S25 donde pertenece. Contenido
correcto: slow-state persistente por pieza, leyes de desgaste solo con
parámetros defendibles (resto `UNSUPPORTED`), acople T/contacto/fricción,
eventos de servicio, aceleración de tiempo sin dependencia de FPS,
replay desde snapshot/seed. Gate: leyes/provenance, sin evidencia solo
tendencias condicionadas. Dep: S21–S25.

## Brief S27 (AUTORIZADO, batch S26–S30)

Contactos, plasticidad/fatiga y daños por montaje: pistón-válvula, rosca,
over-stretch, pérdida de película; `DamageMechanism` con dominio;
irreversible solo por criterio físico; pruebas negativas. Gate: sin
instante de fallo real sin material/geometría/calibración. Dep:
S07–S08, S11, S22, S26.

## Brief S28 (AUTORIZADO, batch S26–S30)

Dyno auditable: freno con inercia/carga, rampas, pérdidas, ambiente,
potencia al freno vs indicada vs pérdidas. Gate: HP desde torque
integrado, nunca circular desde spec. Dep: S14–S27.

## Brief S29 (AUTORIZADO, batch S26–S30)

Suite adversarial: pares de fallos, invariantes, no-linealidad compuesta,
observabilidad, conmutatividad, Monte Carlo con seed fija. Gate: cada
síntoma remonta a ecuaciones; sin `if fault_count>1`. Dep: S23–S28.

## Brief S30 (AUTORIZADO, batch S26–S30)

Auditoría científica y congelación: ModelCards, reproducibilidad en 2
máquinas × 3 perfiles, 4 cadenas causales, benchmark CPU, versionado,
revisión experta. Gate: sin «predictivo» sin evidencia; incompleta pero
honesta puede pasar. Dep: S00–S29.
