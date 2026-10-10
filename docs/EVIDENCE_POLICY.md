# EVIDENCE_POLICY — S00

Síntesis operativa de `docs/foundation.md` §0 (convenciones), §23.1 (clases
de campo) y §24.1 (transferencia). No crea datos nuevos.

## 1. Etiquetas de estado del Foundation [F/D/H/P/X]

| Etiqueta | Significado | Uso en S00 |
|---|---|---|
| `[F]` Fuente verificada | Referencia técnica/manual localizado; falta comprobar edición/versión/aplicación/alcance | Solo si hay R/CF + edición; si no, degradar a `[P]` |
| `[D]` Deducción de ingeniería | Decisión/consecuencia razonada; hay que probarla | Aritmética verificable, nunca como medición |
| `[H]` Hipótesis/modelo simplificado | Aproximación sujeta a calibración/sustitución | Etiquetar + registrar en `model_assumptions.md` |
| `[P]` Pendiente | Documentación insuficiente; no fijar valores | Default para todo número D16Y7 en S00 |
| `[X]` No soportado | Fenómeno reconocido pero no modelado/diagnosticable | Ir a `limitations.md` |

## 2. Clases de campo (§23.1)

| Etiqueta | Qué demuestra | ¿Fixture numérico directo? |
|---|---|---|
| `OEM-TEXT` | Procedimiento/rango en reproducción de manual Honda | Sí, condicionado a año/variante/edición/método; confirmar en S01 |
| `OEM-EPC` | Existencia/número/aplicación de pieza | Solo identidad/compatibilidad, no dinámica |
| `MFR-TECH` | Mecanismo/método de fabricante técnico | Sí como ley general, no como coeficiente Honda |
| `FORUM-MEASURED` | Valor medido con procedimiento descrito | Como observación de campo con incertidumbre; no nominal |
| `FORUM-CASE` | Síntomas/intervención/desenlace | Como caso adversarial/hipótesis; no causalidad demostrada |
| `FORUM-CLAIM` | Consejo/anécdata/cifra sin método | Solo pista; no parámetro productivo |
| `MISSING-DATA` | Requiere banco/despiece/metrología | No; abrir tarjeta de adquisición |
| `REQUIRES_VERIFICATION` | (extensión S00) conflicto no resuelto | Bloquea calibración de esa feature |

Por cada publicación guardar: URL, fecha, autor (opcional), resumen sin
reproducción extensa, motor, ECU/transmisión, stock/modificado, condiciones,
método/instrumento, unidades, incertidumbre, estado (resuelto/probable/sin
resolver), fotos/gráficas disponibles, licencia. No asumir que «solved»
confirma mecanismo.

## 3. Transferencia (§24.1 + donantes)

Mantener juntas `[F/D/H/P/X]` + clase de campo + relación con objetivo:

- `EXACT_OEM` (año/parte/condición) · `SHARED_PART` (identidad demostrada) ·
  `ADAPTED_PROXY` (diferencias + transformación + sensibilidad) ·
  `EXPLORATORY_PROXY` (orden de magnitud).
- Donantes: `DIRECTA / ESCALADA / ANALOGÍA`. **No interpretar `DIRECTA`
  como `EXACT_OEM`** sin identidad de pieza. Misma fundición o geometría
  75×90 no demuestra misma leva/válvula/distribuidor/inyector/ECU.
- Por transferencia guardar: `target_variant`, `donor_variant`,
  `part_number`, `assumed_shared_features`, `known_differences`,
  `original_test_conditions`, `adaptation_transform`, `uncertainty_basis`,
  `source_id`, `evidence_class`, `supersession_rule` + sensibilidad.
- Plausibilidad ≠ calibración. Rechazar `calibration_golden` sin protocolo.

## 4. Esquema mínimo por parámetro (S01 en adelante)

```yaml
parameter_id: d16y7.geometry.bore
value_si: null
unit: m
uncertainty: {type: unknown, reason: source_page_not_reviewed}
source: {id: R01, publication: Civic Service Manual 1996-2000, page_or_section: null}
variant: US_1996_1998_D16Y7_MT_UNCONFIRMED
measurement: {method: documentation}
status: PENDING_VERIFICATION
```

## 5. Licencias y contenido de terceros

- Manual Honda: referencia, no redistribuir scans/PDF/diagramas. Resumir
  hechos, arte propio, revisión legal antes de distribuir.
- Speeduino GPL-2.0: referencia conceptual, implementación independiente;
  auditar contaminación accidental.
- Engine-Sim MIT: cumplir aviso; no es instrumento científico.
- Cantera BSD-3-Clause: verificador, examinar dependencias.
- OpenWAM (CMT-UPV): licencia **no confirmada** — no integrar antes de
  auditar. No confundir con homónimos de IA.
- Foros/fotos/dynos: cifras = hechos publicados (resumir/parafrasear);
  no redistribuir fotos/textos largos sin permiso.
- Marca Honda: uso nominativo, sin inferir autorización comercial.

## 6. Conflictos

Ante discrepancia: crear `ConflictRecord {parameter, claim_a, claim_b,
units, conditions_a/b, variant_a/b, uncertainty, decision, reviewer}`.
Si no se explica por régimen/T/variante/instrumento, bloquear calibración.
No promediar nominal vs mínimo (ej. 184 vs 135 psi son campos distintos).
