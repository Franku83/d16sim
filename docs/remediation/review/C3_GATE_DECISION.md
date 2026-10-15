# Decisiones independientes C3

**Fecha: 2026-10-15.**

- **Gate A — DEVELOPMENT_INTEGRATION_BLOCKED.**
- **Gate B — PARTIALLY_VERIFIED global**, con resultados individuales abajo.
- **Fast-forward: NO ejecutado.** La autorización del usuario era condicional; no se satisface. El bloqueo no se debe a los diez rojos R03/R04 ni a exigir ausencia universal de defectos.

## Gate A: criterios de integración de desarrollo

| Criterio autorizado | Resultado |
|---|---|
| Seis correcciones cerradas o legítimamente acotadas | **No**: C2-02/C2-05 parciales por C3-01/C3-02; C2-04 implica además nueva tolerancia pendiente de decisión |
| Sin P0/P1 conocidos dentro de R01–R02 que comprometan integración | **No**: presión de nodos no representable aceptada y nueva pérdida de presión sin alcance actual aproximado |
| Sin regresiones materiales no aceptadas | P1 detectados son residuales; relajación semántica nueva P2 no cuenta con aprobación explícita. No se demostró nueva regresión física P0 |
| Contratos físicos/suites preservados | Sí en dominios probados:209/0,22/32,19/19,15/15,11/11; nuevo7/11 revela cuatro casos separados por severidad |
| Oráculos reproducibles/íntegros | Sí: cuatro checkers y experimentos anteriores byte-idénticos, productores reales reconstruidos desde Git |
| P2/P3 residuales acotados y aceptados previamente si afectan baseline | **No hay aceptación explícita aportada** para C3-03/C3-04. No se infiere de ADR/reportes del implementador ni de esta solicitud de revisión |
| Foundation/gates históricos intactos | Sí; no se editan ni se aprueban retrospectivamente |
| FF técnicamente seguro | Destino ancestro y limpio/source limpio, identidades estables. La posibilidad Git no sustituye el gate técnico fallido |

**Bloqueos concretos P1:**

1. **C3-01/C2-02/F15:** `validate_derived_thermo` omite presión de nodos. Entradas de masa/T/densidad/U finitas y positivas con presiónInf pasan restore y no-op. Es la frontera expresamente autorizada, no una garantía nueva.
2. **C3-02/C2-05/F06/F07:** una procedencia exact_neutral genuina heredada impide registrar nueva pérdida de pmax; continuación con c2≠0 diverge sin advertencia actual approximate_hot. No es manipulación maliciosa de metadata ni exigencia criptográfica nueva.

**No corresponde emitir solo AWAITING_OWNER_RISK_ACCEPTANCE:** siguen los dos P1. Tras resolverlos en una fase de corrección expresamente autorizada, si se retienen los P2 delimitados deberá existir aceptación explícita previa o corregirse. Una aceptación de riesgo P2 no sustituye el cierre de estos P1. No se solicita aquí permiso para un fast-forward que todavía incumple sus otras condiciones.

## Gate B: certificación científica independiente

| Contrato | Veredicto |
|---|---|
| F01 | VERIFIED_WITHIN_TESTED_DOMAIN — química/especies L1,12 corridas y referencias/refinamientos ensayados |
| F02 | VERIFIED_WITHIN_TESTED_DOMAIN — observación física y firma completa válida; AUD-R01 abierto |
| F05 | VERIFIED_WITHIN_TESTED_DOMAIN — JSON/bits/cero firmado/hashes probados; no autenticación de metadata |
| F06 | PARTIALLY_VERIFIED — históricos genuinos/unknown/R02 caliente correctos; procedencia encadenada y coherencia pendientes |
| F15 | PARTIALLY_VERIFIED — numerosas fronteras y rollback probados; presión de nodos y resistencia extrema pendientes |
| F07 | VERIFIED_WITHIN_TESTED_DOMAIN — snapshots nativos completos, mismo modelo/configuración, replay multicorte; no historia incompleta |
| F08/F09/F10 | VERIFIED_WITHIN_TESTED_DOMAIN — especie fresca, líquidos Oil/Coolant y ledger térmico ambiental |
| RV07/RV08/RV09 | VERIFIED_WITHIN_TESTED_DOMAIN — observaciones ECU, dimensión del fitter y factor dyno, sin cerrar R03/R04 |

**Global: PARTIALLY_VERIFIED.** No valida experimentalmente D16Y7, no certifica OEM ni libertad de defectos. Tampoco convierte toda verificación anterior en inválida. Dominios, incertidumbres y cobertura detallados en R01_R02_C3_INDEPENDENT_VERIFICATION.md.

## Integración y estado Git

- **C3 examinado/origen:** `3aef5f6ccdc076dad6e8be627a5eb836b26ee998`.
- **Destino antes:** `8a7e8686e4e8831fb30d1a71b815e67e7d7139ef`.
- **Destino después:** `8a7e8686e4e8831fb30d1a71b815e67e7d7139ef`.
- Historia lineal verificada:8a→C1→C2→8fc7a59747c5b1acfec7695f830a9b1cd8869d47→3aef5f6.
- No se ejecuta `git merge`, ni reset/force/rebase/squash; no se crea merge commit ni se elimina worktree/rama. Master permaneceff09a0d.
- El nuevo commit de evidencia está exclusivamente en la rama revisora sobre40e75d5. Se comunica su SHA final en la entrega; no se incrusta un hash recursivo dentro de su propio contenido.

Estados inicial/final en initial-git-state.json y final-git-state.json. Auditoría25d, sourceC1/caad, C2/a004, C3/3aef y destino8a limpios, sin cambios del revisor. Master conserva el spec no rastreado preexistente, leído por Git para autoridad. Worktrees de prueba detached C1/C2 conservan artefactos históricos no rastreados; el nuevo detached C3 conserva fuentes nuevas de auditoría no rastreadas localmente, versionadas en la rama revisora; producción rastreada intacta. Revisor limpio tras commit de entrega. Ningún archivo productivo no rastreado participa en la reproducción.

No hay smoke posterior a FF al no existir integración; los cuatro CLI y gates209 sobreC3 sirven como smoke del código examinado. Una futura integración requerirá nuevo gate, nueva verificación de source/destino/limpieza y exclusivamente ff-only al SHA aprobado; no queda preautorizada por este informe.

## Próximos pasos autorizables, no ejecutados

- Corregir presión derivada de nodos y composición de límites de procedencia en una nueva revisión de R01–R02.
- Resolver o someter a aceptación expresa la tolerancia de esquemas conocidos incoherentes y el residual P2 de resistencia extrema, sin alterar oráculos ni expectativas históricas.
- Repetir pruebas dirigidas y conservación/replay/suites pertinentes; emitir de nuevo ambos gates separadamente.

AUD-R01,K01–K03,G01–G07,F03/F04/F11–F14/F16/F18/F19 permanecen abiertos. **STOP: no iniciar ni autorizar R03/R04/R05/S31; no merge a master.**
