# R01–R02-C4 — Reporte de implementación

**2026-10-15 · IMPLEMENTED_PENDING_INDEPENDENT_REVIEW.** No se concede gate de integración ni certificación científica. Sin merge/fast-forward ni R03/R04/R05/S31.

## Identidad y autorización

- Base evaluada C3: `3aef5f6ccdc076dad6e8be627a5eb836b26ee998`.
- Revisor original: `36919b0455e8eaf741a72ff4a74ae85f40e116ba`; sus archivos/worktree no se modificaron.
- Rama/worktree de implementación: `remediation/r01-r02-c4`, `/home/franku/projects/d16sim-remediation-c4`.
- Preflight conservado: `ed56f68a95eca81ed20ee2c0b943cf8bba884a5c`, con7/11 y conflicto documentado.
- El responsable autorizó formalmente opción A; [registro](C4_OPTION_A_AUTHORIZATION.md). Esto permite una única copia derivada del fixture con model=None; no modificar oráculos, estado/hash/expectativas/tolerancias ni permitir incoherencia en producción.
- El SHA final del candidato se entrega al usuario y se obtiene con `git rev-parse remediation/r01-r02-c4`. Los tests se ejecutaron desde árbol Git `90d51d9a538034dc8ff9d6243cc4c6b1d13904c1`; la evidencia de identidad de fuentes confirma que los crates y el driver de verificación del candidato final son los mismos. Los documentos/resultados posteriores no cambian el código probado.

## Cambios por defecto

| Hallazgo | Corrección mínima | Antes C3 → después C4 |
|---|---|---|
| **C3-01 P1** | `validate_derived_thermo` comprueba presión de plenum y escape con NodeW::p, además de densidad; finita y >0, sin rangos OEM. | Restore/no-op aceptaban presiónInf/0 con operandos finitos → Err tipado, estado/ledgers intactos. Control de presión finita2.87e297Pa sigue admitido en restore/no-op. |
| **C3-02 P1** | Una nueva limitación compone un registro actual; `prior` conserva completo el anterior. Alcance conservador e idempotencia; recaptura nativa conserva límite. | exact_neutral heredado ocultaba nueva pérdida de pmax → approximate_hot actual con origen antiguo intacto en prior. No se inventa pmax ni se corrige artificialmente la trayectoria divergente. |
| **C3-03 P2** | Coherencia estricta para todo model presente, incluso en frío. Compatibilidad genuina sin model preservada. Adaptador separado según opción A. | Model1/payload2 y model2/payload1 se aceptaban → rechazados. V1 genuino y copia adaptada se aceptan, con estado/hash idénticos. |
| **C3-04 P2** | Resistencia derivada finita/no negativa o BatteryError tipado; SOC válido para la fórmula. La validación de la combinación comprueba R antes de aceptar restore/no-op; el error se propaga al terminal/Engine. | R0MAX a10°C daba Ok(Inf) y podía acabar en voltaje0 silencioso → Err; Engine falla atómicamente. Temperatura−20°C y positivo MAX/4 siguen válidos; dominio ideal R=0 preservado. |

El unit test productivo C3 que afirmaba tolerancia de esquemas conocidos ahora exige rechazo por la política autorizada; no es uno de los oráculos independientes y se fortalece su condición.

Cambios productivos únicamente en `crates/engine/src/run.rs`, `state.rs` y `crates/electrical/src/battery.rs`; se añade un fixture genuino para regresiones. No cambian parámetros nominales, ecuaciones de combustión/flujo, ley de fricción, solver temporal, criterios dyno ni fronteras de evidencia. ADR-042 y snapshot-contract documentan el nuevo comportamiento; ADR-041 permanece histórico.

## Procedencia compuesta: semántica y compatibilidad

Cada nuevo registro describe el esquema/hash del snapshot **entrante actual**, sus campos ausentes, transformación y alcance. `prior: Vec<MigrationProvenance>` opcional contiene el registro anterior completo; este conserva a su vez su historia. Vacío se omite al serializar y ausente se lee vacío. El registro original no se reemplaza ni se pierde su antiguo exact_neutral; se conserva bajo prior al declarar approximate_hot para la pérdida posterior.

El control genuino2541266 produjo en C4:

- Entrada nueva sin pmax: hash `a840fe7fde7d901a816d3680d9523e1f4cc41f9e66d5739f41fbdd8b8b6b329e`, esquema2, continuación approximate_hot.
- prior[0]: esquema1, hash de origen `deb04a1b413455e20c057ec5cca14c28d691ab52630cef28d172c9224b0dea93`, transformación/campos anteriores completos y continuación original exact_neutral.
- Las trayectorias completa e incompleta siguen divergentes: ω215.94097001258632 frente218.37292220188047rad/s. C4 comunica la incertidumbre; no hace coincidir datos incompletos mediante torque o presión inventados.
- Releer el mismo envelope limitado no duplica el evento; recapturar/restaurar JSON no lo eleva a exacto. Una limitación previa approximate_hot tampoco se elimina por defaults neutrales.

`prior` está fuera de state_hash como el resto del wrapper. EngineState, esquema físico2, accounting1, hashes históricos y bits f64 no cambian. Metadata sigue no autenticada: requiere fuente externa fiable para confiar en su origen. **Compatibilidad Rust:** código externo que construya MigrationProvenance por literal debe añadir `prior: Vec::new()`; compatibilidad JSON no implica compatibilidad de todos los literales externos. Los probes históricos entregados compilan sin alterarlos. Lectores antiguos que ignoren `prior` pueden descartarlo al reescribir; la conservación de la cadena aquí demostrada corresponde a la ruta C4, no a todos los escritores históricos.

## Opción A: ejecución literal y derivada separadas

La fuente histórica de fixture y los cuatro checkers están byte-idénticos a Git36919b0. La única modificación autorizada es esta **en una copia derivada**:

```diff
     v1.state.schema_version = 1;
+    v1.model = None;
```

No se modifica el source original. La fuente literal36919b0 ya incorpora los cuatro campos neutros de inicialización R01 auditados anteriormente; C4 no los cambia y conserva también la referencia original25d528e. Se compilan y ejecutan dos bins separados, con outputs y checkers separados. El source derivado completo, original, diff, hashes y origen están en [adapter/](evidence/verification-02/adapter/).

| Artefacto | SHA256 |
|---|---|
| main original,36919b0:verification/c3_review/src/main.rs | b28adcc0b93950aadf73c950e47b4679f6b90f746dba688ad4cad09c37607982 |
| copia derivada autorizada | c194a4625139f4bb53ef74a8a5cb1b60bb3212bfb1af586d5615eb0603de4e5b |

El probe comprueba que original/derivado/genuino conservan hash válido y estado físico idéntico, hash `deb04a1b413455e20c057ec5cca14c28d691ab52630cef28d172c9224b0dea93`. Literal con model2/payload1: **rechazado**. Derivado y genuino sin model: **aceptados**. No hay excepción productiva por tiempo, hash o condición de fixture.

**El caso14 literal NO pasa en C4.** Se registra como falla de su metadata incoherente. El resultado22/32 corresponde exclusivamente a la ejecución adaptada autorizada. El revisor original debe evaluar la legitimidad del adaptador; la autorización del responsable no es aprobación independiente ni convierte la versión literal en verde.

## Resultados ejecutados

| Batería | C3 antes | C4 después | Conteos adicionales |
|---|---|---|---|
| Rust workspace |209/0 reportados y reproducidos por revisor | **215/0** |59 bloques,30 con casos;6 nuevos casos de regresión |
| fmt/clippy exigidos | exit0 | **exit0/0** | warnings no promovidos por el comando exigido |
| **S30 literal** |22/32 | **21/32** |11 fallos,0 errores,89 predicados,138 filas |
| **S30 copia adaptada** |no aplica | **22/32** |10 fallos,0 errores,89 predicados,138 filas |
| Adversarial R01–R02 intacto |19/19 | **19/19** |196 predicados,198 filas/31 familias |
| Controles C1 intactos |15/15 | **15/15** |150 predicados,138 filas/20 familias |
| Extensión C2 intacta |11/11 | **11/11** |1442 predicados,752 filas/15 familias |
| Dirigidos C3 intactos |**7/11**,4 fallos | **11/11** |360 predicados,333 filas/11 familias |
| CLI smoke mecánico |—| **4/4 comandos exit0** |No es smoke posterior a merge: no hubo integración |

Cinco regresiones nuevas fallaron antes de aplicar fixes (4 Engine+1 Electrical). Con las correcciones pasan; un sexto caso adicional asegura que la composición no promueva una limitación anterior. Se entrega también patch de **solo regresiones**, aplicado a C3 archivado, para reproducir los cinco fallos sin fixes; el caso conservador adicional pasa en ese baseline. No se elimina ninguna assertion fallida.

Los diez pendientes originales siguen exactamente17/18(F03/F04),19/20(F11/F12),21/22(F13),23(F14),28(F16),31/32(F18/F19), propios de R03/R04. El undécimo fallo **literal** es14; no se omite ni se reclasifica como PASS. La comparación de las dos ejecuciones completas confirma que solo cambia `v1_hash_migration.accepted:false→true`; old_bytes/hash y todas las otras observaciones son idénticos. No se reescriben logs para conseguirlo.

## Preservación física y numérica

- Ralentí8s/dt50µs: **1047.3038737395177RPM**, Q46146.847889316596J, W_ind13286.519773754073J; residuo6.956782748242368e−5, idéntico al candidato C3.
- F01: química por paso/refinamiento, presupuestos de inventario, combustible insuficiente/sin combustible/reflujo y ventanas de replay permanecen en los oráculos intactos. F02 conserva observación por flujos,70% escape y canales incompletos; AUD-R01 sigue abierto.
- F05 y F07: JSON/bits/cero firmado, hashes y replay nativo completo multicorte conservados. Las limitaciones de historia incompleta siguen explícitas; c2=0 no prueba identidad universal del estado ni permite omitir otra memoria.
- F08/F09/F10 y RV07/08/09: especie fresca, líquidos, ledger térmico ambiental, ECU, dimensiones del calibrador y factor dyno conservados en las baterías intactas.
- Productores genuinos2541266/ff09/8a/caad/a004:25 snapshots reconstruidos desde Git; todos leen/verifican/recapturan. R02 mid-burn sigue approximate_hot y mantiene su discrepancia1157.151388065635J→1157.1513880656353J, sin sustracción artificial.

Estas son comprobaciones de implementación en un host y dominios finitos, no nueva certificación independiente. No se declara calibración, predicción OEM ni ausencia de defectos. Permanecen abiertos K01–K03,G01–G07,AUD-R01 y los IDs de R03/R04. No se amplía la misión a callbacks maliciosos, fuzzing universal, otros hosts o reconstrucción exacta de legacy caliente.

## Estado y entrega

**IMPLEMENTED_PENDING_INDEPENDENT_REVIEW.** La revisión original y sus gates históricos no se modifican. Master y destino R01–R02 siguen en sus SHAs previos. Fuentes, resultados negativos/positivos y autorización quedan en esta rama aislada, limpia tras commit. Ver reproducción y estado Git final para repetir desde Git sin depender de archivos ignorados. No se hizo merge/ff ni se inició otro batch.
