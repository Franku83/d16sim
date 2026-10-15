# C4 — Hallazgos y cierre independiente

Fecha2026-10-15. Candidato: `8c51b5d9c705745b1a558b00e8bc41d73aad0d7f`. Revisión independiente de la implementación. **No se confirmó un defecto nuevo material ni un residual P2/P3 adicional que requiera aceptación dentro del alcance C4 ensayado.** Esto no demuestra ausencia universal de defectos.

## C3-01 / F15 — VERIFIED_FIXED dentro del dominio probado

- Fuente: `crates/engine/src/run.rs::validate_derived_thermo`, loop de plenum/escape; diff C3→C4 en c4-results/implementation.diff.
- Causa previa: se validaba densidad pero no presión de nodos; restore/no-op aceptaban Inf/0 con operandos finitos.
- Evidencia preservada: checker C3 caso02 ahora pasa. Nuevo `c4-directed -- nodes`: dos nodos×seis estados×dos endpoints=24 filas, más12 corridas factory. Nominal, overflowP, underflowP, overflowρ, extremo finito y pequeño finito.
- Esperado/observado: snapshots con hash válido y cantidades derivadas inválidas se rechazan antes de aceptar/restaurar/reanudar; errores conservan WorkState completo y ledgers. Los estados finitos extremos siguen aceptados en restore/no-op; un avance numéricamente inviable puede devolver Err atómico. run_engine valida su propio estado inicial, no un estado externo inexistente.
- No se añadieron máximos OEM. El criterio es finitud/positividad de la presión/densidad que consume el solver.
- Regresión retenida: tests C3 intactos y nuevo checker C4 casos01/02. No hay riesgo P1 residual demostrado para esta causa en el dominio ensayado.

## C3-02 / F06 — VERIFIED_FIXED dentro del dominio probado

- Fuente: `run.rs::restore`, composición current/prior; `engine/state.rs::MigrationProvenance.prior` opcional.
- Caso original: v1 frío genuino2541266→exact_neutral→avance caliente→pérdida de pmax con c2=.05. Ahora devuelve approximate_hot actual y conserva entero el registro anterior en prior[0]. Las trayectorias completa/incompleta siguen distintas; se declara el límite, no se inventa presión histórica para hacerlas coincidir.
- Prueba nueva: `c4-directed -- provenance`, ocho pérdidas distintas alternadas de pmax y marcador de accounting, con avances entre eventos. Cadena de2 a9 registros contando el origen. En cada transición: hash/esquema actual, anterior completo, orden de hashes y origen histórico preservados. Cada registro previo retiene a su vez su prior; no se exige una lista plana distinta de la estructura implementada/documentada.
- Idempotencia:20 relecturas del mismo envelope+20 recapturas nativas/round-trips por transición, **320 comprobaciones**, sin crecimiento artificial ni promoción a exacta. El origen exact_neutral permanece como origen, no como alcance actual del estado incompleto caliente.
- Compatibilidad: el registro C3 antiguo sin prior coincide con la hoja original C4; esa hoja se deserializa correctamente dentro de las cadenas. Vacío se omite/default, sin cambio de EngineState/state_hash.
- Checkers C3 caso11 y C4 casos03/04 pasan. F07 nativo completo conserva sus pruebas separadas: no se utiliza éxito de una continuación aproximada como replay exacto.
- Limitaciones: metadata no autenticada; writers antiguos pueden descartar prior al reescribir; literales Rust externos requieren inicializar el nuevo miembro. Se prueban8 eventos, no profundidad arbitraria ni metadata maliciosa. Estos límites están declarados en el contrato autorizado y no se convierten en defectos nuevos por falta de universalidad.

## C3-03 / F06 — VERIFIED_FIXED; opción A aceptada independientemente

- Fuente productiva: restore exige igualdad model.schema_version/state.schema_version y accounting conocido para cualquier marcador presente. No hay excepción por tiempo0/hash/fixture.
- Autorización: C4_OPTION_A_AUTHORIZATION.md y, directamente, sección7 de la solicitud humana C4. No se infiere permiso de una afirmación del implementador.
- Source literal36919b0:verification/c3_review/src/main.rs SHA256 `b28adcc0b93950aadf73c950e47b4679f6b90f746dba688ad4cad09c37607982` intacto. Copia derivada SHA256 `c194a4625139f4bb53ef74a8a5cb1b60bb3212bfb1af586d5615eb0603de4e5b`; única línea añadida `v1.model = None;` después de reetiquetar schema1. Diff completo en final-from-git/adapter/fixture.diff.
- Evidencia: estado físico, state_hash, provenance, parámetros, condiciones iniciales y expectativas iguales; hashv1 `deb04a1b413455e20c057ec5cca14c28d691ab52630cef28d172c9224b0dea93` coincide con el productor2541266 genuino. Literal incoherente rechazado; adaptado y genuino sin model aceptados.
- Las dos ejecuciones completas solo difieren en `v1_hash_migration.accepted:false→true`. El resto de observaciones son byte-idénticas. Mismo checker, ningún umbral o assertion alterados.
- **Literal21/32 y caso14 FAIL; adaptado22/32.** El fallo literal no se omite ni se presenta como corregido. La adaptación conserva el significado legítimo de la prueba: lectura de un v1 histórico que nunca llevaba model. No prueba que metadata incoherente sea aceptable.
- Diez combinaciones independientes de esquema/marcador, frías; el checker C3 conserva además los controles calientes. Coherentes/sin marcador histórico se admiten; desconocidos e incoherentes rechazan. Checkers C3 caso10 y C4 casos05/06 pasan.

## C3-04 / F15 — VERIFIED_FIXED dentro del dominio probado

- Fuente: `crates/electrical/src/battery.rs::resistance` comprueba resultado derivado finito/no negativo; `validated` llama también a resistance, cubriendo restore/no-op. Terminal propaga Err antes de limitar voltaje.
- Caso original R0MAX/T10°C rechaza en resistance/validated/terminal/Engine y preserva estado. Control R0MAX/4 válido según combinación. Se conservan−20°C y R=0 ideal.
- Ampliación: **126 combinaciones** de siete temperaturas(incluido borde absoluto), tres SOC y seis juegos de resistencias: cero ideal, subnormal, nominal, grande finita, producto que puede desbordar y suma que desborda. Referencia Decimal independiente de2000 dígitos; considera la combinación completa, no una cota OEM de cada operando. run/restore/no-op, terminal y atomicidad coherentes.
- Casos con resistencia finita muy grande pueden producir terminal0 por el límite físico explícito max(Voc−IR,0). No es el fallo corregido de resistance=Ok(Inf); no se exige nueva exactitud universal de todos los productos eléctricos intermedios.
- Checkers C3 caso06 y C4 caso07 pasan. No queda el residual P2 anterior en estos extremos probados.

## Observaciones y límites abiertos, separados de defectos

1. **KNOWN_LIMITATION/EVIDENCE_GAP:** prior/model/provenance están fuera de state_hash. Cambiar metadata puede conservar hash válido; verified() comprueba estado, no autoría. Ningún consumidor inspeccionado usa prior como prueba criptográfica ni como fuerza/energía física. La procedencia confiable exige fuente externa fiable; no se exige firma nueva.
2. **KNOWN_LIMITATION:** con c2=0 una presión histórica perdida puede no alterar samples ni otros estados dinámicos, pero pmax/hash completos pueden diferir. No se certifica equivalencia universal ni cambio posterior de configuración. Burn_latched/burn_accounted/eventos/fuel siguen persistidos/requeridos.
3. **KNOWN_LIMITATION/EVIDENCE_GAP:** K01–K03,G01–G07,AUD-R01 y los defectos R03/R04 permanecen abiertos. No se declaran calibración, fidelidad OEM, vida útil ni estabilidad general por estos tests.
4. **Evidencia propia corregida, no defecto productivo:** la primera corrida del nuevo checker usó Decimal.prec100 y redondeó la identidad MAX×1 antes de comparar con MAX exacto; reportó falsamente un overflow. Se preservó7/8 y su log. Se corrigió únicamente precisión aritmética a2000, con demostración MAX×1+0=MAX; mismo contrato/sin cambiar tolerancias ni oráculos históricos. Reproducción final8/8. Ver new-checker-reference-correction.json.
5. **Fuera de alcance:** segundo host, fuzzing universal, profundidad arbitraria de metadata, nuevas garantías de autenticación, reconstrucción exacta legacy caliente o funcionalidades R03/R04/R05/S31. La ausencia de estas pruebas no se reclasifica como defecto automático.

No se requiere aceptación de riesgo nueva para los cuatro defectos revisados: están corregidos en el dominio ensayado. La aceptación del adaptador es una decisión explícita de representación del fixture, no una aceptación de metadata incoherente en producción ni un PASS científico universal.
