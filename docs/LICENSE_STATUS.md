# Estado de licencia — S02 (DECIDIDA)

- **Decisión del propietario (2026-10-10):** `Apache-2.0` exclusivamente
  para el código propio del proyecto.
- Archivo `LICENSE` (raíz): texto Apache-2.0.
- Todo `Cargo.toml` del workspace declara `license = "Apache-2.0"`.
- Documentos de investigación, datos de terceros y futuros assets sujetos
  a sus propias condiciones (ver `docs/EVIDENCE_POLICY.md` §5). No se
  redistribuyen scans del manual Honda ni fotos de foros.
- Dependencias: licencias y atribuciones en `THIRD_PARTY_NOTICES.md`
  (generado desde `Cargo.lock`; solo crates con licencias permisivas
  MIT/Apache-2.0/BSD; sin GPL).
- **Prohibido incorporar código GPL sin revisión y autorización explícita**
  (ADR-011 vigente; Speeduino solo referencia conceptual).
- Marca Honda: uso nominativo.
