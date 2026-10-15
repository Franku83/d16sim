# C4 — Conflicto reproducido y decisión previa requerida

**2026-10-15 · PREFLIGHT_PAUSED_PENDING_OWNER_DECISION.** Este documento no concede gate de integración ni presenta un candidato corregido. No se ha modificado producción.

## Identidad y alcance ya ejecutado

- Base exacta: `3aef5f6ccdc076dad6e8be627a5eb836b26ee998`.
- Rama aislada: `remediation/r01-r02-c4`, worktree `/home/franku/projects/d16sim-remediation-c4`, derivada directamente de esa base.
- Evidencia independiente consultada desde `36919b0455e8eaf741a72ff4a74ae85f40e116ba`; leídos los cuatro documentos C3 obligatorios. No se modifica ese worktree ni su rama.
- Master `ff09a0dda82188e6bf27a80d17c902f8972a39d2`, destino R01–R02 `8a7e8686e4e8831fb30d1a71b815e67e7d7139ef` y rama C3 preservados.
- Nueva reproducción sobre código C3 archivado desde Git, sin depender de fuentes ignoradas previas: **7/11 controles dirigidos, cuatro fallos, cero errores;323 predicados y333 observaciones**. Los cuatro fallos son los IDs documentados C3-01…C3-04.
- Los cuatro oráculos y el checker dirigido C3 están byte-idénticos a la referencia Git; hashes en `evidence/preflight-01/integrity.json`. También se conservan los sources de pruebas; se agrega únicamente un bin de observación nuevo al manifest de auditoría generado, sin modificar el Core ni las expectativas.

No se ha ejecutado una aceptación postcorrección, ni se declara209/209 como nueva ejecución: no hay correcciones aplicadas. Las suites completas Rust/fmt/clippy/oráculos siguen pendientes para después de resolver esta decisión. Los resultados anteriores permanecen como evidencia histórica, no se reemplazan.

## Conflicto exacto de C3-03

El oráculo original tiene un caso válido de compatibilidad histórica:

`verification/c3_review/check_contracts.py::test_14_actual_v1_hash_can_migrate` espera `v1_hash_migration.accepted == true`.

Pero el productor del fixture quedó desactualizado al añadirse metadata en C2:

`36919b0:verification/c3_review/src/main.rs`, líneas318–321:

```rust
let mut v1 = cold(&i);              // capture(state schema2) añade model.schema_version=2
v1.state.schema_version = 1;       // cambia payload, pero no el marcador añadido después
// recalcula hash canónico v1 del estado
```

La regla solicitada de coherencia rechaza model2/payload1. La expectativa histórica de aceptar un v1 genuino es legítima; el marcador agregado por el constructor actual a ese fixture no existía en el productor histórico. Por tanto, el conflicto no prueba una incompatibilidad del v1 genuino: está en la representación del fixture y en si se permite adaptar esa representación.

El nuevo probe (`tools/remediation_c4/schema_conflict.rs`) produce, **sin cambiar producción**, lo siguiente:

| Entrada | Payload | Model | Hash válido | Restore C3 | Cumple igualdad estricta |
|---|---:|---|---|---|---|
| Fixture reetiquetado original |1|{schema2,accounting1}|Sí|Acepta|No|
| v1 genuino de2541266 |1|Ausente|Sí|Acepta|Sí, no hay marcador incoherente|
| v2 coherente |2|{schema2,accounting1}|Sí|Acepta|Sí|
| v2 incoherente |2|{schema1,accounting1}|Sí|Acepta|No|

**V1 genuino y fixture reetiquetado tienen exactamente el mismo EngineState y state_hash**:

`deb04a1b413455e20c057ec5cca14c28d691ab52630cef28d172c9224b0dea93`.

También tienen la misma provenance ausente. Solo difiere model. El fixture genuino está rastreado en la revisión y procede del tipo/solver2541266; no se reconstruyó JSON a mano. Esto permite una solución estrecha sin modificar física, tolerancias ni hashes.

`strict_pair_consistent` es un predicado diagnóstico explícito, no un fix aplicado a restore. Que el caso14 original pasaría a fallar bajo rechazo estricto es una **deducción** de esas entradas y expectativas, no un resultado atribuido a un Core corregido que no se ha ejecutado.

## Alternativas concretas para el responsable

### A — Adaptador explícito del fixture v1, igualdad estricta en producción (recomendada)

1. Restaurar igualdad de versiones en el Core para todo model presente, rechazando conocidos incoherentes y desconocidos; mantener lectura legítima de v1/v2 sin model.
2. Autorizar una única adaptación del productor antiguo de fixture para representar fielmente v1: al reetiquetar el payload a1, poner **solo `v1.model = None`**, porque el tipo histórico no tenía ese campo. No cambiar estado físico, hash, condiciones iniciales, expectativas ni tolerancias.
3. Mantener los cuatro archivos de oráculos byte-idénticos. Mantener intacto el source histórico del fixture y su ejecución original; entregar el adaptador separado, su diff mecánico y hashes.
4. Ejecutar y conservar por separado el harness sin adaptar y el adaptado. El original estricto mostraría el conflicto; el adaptado debe demostrar aceptación del v1 genuino, no convertir una entrada incoherente en válida por excepción del Core. No ocultar el rojo original.
5. Agregar regresiones explícitas de pares incoherentes fríos/calientes y de archivos genuinos. Revisión independiente debe aceptar la adaptación antes de conceder gates.

Esta alternativa necesita autorización porque afecta un fixture independiente congelado, aunque no cambie ningún checker/oráculo ni su expectativa. No se aplicó unilateralmente.

### B — Conservar todos los fixtures literalmente y registrar la incompatibilidad

Aplicar rechazo estricto en el Core y conservar el caso14 antiguo como fallo explícito de representación; validar v1 genuino en una prueba adicional. Requiere que el responsable permita evaluar esa incompatibilidad separadamente de una regresión de lectura histórica. No cambiar ni omitir su assertion, ni presentarlo como verde; el objetivo agregado de las suites requeriría esa decisión explícita. No se ha adoptado esta política.

No recomiendo excepciones implícitas por tiempo0, un hash específico o valores «parecidos al fixture»; tampoco omitir model en capturas nativas para conseguir verde. Estas soluciones desplazarían la inconsistencia a la semántica productiva.

## Correcciones mínimas preparadas como plan, no implementadas

- **C3-01:** comprobar presión de ambos NodeW en validate_derived_thermo, finita y >0, usando la misma convención m·R·T/V que consume el solver. Cubrir overflow/underflow, densidad y control de presión finita extrema, restore/no-op/avance; conservar copia candidata y errores tipados.
- **C3-02:** componer un límite nuevo de continuidad cuando falta historia requerida aunque exista provenance; preservar íntegramente el origen previo y registrar la nueva transición en lugar de sobrescribirla u omitirla. Añadir regresiones de procedencia encadenada, recaptura/round-trip y ausencia de duplicación al restaurar repetidamente. Cualquier extensión de metadata debe ser opcional, sin tocar state_hash ni desvirtuar la historia anterior.
- **C3-03:** solución A o B según decisión; no debilitar coherencia para conservar un fixture desactualizado.
- **C3-04:** validar resultado derivado de resistencia y propagar error antes de que un infinito sea limitado a voltaje0. Conservar positivos finitos,−20°C, dominio absoluto y arranque nominal; no imponer cotas OEM arbitrarias.

Tras decidir, añadir regresiones rojas antes de cada fix, realizar cambios mínimos y ejecutar controles C3/C2/C1/adversarial/S30, Rust workspace, fmt y clippy. Conservar los diez pendientes R03/R04 sin maquillarlos. Esta preparación no autoriza R03/R04/R05/S31, merge ni fast-forward.

## Motivo de parada

La parada procede de la instrucción explícita del usuario:

> «Si alguna corrección entra en conflicto legítimo con un contrato anterior, detente y presenta las alternativas; no cambies unilateralmente la política.»

No procede de un requisito de la skill diagnose ni de un rechazo automático de herramientas. Se necesita decidir el tratamiento del fixture; no una autorización para integración.
