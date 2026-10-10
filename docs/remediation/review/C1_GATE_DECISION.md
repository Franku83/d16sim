# Gate independiente R01–R02-C1 e integración condicional

**Decisión: `C1_REMEDIATION_REQUIRED`. Fast-forward NO ejecutado.**

Origen verificado: `caad1cbf31d01531e0fbd81be6190f52b2d1c2d7` (`remediation/r01-r02-c1`). Destino preservado: `8a7e8686e4e8831fb30d1a71b815e67e7d7139ef` (`remediation/r01-r05`). Revisión anterior preservada: `1d614378dd8481acb2a513b4c6dc6572345e5c66`. Nueva evidencia en la misma rama revisora, commit separado de implementación; SHA final comunicado en la entrega y disponible mediante git rev-parse.

## Condiciones

| Criterio requerido | Decisión y evidencia |
|---|---|
| RV01–RV09 cumplen contratos | **NO:** RV02/RV04/RV05/RV06 parcialmente corregidos; C1-01/02/03/04/05 |
| F01/F02/F05/F06/F15 cerrados o acotados sin comprometer exigencias | **NO:** F02/F06/F15 siguen comprometidos; F01/F05 cerrados en dominio nativoC1 probado, F06 pre-R02 pasa pero estado8a caliente no queda etiquetado |
| F07/F08/F09/F10 conservados | Sí: replay54 comparaciones completas, fresh22, líquidos8, térmico33 y fronteras válidas adicionales |
| Ninguna nueva regresión material | No se demuestra nueva regresión física P0; defectos de Engine/Thermal son residuales8a; metadata nuevo requiere política de confianza |
| Oráculos y experimentos intactos | Sí: checks byte-exactos, diffs y SHA256; helper privado extraído de C1 |
| Resultados originales reproducibles | Sí: original22/32 y adversarial19/19; reproducción desde Git y productores genuinos |
| Diez fallos históricos correctamente atribuidos | Sí: exactamente17/18(F03/F04,R04),19/20(F11/F12,R03),21/22(F13,R03),23(F14,R03),28(F16,R03),31/32(F18/F19,R03) |
| Foundation/gates históricos intactos | Sí; revisora añade solo evidencia. Implementador añade adendas identificables |
| Destino ancestro y ramas limpias | Sí en comprobaciones de identidad; no basta para sustituir el gate científico fallido |

Los diez rojos históricos **no** bloquean por sí solos C1. Los nuevos defectos residuales sí lo bloquean. Un19/19 del harness anterior no autoriza reducir el alcance de F15 ni de las observaciones incompletas a esos fixtures.

## Acciones necesarias antes de reconsiderar integración

1. **C1-01/RV05:** completar validación de EngineInputs/WorkState y coherencia térmica, propagar errores de starter, impedir RunTrace/ledgers no finitos. Mantener rollback ante errores dinámicos demostrado.
2. **C1-02/RV06:** validar ThermalParams y energía derivada en restore y resume antes de devolver/confirmar; snapshots con capacidad0 y wall_fracNaN deben rechazarse.
3. **C1-03/RV02:** protocolo público debe rechazar o declarar Inconclusive con datos no finitos, fracciones imposibles o reparto parcial. Conservar el70% de escape físico y la ausencia honesta de canales. No insertar truth labels para discriminar.
4. **C1-04/RV06:** rechazar overflow del ledger térmico aun cuando cada temperatura/capacidad sea finita, o formalizar un dominio representable que lo excluya antes de mutar. No ajustar umbrales estacionarios ni tolerancias para aprobar Inf.
5. **C1-05/RV04:** separar snapshots R02 de contabilidad diferida y snapshots C1 de consumo por paso; registrar procedencia/continuidad aproximada cuando la historia necesaria no puede reconstruirse. Mantener lectura/hash históricos y replay nativoC1 completo; no inferir versión del solver solo de pmax.
6. Reejecutar ambos oráculos intactos, controles nuevos y matrices corregidas; documentar límites de confianza del metadata (C1-G01) y restringir claims amplios (C1-D01). Estos últimos no sustituyen las correcciones de los defectos demostrados.

Estas son recomendaciones para una futura fase autorizada: **no se implementaron**. La sospecha AUD-R01, K01–K03, G01–G07 y los defectos R03/R04 siguen abiertos; no se exige resolverlos clandestinamente dentro de C1.

## Estado de Git y preservación

La cadena comprobada, sin divergencia: 8a7e868→23e48eb→edb36cc→caad1cb. Se consultaron SHAs completos y padres antes de ejecutar pruebas. No se llamó merge, reset, rebase, squash, force ni eliminación de ramas/worktrees. No existe smoke test posterior a integración porque no hubo integración; los gates ejecutados sobre C1 y los cuatro CLI sirven como smoke de la implementación examinada.

- `master`/worktree principal: ff09a0d; spec no rastreada preexistente conservada.
- `audit/independent-2026-10-10`:25d528e, limpio y seis documentos originales intactos.
- `remediation/r01-r05`:8a7e868, limpio, sin fast-forward.
- `remediation/r01-r02-c1`:caad1cb, limpio, sin cambios del auditor.
- `verification/r01-r02-2026-10-10`: nueva evidencia sobre1d61437, sin diffs productivos; limpio tras commit de entrega.
- `d16sim-verification-c1`: detachedcaad1cb; archivos de ensayo/resultados nuevos no rastreados declarados; producción rastreada intacta. Se conserva.

El estado completo antes del commit de evidencia está registrado en git-review-state.json; el SHA final y la comprobación posterior al commit se comunican en la entrega. La posibilidad técnica de fast-forward sigue disponible, pero su autorización condicional no se satisface.

**STOP.** No se inicia R03/R04/R05/S31 ni se fusiona a master. No se modifican retrospectivamente gates S16/S24/S30: se recomienda reevaluar los requisitos concretos de observación válida y fail-closed implicados en F02/F15.
