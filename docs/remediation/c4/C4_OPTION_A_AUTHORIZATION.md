# Autorización explícita de opción A — 2026-10-15

El responsable autorizó formalmente la opción A de C4_SCHEMA_FIXTURE_DECISION.md en esta conversación. Alcance: corregir exclusivamente C3-01…C3-04 en el worktree C4.

Condiciones preservadas:

1. Coherencia estricta esquema/modelo en producción; ninguna excepción para fixtures incoherentes.
2. Solo una copia derivada del fixture afectado recibe `model=None`; estado/hash/parámetros/expectativas/tolerancias no cambian.
3. Cuatro oráculos y fuentes históricas byte-idénticos; tampoco se modifica el checker C3.
4. Ejecuciones literal y adaptada separadas, sin declarar PASS literal del caso14 si sigue fallando.
5. Comprobar v1 genuino sin model aceptado y v1 con model2 rechazado.
6. Registrar diff exacto, SHA256, origen Git y transformación.
7. Exponer la decisión y su limitación para evaluación del revisor original.

La autorización permite esta adaptación; **no** concede gate de integración ni certificación científica. La entrega será IMPLEMENTED_PENDING_INDEPENDENT_REVIEW. No modificar revisor/oráculos originales, no merge/ff a destino ni master, no iniciar R03/R04/R05/S31. Si aparece otro conflicto contractual material, detenerse y presentar alternativas.

Esta aceptación sustituye únicamente la decisión pendiente del preflight, que permanece como evidencia histórica sin sobrescribir su estado original ni sus resultados7/11.
