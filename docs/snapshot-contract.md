# Contrato de snapshots — S30-T6 (+ remediación R02, 2026-10-10)

- Esquema engine v2 (`schema_version`), wear v1.
- JSON canónico (`engine-types::canonical_json`, claves ordenadas) +
  SHA-256 (`state_hash`).
- **Verificación antes de migrar (R02-F06):** `restore` reconstruye la
  representación canónica *del esquema declarado* y verifica el hash
  **antes** de aplicar cualquier migración. El v1 histórico (anterior a S21)
  no tenía campo `thermal`: su hash se comprueba sobre los bytes v1 y luego
  se migra a v2. Un hash v1 válido ya no se rechaza por añadir `thermal:null`.
- **Compatibilidad (R02-F05):** la ruta de parseo de f64 es lossless
  (`serde_json/float_roundtrip`); el round-trip por disco preserva bits y
  hash. Se separa integridad de archivo (hash) de validez física del estado.
- **Campos opcionales de R01 (R02):** `plen_fuel_kg`, `exh_fuel_kg`,
  `fuel_emitted_kg` y `pmax_pa` se omiten de la forma canónica cuando valen su
  valor neutro (0 o ausente). Los snapshots v1/v2 previos conservan su hash
  histórico; los nuevos serializan el estado físico completo (especies en
  nodos + historia de presión). No hay bump de esquema: la extensión es
  retrocompatible y se lee con `#[serde(default)]`.
- **Validez física fail-closed (R02-F15):** un hash válido no implica estado
  válido. `restore` rechaza masas/temperaturas no positivas o no finitas,
  fracciones fuera de [0,1], combustible fuera de [0, m] y esquemas
  desconocidos; nunca repara en silencio.
- **Replay (R02-F07):** se persiste toda variable con memoria física que
  afecte la trayectoria, incluido `pmax_pa` (máximo de presión de cilindro que
  alimenta el término de presión de Chen–Flynn). El replay continuo y el
  dividido en cualquier corte coinciden bit a bit para `c2 ≠ 0`.
- Compatibilidad declarada: v2 lee v1 (migración verificada). Cambios de
  esquema incompatibles exigen bump + migración testeada + entrada en
  CHANGELOG; esta extensión es compatible, no la exige.
- `docs/export-contract.md` (`trace-bundle-v1`) sigue vigente para el
  spike visual: solo lectura, sin Blender/UI en el core.
