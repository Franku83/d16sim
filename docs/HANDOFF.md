# HANDOFF — punto de entrada al siguiente agente

Fecha: 2026-10-10 · Sprint activo: **S00 (EN_CURSO, pendiente gate)**.
Siguiente autorizado: **S01 solo tras aprobar S00**. Prohibido adelantar
física/UI.

## Estado científico/arquitectónico actual

- Spec normativa: `docs/foundation.md` (v0.2). Research intactos en raíz.
- Sin código de física, sin crates, sin UI. `model_assumptions.md` vacío
  intencional. Todo número D16Y7 = `PENDING` / `MISSING-DATA`.
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

1. Gate S00 sin verificación por colaborador independiente.
2. Licencia sin decidir (`docs/LICENSE_STATUS.md`).
3. S01 bloqueado hasta gate S00.

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

## Brief S01 (NO iniciar hasta gate S00)

Trabajar **solo** en S01 tras aprobación: fijar única configuración D16Y7
(año/mercado/modelo/transmisión/ECU), auditar OEM por sección, diccionario
tipado con página/edición/estado/incertidumbre, matriz
`CONFIRMED / DISPUTED / UNKNOWN`, protocolo de adquisición. Sin física.
