# Contrato de snapshots — S30-T6

- Esquema engine v2 (`schema_version`, `#[serde(default)]` migra v1 con
  red térmica a frío/ausente según entradas); wear v1.
- JSON canónico (`engine-types::canonical_json`, claves ordenadas) +
  SHA-256 (`state_hash`); `restore` verifica hash antes de migrar.
- Compatibilidad: v2 lee v1; v1 NO lee v2 (rechazo explícito, nunca
  reinterpretación). Cambios de esquema exigen bump + migración
  testeada + entrada en CHANGELOG.
- `docs/export-contract.md` (`trace-bundle-v1`) sigue vigente para el
  spike visual: solo lectura, sin Blender/UI en el core.
