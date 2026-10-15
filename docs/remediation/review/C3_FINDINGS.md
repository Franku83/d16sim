# Hallazgos independientes C3

Fecha de revisión: **2026-10-15**. Objetivo: `3aef5f6ccdc076dad6e8be627a5eb836b26ee998`. Los números C3-* son hallazgos del revisor; los seis C2-* siguen identificando las correcciones solicitadas. No se implementó ningún fix.

## C3-01 — Falta validar la presión derivada de plenum/escape

**CONFIRMED_DEFECT · P1 · categoría 1: incumplimiento del contrato autorizado C2-02/F15. Residual, no regresión introducida por C3.**

- **Código:** `crates/engine/src/run.rs::validate_derived_thermo`, líneas789–821, especialmente815–819; `NodeW::p`,289–290; `restore`,606–608 y `resume`,1617–1619/1645.
- **Requisito comprometido:** validar T/p/ρ derivadas antes de aceptar estados, incluso restore con hash válido y resume no-op; matriz C3 C2-02 y ADR041§2. No es una nueva exigencia OEM ni fuzzing fuera de alcance.
- **Reproducción ejecutable:** `c3-directed -- thermo`, `derived_node/pressure_overflow`, para plenum y escape. Mismo probe compilado contra C2 en `c2-control-thermo.jsonl`.
- **Condiciones:** m=1e−5kg, T=1e100K, V=1e−214m³, cv nominal. Masa, temperatura, volumen, densidad(m/V=1e209kg/m³) y energía interna(~7.175e97J) son finitos; la presión calculada por la operación del Core m·R·T/V desborda. Inputs y WorkState usan el mismo volumen. Snapshot capturado por API con hash válido.
- **Esperado:** Err tipado antes de devolver un estado con presión no finita; en resume, estado y ledgers intactos.
- **Observado:** restore=Ok y resume(t_end=0)=Ok, `pressure_finite_positive=false`, `unchanged=false` por rebase de ledgers. Avance0.1ms sí rechaza con iv/ev flow y preserva íntegramente el estado. No panic ni prueba de corrupción nominal. Subflujo de presión a0 reproduce el mismo hueco; densidad infinita sí se rechaza, y presión finita extrema se admite en restore/no-op como control positivo.
- **Causa confirmada:** el loop de cilindros valida presión y densidad; el de nodos valida solamente densidad. La comprobación tardía del solver no cubre restore ni avance nulo.
- **Impacto/severidad:** garantía explícita fail-closed de estados físicos/representabilidad incumplida en dos volúmenes. P1 por el contrato de frontera ya autorizado, como C2-02 original; no por afirmar riesgo OEM ni efectos nominales inexistentes. Son extremos numéricos, no resultados experimentales. No se escala la conservación nominal a FAIL.
- **Recomendación:** validar presión derivada de ambos nodos antes de aceptar/restaurar/confirmar; conservar el positivo finito extremo y errores de avance atómicos. No imponer máximo OEM inventado.
- **Regresión propuesta:** pares finito/overflow/underflow, ambos nodos, hash correcto, restore/no-op/avance; conservar comparación de todo WorkState.
- **Evidencia:** `c3-results/final-from-git/thermo.jsonl`, `c2-control-thermo.jsonl`; caso02 del nuevo checker falla. Misma conducta sobre C2 confirma que es residual. **Bloquea Gate A.**

## C3-02 — Procedencia exact_neutral heredada oculta una pérdida posterior de pmax

**CONFIRMED_DEFECT · P1 · categoría 1: incumplimiento C2-05/F06/F07. Residual del cierre incompleto, no nueva exigencia de autenticación.**

- **Código:** `run.rs::restore`,546/583–601, especialmente588–589: la migración nueva solo se registra si provenance era None.
- **Reproducción:** `c3-directed -- replay`, fila `inherited_provenance_loss`. Fixture v1 frío genuino del productor2541266, hash `deb04a1b413455e20c057ec5cca14c28d691ab52630cef28d172c9224b0dea93`; bytes rastreados y comparados con productor recién reconstruido desde Git.
- **Pasos:** restaurar v1 frío con Chen–Flynn c2=.05 → migración legítima exact_neutral; avanzar a0.5s; capturar snapshot nativo completo; retirar únicamente pmax; recapturar con API capture_with manteniendo la procedencia auténtica existente; comparar full y stripped hasta0.55s. No se forja ninguna etiqueta ni original_hash; la metadata heredada no se modifica.
- **Esperado:** rechazo o señal explícita approximate_hot de la nueva pérdida de historia, preservando el origen histórico. La etiqueta original de una migración fría no acredita la continuidad de un snapshot caliente incompleto posterior.
- **Observado:** ambos hashes verifican; restore admite el estado y devuelve la procedencia exact_neutral original sin registrar el nuevo límite. Las trazas divergen; ω final **215.94097001258632 frente218.37292220188047rad/s**. Mismo defecto con el mismo probe sobre C2. La variante sin provenance previa, sí ejercitada por los oráculos anteriores, ahora se etiqueta correctamente approximate_hot.
- **Causa confirmada:** native_exact detecta historia insuficiente, pero `ws.provenance=snap.provenance.clone()` y el guard provenance.is_none impiden actualizar el alcance de continuación. La procedencia histórica puede ser verdadera en origen y, aun así, insuficiente para comunicar la limitación actual.
- **Impacto:** clasificación de continuidad incorrectamente acotada cuando se encadenan migración y posterior pérdida de historia. No se exige recuperar pmax real ni se atribuye a C3 crear el exceso químico R02. No se trata de metadata maliciosamente falsificada o de falta de firma criptográfica.
- **Recomendación/regresión:** no dejar que una procedencia previa impida registrar nueva pérdida relevante de historia; conservar información histórica y señalar el alcance actual. Probar provenance None/exact_neutral/approximate_hot, snapshots completos e incompletos, c2=0 y c2≠0, varias generaciones de captura.
- **Evidencia:** replay y c2-control-replay en `final-from-git`; caso11 rojo. **Bloquea Gate A.**

## C3-03 — Se relaja la consistencia de esquemas conocidos para conservar un fixture

**CONFIRMED_DEFECT / DOCUMENTATION_DRIFT · P2 · categorías 1/2/4: desviación contractual acotada y regresión del rechazo de pares conocidos incoherentes. No se eleva automáticamente a P1.**

- **Código:** `run.rs::restore`,553–585; `engine/state.rs::ModelIdentity.schema_version`,117 («debe coincidir con el estado»); ADR041§4 y snapshot-contract modificados.
- **Reproducción:** `c3-directed -- semantics`, payload schema2/model{schema1,accounting1}; `c2-control-semantics` ejecuta el mismo experimento sobre C2. También se registra el par inverso.
- **Esperado bajo el contrato autorizado:** rechazar identificador incoherente o aplicar una política de compatibilidad expresamente aprobada, sin confundir pertenencia al conjunto{1,2} con coherencia entre dos campos que deben coincidir.
- **Observado:** C2 rechaza model1/payload2; C3 acepta con hash válido y provenance=None. El par inverso se acepta con migración schema1. Los desconocidos999 sí se rechazan correctamente en ambos esquemas. No se demostró divergencia física causada únicamente por cambiar entre esos marcadores conocidos de igual accounting.
- **Causa/política declarada:** el implementador elimina la igualdad para mantener un fixture del oráculo que captura schema2 y después reetiqueta el estado como1. La excepción se generaliza a ambos sentidos; incluso el unit test que antes rechazaba model1/payload2 fue modificado. Los oráculos independientes siguen byte-idénticos: la relajación está en producción y su contrato técnico.
- **Alcance:** consistencia de metadata y significado del contrato, no autenticación ni demostración de torque/masa incorrectos. Por ello P2, con impacto delimitado. Una declaración unilateral en ADR/reporte del implementador no equivale a aceptación explícita previa del responsable.
- **Recomendación:** mantener rechazo de incoherencia o presentar excepción de compatibilidad estrecha, con procedencia y semántica precisas, para revisión/aceptación. No cambiar el oráculo para ocultar el conflicto. Reconciliar la documentación del tipo con la política aceptada.
- **Regresión/evidencia:** matriz v1/v2×ausente/coherente/incoherente/desconocido; caso10 del nuevo checker rojo. Este caso no reemplaza el oráculo C2 ni invalida sus once resultados positivos.
- **Gate:** no es por sí solo un P1; si subsiste afectando el baseline propuesto requiere decisión explícita del responsable conforme Gate A§6. No hay aceptación aportada en esta sesión. No es la única condición pendiente, por C3-01/02.

## C3-04 — La resistencia calculada de batería puede desbordar con parámetros finitos

**CONFIRMED_DEFECT · P2 · categoría 4: residual acotado de representabilidad. No regresión C3 ni nuevo P1 automático.**

- **Código:** `crates/electrical/src/battery.rs::resistance`,121–122; `terminal_voltage`,137, y validación intrínseca.
- **Reproducción:** `battery_extreme`, T=10°C, SOC1, R0=f64::MAX, restantes parámetros ilustrativos. Control R0=MAX/4 y control20°C. Probe idéntico contra C2 y C3.
- **Esperado:** cálculo de resistencia devuelve resultado finito o Err explícito; un parámetro finito no garantiza producto finito.
- **Observado:** validated=Ok, resistance=Ok(Inf), run_engine0.001s=Ok en C2 y C3. En R0=MAX/4 el resultado sí es finito. El terminal se limita a0 y no se observó salida RunTrace no finita ni error que mute parcialmente el estado.
- **Causa:** se validan operandos, no resultado del producto por factor de descarga/frío y suma. El negativo−20°C legítimo ahora funciona correctamente en parámetros ilustrativos; esa corrección concreta C2-06 permanece verificada.
- **Impacto:** API eléctrica en extremos de resistencia, sin evidencia de daño a conservación/trayectoria nominal. La limitación de intensidad/severidad justifica P2; no se presume curva OEM ni se pide calibración. La réplica a10°C sobre C2 demuestra origen previo, evitando atribuirlo a la ampliación legítima a temperaturas negativas C3.
- **Recomendación/regresión:** validar resultado derivado o documentar un dominio representable legítimo; temperaturas y capacidades/resistencias pareadas, con control positivo. No rechazar una temperatura Celsius negativa válida como solución.
- **Evidencia:** battery/c2-control-battery en `final-from-git`; caso06 rojo. Si queda como residual del baseline propuesto, requiere aceptación expresa conforme Gate A§6; no se inventa esa aceptación.

## Límites y riesgos que NO se convierten en nuevos defectos

- **C3-L01, categoría5:** con c2=0, quitar pmax no cambia los samples ni otros campos dinámicos en los cuatro cortes ensayados, pero pmax final difiere en tres; no se certifica identidad universal del estado/hash ni cambio futuro de fricción. Se preservan burn_latched, burn_accounted, prev_local y combustible; omitir esos campos requeridos de JSON provoca error de parseo. La condición c2=0 no autoriza perder esa memoria.
- **C3-L02, categoría5:** model/provenance siguen sin autenticación por state_hash, como declara el contrato. Un hash válido prueba integridad del estado, no autoría del metadata; se requiere fuente externa fiable. No se exige una firma nueva.
- **K01–K03, G01–G07 y AUD-R01:** abiertos sin reinterpretación. Ninguno se cuenta como nueva regresión C3. Una firma diagnóstica normalizada no demuestra identificación multicause ni independencia de instrumentos.
- **Categoría6, fuera de alcance:** fuzzing universal, segundo host, nuevos modelos materiales, OEM/calibración, reparación de F03/F04/F11–F14/F16/F18/F19, R03/R04/R05/S31. Su ausencia no bloquea por sí sola este gate.
- **Sin P0 nuevo demostrado.** Errores dinámicos ensayados preservan estado completo; el problema C3-01 es la aceptación errónea antes del avance, no una demostración de mutación tras Err.
