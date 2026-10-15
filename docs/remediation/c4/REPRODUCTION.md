# Preflight C4 reproducible

Este preflight reproduce el candidato C3 sin corregirlo. Requiere objetos Git de C3 y del commit revisor36919b0, Python con tarfile filter=data, Rust/Cargo y dependencias bloqueadas en cache para --offline.

Desde un checkout de la rama C4 que contenga estos scripts:

```bash
python3 tools/remediation_c4/preflight.py \
  --output docs/remediation/c4/evidence/preflight-nuevo \
  --archive target/c4-preflight-nuevo
```

Usar destinos nuevos y vacíos: el runner se niega a sobrescribir logs o archivos históricos. Archiva producción exacta C3 y extrae el harness exacto del commit revisor; los cuatro oráculos y el checker C3 se contrastan byte a byte. No depende de directorios de build ni fixtures ignorados previos.

Ejecuta seis modos del probe C3, guarda todas las333 observaciones y el checker. El **checker** debe devolver exit1 con7/11 y cuatro fallos; el **runner de reproducción** devuelve0 solo si reproduce exactamente ese baseline y completa el probe adicional de conflicto. No interpretar exit0 del runner como remediación o gate aprobado.

Resultados observados en `evidence/preflight-01/`:

- `c3-summary.json`:11 casos,7 pasan,4 fallan,0 errores,323 predicados,333 filas.
- `fields/thermo/thermal/battery/replay/semantics.jsonl`: observaciones intactas.
- `schema-conflict.jsonl`: versiones/hash/restore y comparación física entre fixture genuino y reetiquetado.
- `integrity.json`: hashes y referencias Git de todos los oráculos/sources preservados.
- `commands.json`: argumentos/cwd/exit/duración de cada ejecución.
- `environment.json`, `completion.json`: plataforma y estado explícito sin fixes.

El source adicional `schema_conflict.rs` no modifica Engine ni actúa como reemplazo de un oráculo. Se compila como otro bin del manifest de auditoría extraído, en el archivo generado dentro de target. `strict_pair_consistent` describe la regla propuesta; no se ha aplicado a producción.

No hay todavía resultados postcorrección ni suites de aceptación C4. No se reclasifican los cuatro fallos como resueltos, no se suprimen assertions, no se alteran gates ni se presenta calibración/validación experimental. Ver C4_SCHEMA_FIXTURE_DECISION.md para la decisión requerida antes de continuar.
