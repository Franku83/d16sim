# ADR-042 — C4: presión derivada, procedencia compuesta y coherencia estricta

- Fecha: 2026-10-15.
- Estado: IMPLEMENTED_PENDING_INDEPENDENT_REVIEW.
- Base: C3 `3aef5f6ccdc076dad6e8be627a5eb836b26ee998`; hallazgos de36919b0.
- Alcance: C3-01…C3-04; no R03/R04/R05/S31.

## Decisiones implementadas

1. Validar la presión de ambos nodos con la misma operación que consume el Core, además de su densidad. Ningún máximo OEM nuevo; rechazar resultados no representables antes de aceptar estados, también en restore/no-op.
2. Si una nueva limitación requiere migración, crear un registro del snapshot actual y conservar el registro anterior entero en `prior`. Campo opcional de metadata, vacío omitido/default; no participa en state_hash. Alcance conservador: una pérdida previa no se vuelve exacta por default. Idempotencia por identidad del evento actual (esquema/hash/transformación/campos/alcance), preservando su prior. Snapshots completos nativos mantienen su procedencia existente.
3. Restaurar igualdad estricta modelo/payload, sustituyendo la tolerancia de ADR-041§4. Desconocidos e incoherentes se rechazan incluso en frío. V1 genuino sin marcador sigue admitido.
4. Comprobar finitud/no negatividad de la resistencia derivada antes de devolver Ok; SOC válido para la fórmula. Validar también la combinación en Battery::validated (restore/no-op), y propagar el error por terminal/Engine sin convertirlo en voltaje0. No se prohíben Celsius negativos legítimos ni se reajustan nominales.

## Conflicto del fixture y autorización

El fixture del oráculo original captura v2 y reetiqueta el estado a v1, dejando metadata v2 añadida después de su diseño original. Se comprobó igualdad exacta de estado y hash con el v1 genuino2541266; solo difiere model. El responsable autorizó formalmente la opción A: añadir únicamente `v1.model = None` en una copia derivada, sin cambiar física/hash/expectativas/tolerancias. El source histórico y cuatro oráculos quedan byte-idénticos. Se ejecutan y reportan las versiones literal y derivada, incluida la falla literal del caso14.

Alternativas descartadas: permitir discrepancias por temperatura/tiempo/hash; omitir marcadores en capturas nativas para satisfacer tests; modificar oráculos; sustituir resultados literales con adaptados. Ninguna está autorizada ni implementada.

## Compatibilidad y límites

Esquema físico2/accounting1 sin cambio. JSON de metadata anterior se lee con prior vacío; el estado canónico y hashes no cambian. Los literales Rust externos de MigrationProvenance requieren inicializar el nuevo miembro. Metadata sigue no autenticada; no se añade firma ni prueba de autoría. Continuación legacy caliente sigue aproximada y no se resta la discrepancia R02~1157J.

Regresiones: presión finita/overflow/underflow en ambos nodos; pérdida de pmax tras migración genuina y cadena/JSON/recaptura/idempotencia; coherencia estricta y v1 genuino; resistencia extrema/temperatura/atomicidad Engine. Los tests verdes son evidencia del implementador, pendiente de revisión del auditor original. No conceden integración, calibración ni certificación OEM.
