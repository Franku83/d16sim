# Gate independiente R01–R02-C4

**2026-10-15. Gate A formal: READY_FOR_DEVELOPMENT_INTEGRATION. Gate B: VERIFIED_WITHIN_TESTED_DOMAIN para los contratos especificados y sus restricciones.**

Esta decisión se registra **antes de cualquier integración** en un commit exclusivo de verificación. El permiso humano autoriza un fast-forward condicionado a este resultado; no se vuelve a pedir una autorización ya concedida. R03/R04/R05/S31 y merge a master siguen prohibidos.

## Gate A — criterios

| Condición | Evidencia/decisión |
|---|---|
| C3-01…C3-04 verificados | Sí; cuatro cierres por causa, oráculos C3 11/11 y ampliación C4 8/8 |
| Opción A legítima/trazable | Sí; única línea model=None en copia, source/checkers intactos, estado/hash igual al v1 genuino |
| Literal/adaptado separados | Sí;21/32 y22/32, caso14 literal FAIL conservado, única observación diferente accepted |
| Sin nuevas regresiones materiales | No se demostraron en el dominio ensayado; físicos/balances/replay conservados |
| Sin P0/P1 conocidos abiertos dentro de R01–R02 que impidan integrar | Cuatro hallazgos de C3 cerrados; no se encontró otro en las pruebas focalizadas. No es garantía universal |
| Residuales P2/P3 del baseline requieren aceptación | No se identificaron nuevos residuales que requieran aceptación adicional. P2 previos corregidos; opción A tiene autorización humana explícita |
| Foundation/gates históricos intactos | Sí, fuentes Git contrastadas; no se reescribe el historial de fallos |
| FF lineal y seguro | Cadena8a→C1→C2→C3→ed56→8c, source/destino limpios; revalidación obligatoria inmediata antes de ejecutar |

Los diez rojos R03/R04 son pendientes explícitos fuera de alcance. El undécimo rojo literal es el conflicto del fixture cuya adaptación estrictamente delimitada fue autorizada y aceptada independientemente; no se oculta.

## Gate B — separado de la integración

| Contrato | Evaluación |
|---|---|
| F01/F02/F05 | VERIFIED_WITHIN_TESTED_DOMAIN: química/especies, observación física y bits/hashes/JSON ensayados |
| F06 | VERIFIED_WITHIN_TESTED_DOMAIN: histórico genuino, coherencia estricta, clasificación actual/prior, idempotencia y límites; literal/adaptado separados |
| F15 | VERIFIED_WITHIN_TESTED_DOMAIN: matrices públicas/representabilidad y errores atómicos dentro de casos probados |
| F07/F08/F09/F10 | VERIFIED_WITHIN_TESTED_DOMAIN: nativos completos/multicorte, especie fresca, líquidos de API y ledger ambiental |
| RV07/RV08/RV09 | VERIFIED_WITHIN_TESTED_DOMAIN: ECU, dimensiones de callback y corrección dyno; no cierra R03/R04 |

No equivale a simulador predictivo, datos OEM exactos, calibración ni validación experimental. Permanecen AUD-R01,K01–K03,G01–G07 y F03/F04/F11–F14/F16/F18/F19. Metadata no autenticada, lectores antiguos pueden perder prior, compatibilidad JSON no garantiza literales Rust externos. Véanse dominios y cobertura en el informe consolidado.

## Operación autorizada y SHAs

- Origen C4 aprobado: **8c51b5d9c705745b1a558b00e8bc41d73aad0d7f**.
- Destino antes: **8a7e8686e4e8831fb30d1a71b815e67e7d7139ef**.
- Destino autorizado: únicamente `remediation/r01-r05`.
- Resultado que debe obtenerse después: HEAD destino exactamente igual al origen aprobado.

Antes de operar: comprobar otra vez HEAD/ref C4 y destino, ascendencia, limpieza y producción sin archivos no rastreados. Si falla alguna condición, detener sin cambiar estrategia. Única operación permitida: `git merge --ff-only` del SHA aprobado en el worktree del destino. Sin rebase/reset/squash/force/merge commit; nunca master.

**Estado al registrar esta decisión: INTEGRATION_PENDING_FINAL_GIT_CHECKS.** El registro posterior añadirá resultado real, SHA antes/después, smoke y limpieza; no se afirmará integración por haber emitido este veredicto.

La evidencia independiente permanece en la rama revisora sobre36919b0. El ff incorpora únicamente la historia de implementación hasta8c; no mezcla estos probes ni documentos de revisión con producción. No se eliminan ramas/worktrees históricos.

**STOP tras eventual integración y smoke. No iniciar R03/R04/R05/S31.**
