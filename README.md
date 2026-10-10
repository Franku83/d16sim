# Engine Engineering Sandbox — D16Y7

Laboratorio virtual de ingeniería de motores. Plataforma piloto: Honda D16Y7
(SOHC 16V non-VTEC). Estado: **research/foundation, sin motor físico
implementado, calibrado ni validado**.

## Fuente de verdad

- `docs/foundation.md` — copia versionada del Foundation
  (`ENGINE_ENGINEERING_SANDBOX_D16Y7_RESEARCH_FOUNDATION.md`,
  Research/Foundation v0.2, 2026-10-09).
- Research complementario, **íntegro e independiente** (no reescrito, no
  sustituido por la adenda §24):
  - `../D16Y7_RESEARCH_v0.1.md`
  - `../D16Y7_RESEARCH_DONANTES_v0.1.md`
- Trazabilidad de ejecución: `docs/SPRINTS.md`, `docs/PROGRESS.md`,
  `docs/HANDOFF.md`.

## Estado de sprints

- **S00 (en curso):** charter, alcance científico y reglas del repositorio.
  Sin código de física. Ver `docs/sprints/S00.md`.
- **S01–S37: pendientes.** No adelantar. Ver `docs/SPRINTS.md`.

## Estructura

```text
docs/
  foundation.md          # spec versionada (fuente normativa de reglas)
  charter.md             # JTBD §0.1
  definition-of-done.md  # DoD + criterios de rechazo
  EVIDENCE_POLICY.md     # etiquetas [F/D/H/P/X] + clases OEM/FORUM + transferencia
  limitations.md         # no-soportado y dominios de extrapolación
  model_assumptions.md   # libro de hipótesis (vacío en S00: sin modelos)
  RISK_MATRIX.md         # matriz de riesgos
  BACKLOG.md             # backlog documental (sprints como backlog, no tickets de física)
  SPRINTS.md             # dependencias, estado y gates
  PROGRESS.md            # log objetivo de cambios ejecutados
  HANDOFF.md             # estado actual + cómo continuar
  LICENSE_STATUS.md    # licencia por decidir
  adr/                   # ADR-001..012 + plantilla
  sprints/S00.md         # ficha de ejecución S00
data/
  provenance_registry.yaml  # catálogo de fuentes Rxx/CFxx (índice, no valores)
```

## Reglas inviolables (resumen S00)

1. Ningún número sin fuente + condiciones + clase de evidencia.
2. `null` ≠ 0. Desconocido = `PENDING` / `MISSING-DATA`, nunca constante
   de conveniencia silenciosa.
3. Verificación ≠ validación ≠ calibración. No declarar validez física por
   tener tests de software.
4. Fallo = estado físico + ecuaciones, nunca `if` punitivo ni slider `+HP`.
5. Headless determinista primero. Sin 3D/UI hasta la Fase D.
6. Este software **no es apto para decisiones de seguridad real**
   (tuning, reparación, certificación) hasta validaciones apropiadas.
   Ver `docs/charter.md` y `docs/limitations.md`.

## Reproducir (S00)

S00 no exige toolchain numérico. Verificación documental:

```bash
git log --oneline -5
ls docs/foundation.md docs/SPRINTS.md docs/PROGRESS.md docs/HANDOFF.md \
   docs/charter.md docs/definition-of-done.md docs/EVIDENCE_POLICY.md \
   docs/limitations.md docs/model_assumptions.md \
   data/provenance_registry.yaml
# CI documental (si hay network/actions): .github/workflows/docs-check.yml
bash -c 'test -f docs/foundation.md && echo FOUNDATION_OK'
```

## Licencia

Pendiente de decisión. Ver `docs/LICENSE_STATUS.md`. No hay código que
licenciar en S00; no copiar firmware Speeduino (GPL-2.0) ni scans del manual
Honda ni CAD de terceros.
