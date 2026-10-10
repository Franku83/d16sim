# Estado de licencia — S00 (por decidir)

No hay archivo `LICENSE` en S00 por diseño: no hay código que licenciar y
la decisión requiere criterio del propietario (propietario vs copyleft vs
permisiva + contenido/CAD por separado).

## Opciones a decidir en S01 o antes de S02 (primer código)

- Código: MIT / Apache-2.0 (permisivas, compatibles con aviso Engine-Sim
  MIT y Cantera BSD-3) vs GPL-2.0+ (si se derivara Speeduino — **no
  previsto**; la regla es implementación independiente).
- Contenido (CAD, esquemas, manuales): licenciado independientemente del
  código; arte propio, sin scans Honda ni fotos de foros sin permiso.
- `THIRD_PARTY_NOTICES.md`: obligatorio desde S02 (Engine-Sim MIT,
  Cantera BSD-3, Speeduino GPL-2.0 como referencia-no-vinculada, OpenWAM
  pendiente de auditoría).

## Restricciones vigentes ya

- No copiar firmware Speeduino ni activos Honda sin revisar licencias
  (ADR-011).
- No monetizar/etiquetar resultados como ingeniería certificada (ADR-012).
- Marca Honda: uso nominativo.

Decisor: propietario del repositorio. Fecha límite: antes del primer commit
con código (S02).
