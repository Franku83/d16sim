# Definition of Done + criterios de rechazo — S00

## DoD general (todo sprint físico, §12.3)

Ningún sprint físico se cierra sin:

1. ADR si hubo decisión nueva de modelo/stack.
2. Modelo y ecuación con fuente, unidades, dominio y limitaciones.
3. Parámetros versionados + prueba que falla sin el mecanismo
   (no «mock always passes»).
4. Balance físico si aplica + estudio de convergencia para solver.
5. Escenario CLI reproducible con manifiesto y baseline permitido.
6. Protocolo de validación externa o etiqueta clara de **solo verificado**.
7. Actualizar `docs/SPRINTS.md`, `docs/PROGRESS.md`, `docs/HANDOFF.md`,
   `docs/limitations.md`.
8. Demostración reproducible: comando + output esperado, no screenshot
   como única evidencia.

En S00 los puntos 2–6 son **N/A documentado** (sin física): el DoD de S00 es
documental + gate de comprensión (abajo).

## DoD específico S00

- [x] Repositorio inicial con `README.md`, `docs/foundation.md`,
      `docs/SPRINTS.md`, `docs/PROGRESS.md`, `docs/HANDOFF.md`,
      `docs/limitations.md`, plantilla ADR.
- [x] 12 ADR iniciales convertidos a registros con alternativas y triggers.
- [x] Roles, responsables y reglas de cambio definidos (ver HANDOFF).
- [x] Charter con JTBD §0.1 incorporado (`docs/charter.md`).
- [x] `EVIDENCE_POLICY.md` con etiquetas §23.1 + transferencia §24.1.
- [x] Referencias a ambos research **sin reescribirlos**.
- [x] Matriz de riesgos + backlog documentado.
- [x] CI documental mínima.
- [x] Cero valores numéricos D16Y7 fijados como nominal en archivos nuevos.
- [x] Sin código de física ni UI.

## Protocolo «falta evidencia → PENDING, no inventar»

1. Todo parámetro nuevo entra con `value_si: null`,
   `status: PENDING_VERIFICATION`, `uncertainty.reason` declarado.
2. Prohibido `0.0` como sustituto de desconocido. `null` = desconocido.
3. Discrepancia entre fuentes → `ConflictRecord` + etiqueta
   `REQUIRES_VERIFICATION`. No promediar, no sobrescribir OEM.
4. Clases obligatorias: `[F/D/H/P/X]` + clase de campo
   (`OEM-TEXT`, `OEM-EPC`, `MFR-TECH`, `FORUM-MEASURED`, `FORUM-CASE`,
   `FORUM-CLAIM`, `MISSING-DATA`) + transferencia
   (`EXACT_OEM` / `SHARED_PART` / `ADAPTED_PROXY` / `EXPLORATORY_PROXY`).
5. `calibration_golden` prohibido sin protocolo/metrología completos.

## Criterios de rechazo (el PR/sprint NO pasa si…)

- Introduce un número D16Y7 sin fuente/edición/página/condiciones/estado.
- Convierte `FORUM-CLAIM` en nominal o `DIRECTA` (donante) en `EXACT_OEM`.
- Mezcla años/mercados/transmisiones/ECU en un mismo BOM sin equivalencia.
- Añade solver/UI en S00.
- Promete «≤X% error en HP» sin error de referencia, incertidumbre de
  instrumento y sensibilidad del modelo.
- Usa síntoma→penalización (`if no gasket: engine_destroyed`,
  `if boost: +X hp`) en lugar de estado físico + ecuación.
- Declara «validado» con solo tests de software.
- Persiste output de agente como fuente técnica sin revisión.

## Criterio de utilidad (recordatorio)

Se evalúa por subsistema y dominio de validez; separar causalidad explicable
de precisión predictiva. Una curva bonita con `VERIFIED_NUMERICALLY` no
equivale a medición real.
