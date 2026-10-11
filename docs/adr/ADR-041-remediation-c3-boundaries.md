# ADR-041 — Validación de leva/inyector, termodinámica derivada e identidad semántica estricta

- **ID:** ADR-041
- **Fecha:** 2026-10-10
- **Estado:** ACEPTADO (remediación R01–R02-C3)
- **Decisión propuesta:**
  1. **Leva e inyector (C2-01):** `validate_inputs` valida los parámetros
     públicos activos del inyector (`q_ref`, `p_ref`, mapa de deadtime,
     incertidumbre) y de ambas levas (duración advertised/at050 con
     `at050 ≤ adv`, alzada, flanco, centerline, LSA) antes de run/restore/
     resume; un valor inválido no puede degradarse a alzada o inyección cero.
  2. **Termodinámica derivada (C2-02):** además de los operandos, se valida
     `T = U/(m·cv)` en el estado y presión/densidad derivadas en las fronteras
     de restore/resume/run_engine (incluido avance nulo).
  3. **Energía térmica dependiente del estado (C2-03):** `validate_inputs`
     valida solo la validez intrínseca de `ThermalParams`; la energía
     representable se comprueba contra el **estado real** en cada API
     (`validate_physical_state`, `initial_state`). No se impone `cold(20)` a
     un restore/resume con otro estado válido.
  4. **Identidad semántica estricta (C2-04):** cualquier `model` presente debe
     ser conocido (esquema 1/2 y contabilidad actual) en todos los esquemas;
     desconocido ⇒ rechazo. Sin `model` se conserva la lectura histórica.
     Tolerancia documentada: no se exige igualdad exacta modelo↔payload
     (`schema_version`) porque el fixture de migración v1 del oráculo
     independiente captura bajo esquema 2 y reetiqueta el payload a 1.
  5. **Historia de presión requerida (C2-05):** la continuación es nativa
     exacta solo con modelo conocido, esquema actual y `pmax` presente cuando
     la fricción depende de presión (Chen–Flynn `c2 ≠ 0`); si falta, se
     etiqueta `approximate_hot`. No se reconstruye presión histórica.
  6. **Dominio térmico de batería (C2-06):** la temperatura Celsius admite
     valores negativos por encima del cero absoluto; las resistencias siguen
     exigiendo finitud y no negatividad. El comportamiento nominal de arranque
     no cambia.
- **Contexto/motivo:** verificación independiente C2 (`40e75d5`,
  `C2_REMEDIATION_REQUIRED`) reprodujo cinco incumplimientos y un residual:
  parámetros de leva/inyector sin validar, temperatura derivada no finita,
  rechazo indebido de energía térmica válida, identidad desconocida admitida
  en v1 y pérdida de `pmax` clasificada como nativa exacta.
- **Alternativas consideradas:** (a) cota OEM de capacidades — rechazada;
  (b) rechazar toda falta de `pmax` — rechazada: con c2=0 no afecta la
  trayectoria; (c) igualdad estricta modelo↔esquema — rechazada: rompería el
  fixture de migración v1 del oráculo original; (d) inferir el productor por
  valores — rechazada por la spec.
- **Consecuencias:** sin cambio numérico (ralentí nominal idéntico); estados
  pre-C2 incompletos quedan explícitamente aproximados; snapshots nativos
  completos siguen bit-exactos.
- **Trigger de revisión:** nueva evidencia experimental, cambio de convención
  de contabilidad o de la política de autenticación de metadata.
- **Referencias:** `C2_FINDINGS.md` C2-01…C2-06; `C2_GATE_DECISION.md`;
  `docs/remediation/R01_R02_C3_ACCEPTANCE_MATRIX.md`;
  `docs/remediation/R01_R02_C3_BATCH_REPORT.md`; ADR-038/039/040.
