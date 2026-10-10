# ENGINE ENGINEERING SANDBOX — D16Y7
## Investigación fundacional, especificación del núcleo físico y plan de ingeniería reproducible

**Edición:** Research/Foundation v0.2 (charter + research incorporado) · **Fecha:** 2026-10-09  
**Plataforma piloto:** Honda D16Y7 (familia D16, cuatro cilindros, SOHC, 16 válvulas, sin VTEC).  
**Orientación:** desktop-first, núcleo headless multiplataforma; visualización, aprendizaje y evaluación opcional desacoplados. **No se plantea como videojuego de reparación.**  
**Estado:** documento de diseño y programa de investigación; **NO** hay motor físico implementado, calibrado ni validado.  
**Fuente de verdad:** especificación versionada + fixtures trazables + pruebas automáticas, no mensajes de agentes ni cifras improvisadas.

> **Visión:** un laboratorio de ingeniería de motores donde es posible partir de un bloque desnudo, instalar cualquier combinación físicamente permitida, cometer errores reales de ensamblaje, poner el motor en marcha, investigar síntomas con instrumentos, repararlo y modificarlo. La consecuencia debe proceder del modelo físico, no de una misión o `if` diseñado para castigar al jugador.

> **Job-to-be-Done (JTBD) — objetivo principal:** permitir que una persona **comprenda, experimente y diagnostique** fenómenos mecánicos, termodinámicos y electrónicos de motores de combustión mediante un laboratorio virtual **reproducible, transparente, auditable y con dominio de validez explícito**. Su ventaja frente al motor físico es el **control contrafactual**: observar estados internos, congelar/rebobinar, modificar una sola variable, mantener las demás y repetir con el mismo estado inicial y semilla. El propósito secundario es mejorar progresivamente la correlación cuantitativa con motores reales; **no** sustituir mediciones, decisiones de tuning ni diagnósticos sobre un motor físico concreto.


> **Límite epistemológico:** «modelar todos los factores» literalmente requeriría resolver sólidos, fluidos, combustión química, contactos, electrónica, microgeometría y desgaste multiescala con datos que no están disponibles. **No es una promesa técnicamente honesta.** Sí podemos **inventariar todos los fenómenos relevantes, asignarles una representación explícita, declarar aproximaciones y ampliar su fidelidad sin alterar los contratos del sistema**. No confundiremos cobertura del estado con exactitud predictiva.

---

# 0. Cómo usar este documento

Este archivo es un **documento fundacional vivo**, no una lista de tickets lista para ejecutar sin investigación. No compactar fases. Prohibido declarar que una fase terminó solo porque existe un tipo de dato, un slider, una animación o una prueba contra valores fabricados.

**Convenciones de evidencia**:

- **[F] Fuente verificada:** referencia técnica o manual localizado; aún hay que comprobar edición, versión, aplicación y alcance.
- **[D] Deducción de ingeniería:** decisión o consecuencia razonada; hay que probarla.
- **[H] Hipótesis/modelo simplificado:** aproximación sujeta a calibración o sustitución.
- **[P] Pendiente:** no se obtuvo documentación suficiente; no fijar valores numéricos.
- **[X] No soportado:** fenómeno reconocido pero aún no modelado o diagnosticable.

Para **cada parámetro**, mantener: `id`, descripción física, unidad SI, dominio, valor o distribución, fuente/edición/página/URL, variante D16Y7, método de extracción, incertidumbre, versión del dataset, estado `[F/D/H/P]`, responsable y fecha de revisión. Un valor copiado de internet sin variante ni método **no es un dato de producción**.

**Orden de lectura:** decisiones (1), física y realismo (2–7), arquitectura/reproducibilidad (8–12), validación (13), sprints (14–16), datos/experimentos y fuentes (17–20).

## 0.1 Charter de producto científico — para S00 (no es una feature adicional)

**Tres capacidades que gobiernan las prioridades:**

1. **Comprender:** inspeccionar ecuaciones, estados internos, hipótesis y balances de masa/energía; una representación L1 explicable y honestamente etiquetada puede ser útil aunque no prediga todos los detalles del D16Y7 real.
2. **Experimentar:** ejecutar intervenciones controladas y contrafactuales (A/B, snapshot, replay, misma seed, una variable a la vez). Las demás variables deben quedar fijadas o su acoplamiento debe declararse explícitamente. No basta un slider que fuerce la salida.
3. **Diagnosticar:** formular hipótesis, seleccionar instrumentos, medir y discriminar causas distintas con síntomas semejantes, **solamente** dentro de los fallos cubiertos por el modelo. Sin ayudas automáticas durante exploración libre; un evaluador **externo y opcional** puede contrastar diagnósticos contra la verdad conocida de un *fixture* al finalizar un escenario, sin alterar el estado del mundo simulado ni imponerlo como requisito del core.

**Condición de utilidad por subsistema:** un alcance concreto puede considerarse útil cuando sus relaciones son observables y explicables, sus experimentos se reproducen, sus mediciones distinguen hipótesis pertinentes y el sistema comunica explícitamente qué está verificado, extrapolado, pendiente o no soportado. No exige completitud física total ni predicción exacta de motores reales; tampoco permite declarar validez física por tener solo tests de software. Mantener separados **verificación**, **validación** y **calibración**.

**Límites de alcance:** no convertirse en videojuego de misiones, en calibrador certificado, en herramienta de reparación sin verificación externa ni en un modelo que inventa un único diagnóstico para cada síntoma. El usuario puede provocar fallos mediante estados físicos de ensamblaje; los resultados tienen que emerger de los modelos y sus interacciones. Este JTBD es criterio para arbitrar decisiones, **no añade sprints ni obliga a implementar un módulo educativo o evaluador en S00**.

**Fuentes de investigación complementarias para el repositorio:** `D16Y7_RESEARCH_v0.1.md` (datos propios Y7, literatura y casos) y `D16Y7_RESEARCH_DONANTES_v0.1.md` (donantes, transferencias y límites). Conservar ambos **íntegros e independientes**; la adenda §24 sirve de índice operativo y no los sustituye. Ajustar enlaces relativos a la ubicación efectiva de los archivos al incorporarlos al repositorio.

---

# 1. Decisiones fundamentales (ADR inicial)

| ID | Decisión propuesta | Motivo | Revisión / alternativa |
|---|---|---|---|
| ADR-001 | **D16Y7 OEM no-VTEC como plataforma única inicial** | Menor complejidad de distribución; amplia documentación Honda | Fijar **modelo/año/mercado/transmisión/ECU** en S01; no mezclar 1996 y 2000 |
| ADR-002 | **Core headless Rust** (C++ alternativa si un solver lo justifica) | Control de tipos, memoria, pruebas y compilación nativa | Benchmark antes de comprometer WASM |
| ADR-003 | **Desktop-first, interfaz web opcional** | No limitar bibliotecas numéricas ni análisis pesado | Tauri/Three.js solo en fase UI; migración posible |
| ADR-004 | **Ángulo de cigüeñal como coordenada primaria del ciclo** + tiempo físico para transitorios | Sincroniza volumen, distribución, chispa y cilindros | Adaptación near-zero RPM y motor detenido |
| ADR-005 | **Estado físico canónico, instrumentos como observadores** | Evita que el juego “sepa” y regale la avería | Sensores tienen ruido, sesgo, fallos y ancho de banda |
| ADR-006 | **Propiedades/mecanismos generales antes de averías concretas** | Una junta puede fugar sin un script “bad_gasket” | Se permiten modelos reducidos identificados |
| ADR-007 | **Unidades SI internamente y tipos dimensionales** | Previene errores silenciosos (psi/Pa, °F/K, N·m/lb·ft) | Mostrar unidades seleccionables solo en UI |
| ADR-008 | **Cada feature física entra con V&V y procedencia** | Asegurar que el motor sea consistente y reproducible | ASME VVUQ y NASA-STD-7009B [R09, R10] |
| ADR-009 | **No CFD/FEA 3D completo dentro del loop interactivo** | Costo inmenso y falta de parámetros/materiales | Solvers externos/offline para mapas sustitutos verificados |
| ADR-010 | **Sin pop-ups explicando errores de montaje** | Diagnóstico emergente, experiencia tipo “Dark Souls” | Excepciones: accesibilidad, elección explícita de modo tutorial y errores del propio simulador |
| ADR-011 | **No copiar firmware de Speeduino ni activos Honda sin revisar licencias** | Speeduino GPL-2.0; manual Honda protegido; modelos CAD con derechos | Implementación propia fundamentada en conceptos/ecuaciones y contenido autorizado |
| ADR-012 | **No monetizar/etiquetar resultados como ingeniería certificada** | El primer producto no es software de calibración/safety-critical | Publicar dominio de validez por modelo |

**Definición del primer gran hito (“BASE-REPRODUCIBLE”):** un D16Y7 stock definido por datos trazables, estado persistente del motor, herramientas programáticas de ensamblaje, ciclo termodinámico de 4 cilindros con gas exchange simplificado, arranque/ralentí controlado, sensores/ECU de referencia, al menos fallas por causalidad documentada, medidas virtuales, replay determinista bajo entorno fijo, y batería V&V. **No exige el 3D.** Sin datos medidos, puede ser *base computacional validada solo parcialmente*, y deberá llamarse así.

---

# 2. Investigación de viabilidad: hallazgos y límites

## 2.1 Documentación del D16Y7 [F]

El manual de servicio Honda Civic 1996–2000 identifica el D16Y7 como 1.6 SOHC 16-valve con inyección multipunto [R01]. En las secciones de PGM-FI aparecen ECM/PCM, MAP, IAT, CKP/TDC/CYP y variaciones por año [R01, R02]. El manual documenta inspección de cigüeñal, camshaft, tensado de correa, culata, tolerancias y diagnóstico [R01]. **Esto no equivale a tener todas las curvas físicas**: el manual de taller suele ofrecer cotas y límites de servicio, no mapas completos de flujo, coeficientes térmicos, rigideces de juntas, dinámica de anillos o presiones indicadas por ángulo.

**Configuración propuesta a congelar (pendiente de verificación documental definitiva):** 1996–1998, mercado estadounidense, Honda Civic D16Y7 con transmisión manual. Elige **una** configuración de motor/ECU y números de pieza reales en S01; «D16Y7» por sí solo no determina todas las señales o estrategias de software.

| Dato piloto | Valor de referencia preliminar | Estado |
|---|---:|---|
| Arquitectura | 4 cilindros en línea, 4 tiempos, SOHC, 16 válvulas, sin VTEC | [F] estructura, edición concreta por fijar |
| Diámetro nominal | 75 mm | [P] cotejar con tabla OEM y variante |
| Carrera nominal | 90 mm | [P] cotejar con tabla OEM y variante |
| Cilindrada geométrica calculada | ~1590.43 cm³ (`4 × π/4 × 75² × 90 mm³`) | [D] solo válida si diámetro/carrera confirmados |
| Compresión geométrica de referencia | aprox. 9.4:1 | [P] confirmar para variante |
| Potencia de catálogo | aprox. 106 hp | [P] confirmar norma, año, mercado y corrección; **no usar como único objetivo de calibración** |
| Orden de encendido | 1-3-4-2 | [P] confirmar contra manual aplicable |
| CKP/TDC/CYP | documentados por Honda; conexiones/patrones varían | [F] existencia; [P] forma de onda, dientes, polaridad, estrategia exacta |
| CKF | distinguir años/configuraciones | [P] no inferir presente en todas |
| Compresor turbo | **ninguno de fábrica** | [F] configuración atmosférica de partida |

No introducir una propiedad ausente “de oficio”. Ejemplo: **no colocar sensor de detonación operativo si el D16Y7 variante OEM seleccionado no lo trae**. El inventario debe reflejar hardware real, no una ECU moderna ficticia.

## 2.2 Qué proyectos ayudan [F]

| Proyecto | Qué aporta | Qué NO resuelve / precaución |
|---|---|---|
| [Engine-Sim de AngeTheGreat][R03] | Dinámica aproximada, audio, pipeline de motor | El autor advierte explícitamente que **no** es un instrumento científico; licencia MIT [R03a] |
| [Speeduino][R04] | Ejemplos reales de decodificación de disparos, inyección, mapas, filtros, scheduler | Firmware **GPL-2.0**: estudiarlo como referencia, no copiar por accidente al núcleo propietario [R04a] |
| [Ardu-Stim][R05] | Generación de patrones de trigger para pruebas de ECU | No representa automáticamente el patrón OEM del año elegido |
| [Cantera][R06] | Termodinámica, especies, cinética química y reactores; solver de referencia | Ejemplo ICE publicado es **diésel** e ilustrativo; no calibra un D16 gasolina; BSD-3-Clause [R06a] |
| [OpenWAM (CMT-UPV)][R07] | Solvers 1D de gases en admisión/escape de motores | Código histórico (release oficial identificado 2016), dependencia/licencia por **auditar**; no confundir con un proyecto de IA homónimo |
| [Haltech Trigger System][R08] | Explica señales trigger/home, 720° y sincronización de ECU | Referencia de ECU aftermarket, **no** especificación Honda PGM-FI OEM |
| [Tauri][R13] | Contenedor desktop con backend Rust y UI web | **No** es el motor de física; solo shell/interfaz |
| [Garrett compressor maps][R15] | Presión absoluta/relativa, eficiencia, flujo, surge, choke | Para etapa turbo, no primera base stock |

**Nota de búsqueda crucial:** `github.com/OpenWAM/OpenWAM` y `OpenWAM-Official/OpenWAM` pueden referirse a **world-action models de IA**, no a simulación 1D de motores. El proyecto de motores encontrado es `github.com/CMT-UPV/OpenWAM` [R07]. La colisión de nombres invalida suposiciones sobre licencias.

## 2.3 Qué conocimiento falta [P]

1. Especificación exacta de ECU Honda: patrón CKP/TDC/CYP (dientes, umbral, fase), wiring, mapas OEM y estrategias fall-back.
2. Levantamiento del motor: cámaras de combustión, deck height, longitud de biela, geometría de pistones/aros y holguras por condición.
3. Distribución real: eventos apertura/cierre y alzadas de válvulas en función de ángulo, no solo “camshaft OEM”.
4. Flujo de culata por alzada y caída de presión, pérdidas de puertos, plenum y escape.
5. Datos experimentales de presión en cilindro, torque, consumo, temperaturas y emisiones en distintas condiciones.
6. Curvas de bomba de aceite/agua, viscosidad del aceite vs temperatura, cojinetes y refrigeración.
7. Elasticidad de tornillos/culata/bloque, coeficientes de fricción de rosca, comportamiento de junta, límites térmicos y fatiga.
8. Mapas de inyección/atomización, combustible y bobina/igniter especificados.
9. Distribuciones de tolerancias y dispersión entre unidades reales.

**Resultado de la investigación:** hay suficiente soporte para crear una **arquitectura rigurosa y un solver reducido**. **No hay suficientes datos recopilados todavía para prometer predicciones fiables de horsepower a cualquier modificación, detonación exacta o instante de rotura de una biela.** Cada brecha queda en el registro de incertidumbres.

---

# 3. Política de realismo: cobertura, resolución y credibilidad

## 3.1 Matriz obligatoria por fenómeno

Cada fenómeno registra por separado:

1. **Representación del estado:** ¿existe una variable/objeto para ese efecto?
2. **Mecanismo:** ¿qué ecuación transforma inputs en outputs?
3. **Acoplamiento:** ¿interactúa bidireccionalmente con el resto?
4. **Datos:** ¿qué parámetros y mediciones respaldan la ecuación?
5. **Fidelidad:** orden de modelo (0D/1D/FE/reducido), alcance y simplificaciones.
6. **Observabilidad:** ¿cómo podría medirlo el jugador con instrumentos?
7. **Verificación:** ¿el código resuelve bien las ecuaciones previstas?
8. **Validación:** ¿predice un motor real dentro de incertidumbre reportada?

**Niveles (por subsistema, nunca etiqueta global del juego):**

- **L0 — Semántico:** pieza/sensor declarado, sin modelo físico. No influye en el resultado.
- **L1 — Conservativo simple:** ecuaciones/relaciones físicas básicas; puede verificarse, no necesariamente validarse.
- **L2 — Modelo reducido calibrado:** responde a régimen, condiciones y dimensiones con mediciones en rango conocido.
- **L3 — Modelo especializado:** mayor resolución y/o acoplamientos validados dentro de un dominio más amplio.
- **L4 — Offline de referencia:** modelo avanzado 1D/3D/FE, con tiempo de cómputo variable. No implica automáticamente mejor predicción sin buenos parámetros.

**Etiquetas de credibilidad**: `UNVERIFIED`, `VERIFIED_NUMERICALLY`, `CALIBRATED`, `VALIDATED_IN_DOMAIN`, `EXTRAPOLATED`, `UNSUPPORTED`. Una curva bonita con `VERIFIED_NUMERICALLY` no equivale a medición real.

## 3.2 Tres modalidades de ejecución

- **Headless deterministic scenario**: sin UI, comandos reproducibles para CI y estudios de sensibilidad. Plataforma y toolchain fijados; checksums.
- **Interactive realtime**: time step/eventos adaptativos y modelos reducidos; decisiones de fidelidad documentadas.
- **Offline analysis**: escenarios con alta resolución, barridos y comparación contra datos externos; mapas surrogate versionados para tiempo real.

**Nunca:** asumir que ejecutar 60 FPS = fidelidad; permitir que el render modifique la física; generar síntomas directamente desde un evento `fault`; rellenar desconocidos con constantes “de conveniencia” silenciosas; sustituir un fallo físico por una penalización de HP arbitraria.

## 3.3 Objetivos no funcionales iniciales [D]

- Reproducir **la misma traza numérica en la misma plataforma, toolchain, flags, seed y versión del modelo**. Equivalencia bit a bit entre CPU/OS/WASM **no garantizada**: comparar tolerancias y eventos cuando corresponda.
- Separación absoluta entre reloj físico, reloj de eventos, render y ritmo del usuario.
- Ecuaciones y derivaciones documentadas con dimensionalidad y dominios.
- Errores numéricos detectados: masa/energía negativa, NaN, presiones imposibles, penetraciones no justificadas, pérdida de sincronización de scheduler, pasos rechazados.
- Un estado inválido produce `SimulationError` trazable, **no una avería ficticia**.
- Toda transición/desmontaje/reparación es un **comando** reversible por replay completo (no exigir deshacer físicamente una rotura).
- Los modos de dificultad pueden cambiar información/tutoriales/ayudas, nunca las leyes físicas para el mismo dataset y configuración.

---

# 4. Inventario de subsistemas: qué se debe modelar (cobertura total del mapa, no fidelidad total desde día 1)

Esta tabla es el contrato de cobertura: cada entrada debe permanecer visible en la arquitectura aunque empiece L0. La columna de fase es sugerencia, **no** autorización para incluir un efecto no validado.

| Subsistema | Estado físico mínimo | Entradas y acoplamiento | Salidas, instrumentos y fenómenos | Evolución |
|---|---|---|---|---|
| Block/crankcase | bore, superficie, deck, masa, T, grietas | presión en cilindros, aceite, tornillos, refrigerante | expansión, límites geométricos, fugas, inspección | L0→L2 |
| Cigüeñal | fase 720°, velocidad, inercia, descentramiento, torsión | torque gas, fricción, carga | RPM, vibración, potencia al freno | L1→L3 |
| Bielas | longitud, masa, big/small end clearance, fatiga | fuerzas de gas/inercia, lubricación | esfuerzos, interferencias, desgaste | L0→L2 |
| Pistón | masa, corona, pines, skirt, offset, T | gas, pared, anillos | fuerza, fricción, expansión, contacto | L1→L3 |
| Anillos | gap, tension, seating, desgaste, leakage | presión, temperatura, film aceite, honing | blow-by, consumo aceite, compresión | L0→L2 |
| Cilindro | diameter vs z/θ, ovalidad, taper, rugosidad, rayas | desgaste, lubricación, gases | sellado, fricción, mecanizado | L0→L3 |
| Cojinetes | clearance, oil film, T, estado de superficie | velocidad, carga, viscosidad/caudal | pérdida fricción, daño por hambre de aceite | L0→L2 |
| Culata | planitud, deformación, T, puertos, cámara | fasteners, gases, cooling | sellado, compresión, flow | L0→L3 |
| Junta culata | material, espesor, mapa de resistencia de sellado | compresión local, presión, T, ciclos | fuga cámara/aceite/agua, no booleano | L1→L3 |
| Pernos/espárragos | posición, torque-angle, precarga, yield, estado rosca | fricción rosca, secuencia y superficie | clamp field, distorsión, fallas | L1→L3 |
| Camshaft/rockers | perfiles alzada, lóbulos, fricción, lash | correa, rpm, aceite | lift, timing, valve float, desgaste | L1→L3 |
| Timing belt/tensioner | dientes, fase, tensión, desgaste | cam/crank torque y temperatura | mis-timing, saltos, rotura | L1→L3 |
| Válvulas/guías/asientos | masa, lift, lash, sellado, temperaturas | cam, resortes, gases, lubricación | airflow, compresión, contacto pistón | L1→L3 |
| Colector admisión | V, p, T, pérdidas/ondas | throttle, filtro, cilindros, ambiente | MAP, VE, response | L1→L3 |
| Escape | p, T, backpressure, gases residuales | blowdown, restricciones | pumping losses, calor, sonido | L1→L3 |
| Combustible | presión rail, densidad, temperatura, composición | bomba, regulador, inyectores | masa inyectada y fallas de suministro | L1→L3 |
| Air-fuel/combustión | mezcla, residuales, fracción quemada, p, T | inyección, chispa, flujo, refrigeración | IMEP, torque, p–V, EGT estimada | L1→L3 |
| Encendido | energía bobina, dwell, bujía, fase | ECU, batería, CKP/TDC/CYP | chispa, fallos, avance efectivo | L1→L3 |
| ECU/PGM-FI | estado sincronización, mapas, sensores, scheduler | señales, ambiente, feedback | pulsos inyector, dwell, timing, códigos | L1→L3 |
| Sensores | valor real, física de transductor, cable, ruido | magnitud física, alimentación, temp | señal virtual medible, DTC indirecto | L1→L3 |
| Cableado/eléctrico | resistencias, conexiones, masa, tensiones | batería, alternador, consumo | caídas, open/short/intermitencias | L1→L3 |
| Cranking/starter | torque vs RPM, batería, tensión | inercia, compresión, temperatura | waveform RPM de arranque | L1→L2 |
| Sistema de aceite | viscosidad(T), nivel, bomba, galerías, presión | RPM, cojinetes, T, fugas | oil pressure, lubricación, fallas | L1→L3 |
| Refrigeración | capacidad T, bomba, termostato, radiator | calor cilindros, ambiente, RPM | ECT, sobrecalentamiento | L1→L3 |
| PCV/cárter | volumen/presión/ventilación | blow-by, pistones, aceite | presión cárter, aceite arrastrado | L1→L2 |
| Soportes/dyno | inercia, carga, torque resistente | brake/load law | torque freno, transitorios y power | L1→L3 |
| Turbo posterior | turbina/compresor mapas, rotor, intercooler | exhaust enthalpy, MAP, fueling | boost transitorio y límites | L0 hasta expansión |
| Instrumentación | ruido, resolución, calibración, sample rate | puertos medibles y sondas | multímetro/scope/compression/leak-down | L1→L3 |
| Herramientas/operaciones | ajustes/torque/mediciones, historial | usuario, estado ensamble | piezas instaladas/desmontadas, errores reales | L1→L3 |

**Fundamental:** no hay «sistema de daños» que reparta penalizaciones. Hay geometría, materiales, estados termodinámicos, conexiones y leyes de degradación. Un *fault catalog* existe **para pruebas y generación de escenarios**, no como motor de consecuencias.

---

# 5. Diseño físico del ensamblaje: desde el bloque desnudo

## 5.1 Topología de objetos y restricciones

Las entidades se separan en cuatro categorías para evitar que “pieza” signifique cien cosas:

1. **PartDefinition**: diseño nominal identificable (ej. tornillo de culata OEM), material, dimensiones, puertos, contactos, requisitos, tolerancias y versión de datos. **Inmutable**.
2. **PartInstance**: pieza concreta con serial, dimensiones realizadas, historial de uso, desgaste/daño, suciedad, temperatura. **Mutable y persistente**.
3. **Interface/Joint**: unión entre instancias; geometría de montaje, orientación, contactos, sellos, precarga, conexiones eléctricas, pasos de fluidos.
4. **Process/Operation**: acción del usuario: limpiar, medir, lubricar, instalar, torquear, girar, presurizar, mecanizar, reemplazar, probar.

**No codificar** `if pistonInstalled then canStart`; motor de arranque puede aplicar torque incluso con piezas ausentes si hay camino mecánico para transmitirlo. **Sí impedir** operaciones geométricamente imposibles (volúmenes sólidos incompatibles sin deformación/corte), y contabilizar contacto/destrucción cuando se aplica fuerza capaz de causar daño.

### Modelo mínimo de entidad (esquema conceptual; no API estable)

```yaml
part_definition:
  id: honda.d16y7.head_bolt.oem.ref_001
  geometry_ref: geometry/head_bolt_v001
  material_ref: material/bolt_steel_unknown
  mounting_interfaces: [head_bolt_bore_1, block_thread_1]
  constraints: [thread_pitch_match, engagement_length, bearing_surface]
  data_confidence: UNVERIFIED
  source_refs: []
part_instance:
  uuid: bolt-0001
  definition_id: honda.d16y7.head_bolt.oem.ref_001
  as_built: {thread_diameter_m: null, free_length_m: null}
  condition: {permanent_strain: 0.0, thread_damage: 0.0, cycles: 0}
assembly_joint:
  id: cylinder_head_fastener_1
  connected_instances: [bolt-0001, head-001, block-001]
  preload_newton: null
  applied_torque_newton_metre: null
  torque_angle_radian: null
  install_history: []
```

**`null` no equivale a cero**; significa desconocido y obliga a escoger modo de aproximación documentado o detener el cálculo relevante (fail-closed). Los seriales deben persistir en savefiles: dos juegos de aros idénticos nuevos pueden acabar distintos por desgaste/temperatura/historia.

## 5.2 Dependencias reales

- Cigüeñal ↔ cojinetes ↔ bancadas ↔ block ↔ aceite: cargas, holguras, lubricación.
- Pistones ↔ anillos ↔ cilindros: compatibilidad diámetro, end gap, sellado, fricción, calor.
- Bielas ↔ bulones ↔ cigüeñal: carrera, geometría, orientación, interferencia, torque de tapa.
- Culata ↔ deck ↔ junta ↔ tornillos: ruta de carga + contactos + sellos de presión/aceite/refrigerante.
- Cam ↔ correa ↔ crank: relación 2:1, offset; perfiles de válvulas/posibles contactos.
- Distribuidor/sensor ↔ señal electrónica ↔ ECU ↔ bobina/inyectores: eventos de encendido/combustible.
- Mangueras/galerías ↔ puertos: flujos solo a través de vías conectadas; desconexión puede crear fuga.

## 5.3 Torque y precarga: ejemplo central de diseño [F/D/H]

**El par aplicado no es directamente fuerza de apriete.** En una aproximación inicial: `T = K · F_preload · d` (torque `T`, coeficiente agregado `K`, precarga `F`, diámetro nominal `d`). El coeficiente cambia con lubricación, condición de roscas, acabado y superficies. ARP publica pares asociados a cargas de apriete bajo lubricación concreta [R11]; Fel-Pro explica secuencias y problemas de sellado por carga desigual [R12]. **No trasladar valores ARP a tornillos Honda OEM.**

El mecanismo implementable, por etapas:

1. El usuario selecciona herramienta, lubricante/condición y ejecuta giro/apriete (historial de torque y ángulo).
2. Un **modelo de fastener** estima precarga y deformación elástica/plástica con incertidumbre.
3. El ensamblaje de head/gasket/block redistribuye cargas a contactos/segmentos (modelo **reducido de mapa de rigidez** calibrable offline; no asumir FEA completa).
4. La junta tiene regiones de estanqueidad (anillos de fuego, aceite, refrigeración), temperatura, daño y una ley de fuga dependiente de presión y compresión local.
5. Fugas resultantes se convierten en **flujos de masa y entalpía** entre dominios (cilindro, cárter, agua, atmósfera), y alteran p/T/combustión/enfriamiento.
6. Solo entonces las observaciones muestran compresión baja, burbujas, temperatura, consumo o gases, si el fenómeno físico realmente lo implica.

**Junta omitida:** puede cambiar altura relativa, compresión geométrica, geometría de distribución/holguras y sellado; **no** asignar “0% compression” o “-80 hp” por booleano. Depende de contacto real y del modelo admitido. Si geometría/dureza insuficientemente conocida, etiquetar `EXTRAPOLATED`.

**Secuencia incorrecta:** conservar el *historial de apriete*; el mismo par final puede producir diferente deformación residual/precarga en un modelo con elasticidad y asiento dependiente de la trayectoria. Sin ese modelo, **no fingir** que la secuencia importa porque simplemente hemos impuesto un multiplicador oculto.

## 5.4 Orden de instalación y errores posibles

- Cada operación tiene **precondiciones físicas**, no reglas “misión completada”.
- Se puede intentar cranking con elementos ausentes; falta de acoplamiento devuelve ausencia de transmisión de torque, no una advertencia de gameplay.
- Se puede montar distribución desplazada; el perfil de válvula y geometría pistón/válvula deciden contacto.
- Si un perno carece de apriete, el joint conserva precarga cercana a cero; **no siempre** falla inmediatamente.
- Herramientas aplican magnitudes, no cambian directamente el daño: torque, desplazamiento, temperatura, presión de prueba, corriente eléctrica.
- Errores numéricos/gaps de datos deben reportarse a log técnico, **sin esconderse como síntomas mecánicos**.

## 5.5 Trazabilidad de montaje

Cada `AssemblyOperation` registra `scenario_time`, `actor`, `tool_id`, `target`, parámetros, unidades, resultado, estado previo/posterior hash y provenance. Debe permitir preguntas: “¿qué tornillo recibió qué ángulo?”; “¿se utilizó la misma biela?”; “¿qué presión mostró el leak-down antes de desmontar?”. El juego no revela por defecto el diagnóstico, pero el historial de acciones propias puede consultarse como libreta del taller.

---

# 6. Modelo físico: del mecanismo al ciclo completo (física de referencia)

**Regla de implementación:** estas expresiones son **puntos de partida**, no una calibración del D16Y7. Registrar libro de hipótesis `model_assumptions.md` y unidades de todas las variables. Una ecuación de termodinámica ideal no predice fielmente un motor con fugas, válvulas abiertas y combustión real [R16].

## 6.1 Coordenadas, sincronización y cinemática de 4 tiempos

Definir un **ángulo global continuo** `theta_crank` sin truncarlo a 360°, una velocidad angular `omega [rad/s]`, `rpm = 60 omega/(2π)` y un **ángulo de ciclo** `theta720 = wrap(theta_crank, 4π)`. La fase de cada cilindro deriva de topología de cigüeñal + orden de encendido + offset físico de su PMS en compresión. No configurar cuatro cilindros con ciclos independientes que ignoren el mismo eje.

Para un mecanismo biela-manivela ideal sin offset del bulón:

- `r = stroke/2` (radio de manivela), `l` longitud entre centros de biela, `A = π B²/4`, `B` diámetro.
- Desplazamiento desde PMS: `x(θ) = r(1−cos θ) + l − sqrt(l² − r² sin² θ)`.
- Volumen: `V(θ) = V_clearance + A x(θ)` (ajustar con geometría real de corona, deck, gasket y cámara).
- `dV/dθ` debe derivarse analíticamente o con diferenciación automática controlada; evitar diferencias finitas ruidosas en la fuerza.
- Velocidad de pistón `dx/dt = (dx/dθ) ω` y aceleración incluyen `dω/dt`, no solo ω constante.
- Relación geométrica `CR = (Vd+Vc)/Vc`; recalcular al sustituir pistón, junta, culata o cilindrada.

Para un 4 cilindros típico, eventos de combustión repartidos en 720° cada 180° **solo si** el orden y la configuración se validan. Posición, etapa (admisión, compresión, expansión, escape) y apertura de válvulas son **estados distintos**; dos cilindros pueden estar en PMS simultáneamente pero en carreras diferentes.

### Invariantes mecánicas

- Una vuelta del árbol de levas por cada **dos** del cigüeñal en el D16 4T.
- La cinemática de las piezas unidas debe mantener holonomía/control de error de restricción.
- Volumen de cilindro positivo para cualquier paso admitido.
- El orden de ignición cambia la fase de eventos, **no** reconfigura mágicamente posiciones físicas de pistones.
- El contacto pistón-válvula se decide por envolventes geométricas y holguras térmicas/dinámicas; «interference engine» no basta para decretar choque en todos los errores de timing.

## 6.2 Ecuación dinámica y arranque

Eje de cigüeñal reducido:

`J_total(θ) dω/dt = Σ_i τ_gas,i(θ) − τ_friction(ω,T,load) − τ_pumping − τ_accessory − τ_dyno + τ_starter − τ_external_losses`.

**Importante:** las convenciones de torque deben evitar restar dos veces bombeo o fricción. Si `τ_gas` se deriva integrando el trabajo completo de admisión+compresión+combustión+escape, ya incorpora *pumping work*; si solo cubre circuito cerrado, el *pumping loss* se añade explícitamente. Definir un solo esquema para ambos cálculos.

`τ_gas,i = p_i · dV_i/dθ` para trabajo indicado instantáneo **con presión de referencia coherente** y según cómo se contabilice contrapresión del cárter. Para un ciclo completo: `W_i = ∮ p_i dV_i`; `IMEP = W_i / Vd_i`. Luego aplicar pérdidas de fricción/accesorios para `BMEP` y potencia al freno. `P_brake = τ_brake ω`; especificar ambiente, tiempo de estabilización y correcciones conforme a metodología compatible con SAE J1349 [R17].

El modelo de arranque incluye: batería estado de carga y temperatura → tensión bajo carga → curva de torque del starter → fricción estática/viscosa + compresión → aceleración de cigüeñal → CKP → ECU → chispa/inyección → combustión. No inicializar el motor a 800 RPM como sustituto del arranque salvo test unitario específicamente etiquetado `seeded_idle`.

## 6.3 Termodinámica por cilindro (0D, modelo progresivo)

En un cilindro cerrado, simplificando gas ideal y calor específico:

`dU/dt = Qdot_chem − pdV/dt − Qdot_wall`, con `U ≈ m u(T,composition)`, `pV = m R_mix T`.

Cuando hay admisión/escape/blow-by, aplicar **sistema abierto**:

`dU/dt = Qdot_chem − pdV/dt − Qdot_wall + Σ_in mdot·h_in − Σ_out mdot·h_out + otras transferencias de energía`;

`dm/dt = Σ_in mdot − Σ_out mdot + fuentes de masa compatibles`, y balance de especies (O₂, N₂, vapor combustible, productos/residuales) a nivel de mezcla escogido. Se debe mantener una definición precisa de entalpía/energía interna y tratar la energía química consistentemente para no contarla dos veces.

**Jerarquía física**:

1. 0D ideal para enseñar balances y verificar ecuaciones.
2. 0D real reducido con `cp(T, mezcla)`, trabajo por crank angle, pared térmica, inyección/encendido, residuales y gas exchange parametrizado.
3. Un/dos zonas (quemados/no quemados) cuando lo justifiquen temperatura/knock/HT.
4. Redes 0D/1D de admisión/escape: solo al obtener datos de puertos/volúmenes/longitudes.
5. Solvers 1D/CFD especializados **offline** para crear referencias y surrogates; si datos insuficientes, limitar dominio.

Como banco comparativo se puede usar Cantera, pero su ejemplo oficial ICE disponible usa un combustible y ciclo de tipo diésel con simplificaciones, **no es baseline gasolina del D16** [R06].

## 6.4 Combustión, chispa y avance

Prototipo: función de masa quemada tipo Wiebe, por ejemplo:

`x_b(θ) = 1 − exp[−a · ((θ−θ_start)/Δθ_burn)^(m+1)]` dentro de ventana definida.

Parámetros `a,m,θ_start,Δθ_burn` **NO universales**. Dependen de RPM, carga, AFR/lambda, residuales, presión, turbulencia, timing, geometría y temperatura; calibrar con presión indicada si existe, o declarar predicción como exploratoria. `Qdot_chem` procede de masa efectiva de combustible quemado, composición y poder calorífico/entalpías, no de `boost * horsepower_gain`.

La ECU ordena evento de chispa y dwell; la combustión solo puede arrancar si hay masa/mezcla inflamable, compresión/carga dentro del modelo, sistema de encendido capaz y condiciones de propagación. Separar **timing solicitado** de **timing efectivo** y de **fase real de liberación de calor**. Retardar encendido reduce/traslada trabajo por presión vs ángulo, no aplica “−3% HP” arbitrario.

**Detonación/knock:** modelos reducidos (ej. integral de Livengood-Wu) dependen de calibración química y mezcla; artículos muestran sensibilidad considerable a datos cinéticos [R18]. En primera base, `knock_predictor=UNSUPPORTED` si no hay validación. No simular daño instantáneo por “AFR demasiado pobre” como verdad universal. Riesgos de detonación, preignición y choque mecánico son mecanismos distintos.

## 6.5 Gas exchange y eficiencia volumétrica

Primero red 0D con nodos de ambiente, filtro, mariposa, plenum y runner/cilindro/exhaust manifold y elementos de restricción/caudal (incluido flujo reversible). En válvulas, alzada `L(θ_cam)`, área efectiva `A_eff(θ)`, coeficiente de descarga `Cd(Δp, Re, lift)` → flujo compresible; estrangulamiento (*choked flow*) cuando corresponda.

No usar `VE(RPM)` arbitraria como única fuente de llenado **si al mismo tiempo se presume modelar de forma causal el efecto de cam/puertos/escape**. Dos opciones honestas:

- `L1`: VE(RPM,MAP,T) medida/ajustada: modo stock calibrado, pero cambios de cam/puertos **no son predictivos**.
- `L2`: lift/valve flow + red 0D/1D con pérdidas, longitudes y ondas según alcance: capaz de responder causalmente a más modificaciones, con mayor costo y calibración.

No mezclar modelos con pérdida o duplicación de masa. Incluir parámetros ambiente (p,T,humedad), carga, temperatura de admisión y composición del combustible.

## 6.6 Eficiencia y torque, con auditoría energética

Reportar **por cilindro y agregado**:

- `IMEP_gross`, `PMEP`, `FMEP`, `BMEP` con convenciones matemáticas explícitas.
- `indicated_work`, `pumping_work`, `friction_work`, `brake_work`, `thermal_loss`.
- `BSFC` y eficiencia térmica **solo** si flujo de masa combustible y energía de combustible están justificados.
- Error relativo de masa/energía en cada ciclo, error integración `p·dV` y balance de trabajo del cigüeñal.

Un pequeño modelo que respeta balances y declara error numérico es más útil que uno que entrega 300 hp sin ruta de cálculo.

---

# 7. Electrónica y ECU: circuito físico, no DTC programados

## 7.1 Señales de posición y sincronización

El manual Honda presenta CKP/TDC/CYP para generaciones aplicables del Civic 1996–2000 y describe también diferencias de diagnóstico/años [R01]. Documentación Haltech describe de manera general la distinción entre trigger y home/cam para determinar posición de cilindro sobre el ciclo 720° [R08]. **No asumir** que el D16Y7 OEM es un sistema moderno de rueda 36-1 ni que todos los sensores estén ubicados en el mismo sitio para todos los años.

Modelo de señal en capas:

1. **Mundo físico:** posición/velocidad real de eje, juego, runout, perfil de rotor y sensor instalado.
2. **Transductor:** variable electromagnética/sensado, forma de onda vs tiempo, amplitud dependiente de RPM/distancia/T si aplica.
3. **Red eléctrica:** cable, conector, masas, resistencia, interferencia, opens/shorts, alimentación.
4. **Adquisición ECU:** sample/filter, umbral, edge detection, antirrebote, dropout y timestamp.
5. **Decodificador:** tooth/event pattern, fase/revolución, pérdida/recuperación de sincronismo.
6. **Scheduler:** programa actuadores y calcula RPM/eventos; ninguna chispa debería saltar porque el render recorrió una escena.
7. **Actuación:** pulsos eléctricos, igniter/bobina/bujía, inyectores, relés; el resultado depende de capacidades reales.

Se modelan por separado fallos como señal ausente, débil, invertida, offset mecánico de distribuidor, cable cortado, conexión intermitente, masa mala, circuito abierto y desincronización. **CKP por sí solo no necesariamente determina fase de 720°**, de ahí señales de referencia de cilindro/hogar según ECU.

## 7.2 Estado de ECU

```text
ECUState {
  power_rail, ground_reference, boot_state,
  crank_sync, cycle_phase_sync, sync_confidence,
  sampled_inputs, filtering_history,
  injection_scheduler, ignition_scheduler,
  fuel_trim_state, idle_control_state,
  diagnostic_monitors, dtcs, fallback_mode
}
```

No asignar directamente síntomas a `dtc=P0335`. El DTC es la **interpretación de un monitor** que observa señales fuera de condiciones esperadas, con tiempo de habilitación, umbrales, criterios y memoria. El motor puede fallar sin DTC; también puede almacenar código sin que haya destrucción mecánica.

## 7.3 Sensores objetivo y variables [P: verificar variante]

| Elemento | Magnitud física | Modelo realista incremental | Consecuencia posible / diagnóstico |
|---|---|---|---|
| CKP/TDC/CYP | posición/velocidad/fase | pulsos por geometría, noise, dropout, offset, cableado | timing/sync/combustión; osciloscopio |
| MAP | presión absoluta plenum | respuesta/filtro/saturación/offset, alimentación | cálculo carga; scan data y multímetro |
| TPS | ángulo mariposa | señal continua o interruptor según hardware, fallos | enriquecimiento/transitorios; multímetro |
| IAT | T aire | termistor/tabla R(T), masa térmica, wiring | densidad estimada/correcciones |
| ECT | T refrigerante | termistor/RC térmico, wiring | enrich/ralentí, ventilador según control |
| O2/sonda lambda | oxígeno residual / equivalence ratio inferida | dinámica de calentamiento/saturación, narrowband o clase OEM | correcciones closed-loop cuando aplica |
| IACV | actuador paso de aire | ley flujo/estado mecánico, alimentación | control ralentí |
| Fuel injector | masa vs pulso/Δp/dead time | tiempo apertura/cierre, dispersión | mezcla y combustión desigual |
| Bobina/igniter/distribuidor | corriente y energía de chispa | dwell, saturación, tensión y pérdidas | misfire/fase/energía |
| Batería/alternador | voltaje, resistencia, carga | SOC, temperatura, ripple/carga | velocidad de arranque y ecuación de actuadores |
| CKF / knock / EGR / VSS | **solo si variante tiene cada uno** | no crear sensor sin hardware | DTC/monitor específico |

**Principio:** una lectura de ECT errónea puede cambiar el comando de combustible/ralentí, pero no modifica instantáneamente la temperatura física del motor; esta solo cambia por energía/flujo.

## 7.4 Orden incorrecto de encendido y distribución

Separar cuatro escenarios:

1. **Cam/crank mecánicamente desfasados**: alters valve events → masa atrapada/compresión efectiva y potencial contacto; ECU puede medir fase distinta según montaje del sensor.
2. **Cables de bujía intercambiados**: spark se entrega a cilindro incorrecto; el cigüeñal y la secuencia mecánica NO cambian.
3. **Tabla/scheduler incorrecto**: ECU dispara eventos mal aun con rotor correcto.
4. **Pérdida de sync CKP/CYP**: puede impedir/dislocar autorización de inyección/encendido según estrategia OEM a modelar.

Evitar narrativas predeterminadas de “biela sale disparada”: una biela rota requiere cargas/fatiga/contactos/estado material suficientes y un modelo de falla. Un encendido incorrecto puede causar no-start, backfire o daño indirecto, pero no implica automáticamente una biela expulsada.

---

# 8. Fricción, aceite, refrigeración, fugas, fatiga y desgaste

## 8.1 Lubricación

No sustituir lubricación por `has_oil=true`. Modelo gradual:

- Aceite: masa/nivel, grado/curva viscosidad `μ(T)`, densidad, aireación/contaminación, degradación.
- Bomba: desplazamiento, eficiencia, succión, relación RPM, válvula de alivio y desgaste.
- Red hidráulica: galerías, restricciones, fugas, filter, cojinetes como ramas dependientes de clearance/T.
- Película de aceite en cojinetes: modelo reducido lubricación hidrodinámica con régimen de película y contacto mixto; no afirmar Reynolds 2D exacto sin geometría y validación.
- Fricción/temperatura/desgaste: velocidad, carga, viscosidad y rugosidad; presión global de aceite **no equivale** a asegurar buena película en cada contacto.

Caso de estudio: arandela/tapa mal montada altera holgura de bancada → mayor fuga interna → presión/caudal redistribuidos → pérdida de película bajo carga → temperatura superficial/daño. Es necesario un acoplamiento real, no `damage += 1` al detectar montaje malo.

## 8.2 Refrigeración y calor

- Capacitancias térmicas separadas: pared de cilindro, culata, pistón, aceite, refrigerante, intercambiador, metal circundante.
- Flujos: combustión→paredes, pared→coolant/oil/ambiente, bomba→caudal, termostato→apertura, radiador→ambiente.
- Propiedades: calor específico, conductividad/coeficientes de película, presión/ebullición del circuito, expansión material.
- En primer nivel usar RC térmico/transient 0D; posteriormente discretización espacial donde importe gradiente/deformación.
- **No** determinar directamente daño de junta si ECT supera X; presión local y mecanismos de fallo deben intervenir.

## 8.3 Anillos, paredes, blow-by y leak-down

Cada cilindro tiene una geometría simplificada de deformación y desgaste espacial `r_bore(z,φ)`, rugosidad/rayado y tolerancias; la representación completa 3D es optativa. Los anillos tienen gap, tensión, asiento, desgaste, temperatura, libertad de movimiento. Un modelo de conductancia de fuga entre cámara→cárter surge de holgura/contacto/presión/temperatura. El blow-by alimenta presión cárter y PCV, reduce trabajo indicado e influye en aceite/contaminación.

- **Compression test**: proceso virtual: motor gira con starter, mariposa/configuración de prueba, válvulas y fugas; gauge con check valve y respuesta propia.
- **Leak-down test**: aire regulado externo en un ángulo elegido (idealmente PMS de compresión para prueba estándar), salida hacia cárter, admisión, escape o refrigeración según rutas reales.
- Un valor de leak-down y una prueba de compresión **no identifican de forma única** el componente dañado; deben coexistir diagnósticos ambiguos.
- Estado de cilindro por zonas: ovalidad, taper, rayaduras, acabado/honing y desgaste progresivo; método de medición con micrómetro/bore gauge.

## 8.4 Desgaste, fatiga y rotura

Razonar en **dos relojes**: eventos rápidos por ciclo/θ y acumulación lenta por tiempo/ciclos/carga/temperatura.

- **Desgaste adhesivo/abrasivo/corrosión**: mecanismos distintos; ley empírica de desgaste solo calibrada para condiciones y materiales relevantes.
- **Fatiga por cargas alternantes**: modelos S–N/Miner simplificados, distribuciones materiales e historial, con advertencia de baja predictibilidad fuera de ensayos.
- **Deformación plástica**: relación esfuerzo/deformación y rigidez local; modelo reducido vs FE offline.
- **Impacto pistón-válvula**: detección geométrica y cálculo de energía de contacto; daño depende de rigidez/material y dinámica, no de etiqueta `timing_bad`.
- **Falla de junta**: pérdida gradual/instantánea de estanqueidad dependiendo de compresión local, T/p y evolución, no booleano fijo.
- **Reparación**: rectificado, pistón sobremedida, reemplazo, limpieza y verificación dimensional modifican el estado *as-built*; no reinician todas las variables del bloque automáticamente.

**Variabilidad física ≠ aleatoriedad inmotivada**: introducir distribuciones documentadas para materiales/tolerancias/lotes y semillas explícitas cuando exista evidencia. Con parámetros constantes y seed fija, la trayectoria simulada se repite. Si la precisión de predicción de roturas es baja, presentar intervalos/clase de riesgo, no prometer un número de ciclos exacto.

## 8.5 Causalidad como grafo comprobable

Ejemplo: `bolt preload → gasket sealing stress → leakage conductance → m_dot(blowby/coolant) → cylinder p/T → indicated torque/temperature → sensor readings → ECU response`.

El grafo sirve para inspección, **pero no reemplaza** las ecuaciones de conservación ni determina automáticamente los coeficientes. Tests metamórficos: cambiar apriete **no puede** modificar directamente mezcla química; deberá haber una ruta física. Si un efecto esperado no emerge porque el submodelo está L0, reportarlo **no implementado** en lugar de agregar un parche causal sin justificación.

---

# 9. Diseño numérico, scheduler e integraciones

## 9.1 Problema híbrido

El simulador es un sistema híbrido de ODE/DAE + eventos discretos:

- `dstate/dt = f(state, inputs, parameters)` para rpm, energías, flujos, temperaturas y estados de circuitos.
- `g(state)=0` restricciones de conexiones cinemáticas/contactos/flujo según formalismo.
- Eventos a ángulo (`IO`, `IC`, `EO`, `EC`, chispa), a tiempo (flancos sensor, pulsos injector), y a condición (válvula abre, contacto, fallo numérico).

**No** forzar todo a un único `update(deltaTime)` de render. Cerca de ω=0 la parametrización por θ se vuelve singular; usar integrador en tiempo para arrancar, detener y reversa, y avanzar crank-angle cuando sea ventajoso, con mapeo y localización precisa de eventos.

## 9.2 Orden de cálculo y conservation

Para cada paso aceptado:

1. Verificar consistencia de topología y eventos pendientes.
2. Resolver/actualizar restricciones de cuerpo rígido y ángulos de ejes.
3. Evaluar movimiento válvulas, volúmenes y áreas.
4. Calcular flujos red gas/aceite/refrigerante bajo esquema numérico elegido.
5. Resolver balances de masa, energía y especies por cilindro/volumen.
6. Evaluar control ECU y actuaciones en los instantes programados; resolver acoplamiento explícito/implícito según rigidez.
7. Calcular fuerzas/torques, resistencias y evoluciones térmicas/desgaste.
8. Ejecutar checks de invariantes, balances e incertidumbre; aceptar/rechazar paso.
9. Publicar **snapshot inmutable** para instrumentos/render a una tasa desacoplada.

**Precaución:** el orden es una guía conceptual. La implementación numérica debe analizar estabilidad y acoplamiento: un sistema fuertemente acoplado puede requerir resolver implícitamente el conjunto o subiterar; actualizar en serie puede introducir artefactos no físicos.

## 9.3 Estrategia numérica por niveles

- Referencia analítica: soluciones exactas/idealizadas (mecanismo, masa cerrada, PV reversible).
- Referencia alta precisión: SciPy/Cantera/solver externo por escenarios, separado del core de juego.
- Runtime: `f64`, tolerancias explícitas, integrador documentado (Runge-Kutta adaptativo para ODE no rígidos; método apropiado para sistemas rígidos/DAE si necesario), event roots y tolerancia angular.
- Integración: comparar error al refinar paso; **sin convergencia, no aceptar** resultados de calibración.
- No usar `f32` para balances acumulativos solo para reducir memoria; UI puede recibir `f32`/resúmenes.
- Solvers con resultados no negativos/conservativos: controlar overshoot de masa, temperatura, entalpía y composiciones; documentar limiters.
- Profiles diferentes (rápido/preciso) deben identificarse en outputs y validarse por separado.

## 9.4 Benchmarks planificados (umbrales sujetos a prueba)

Métricas: segundos físicos simulados/segundo de CPU, tiempo por evento, peak RSS, curva error vs tolerancia, memoria de replay, latencia comandos/telemetría, drift y rendimiento bajo 4× rpm/lotes. Probar máquina de gama baja y media; un benchmark en un procesador potente no justifica prometer realtime universal.

**Dimensionalidad:** a 7000 rpm, discretizar cada 1° de cigüeñal equivale a ~42 000 pasos angulares/segundo físico para **todo el motor**, no 42 000 por cilindro; las cuatro cámaras se evalúan dentro del paso. Para mayor detalle de combustión/eventos el paso podría bajar, multiplicando costo. El costo real depende de número de volúmenes y complejidad del solver; **benchmark, no adivinación**.

---

# 10. Arquitectura de software: core con APIs independientes

```text
engine-engineering-sandbox/
  README.md
  LICENSE / THIRD_PARTY_NOTICES.md
  docs/
    foundation.md                 # este archivo (versión de repo)
    adr/                           # decisiones arquitectónicas
    physics/                       # teoría + derivaciones + unidades + dominios
    variants/                      # D16Y7 año/mercado/ECU fijados
    validation/                    # planes, datos reales, incertidumbres
    sprints/                       # un fichero por sprint + SPRINTS.md índice
    PROGRESS.md                    # avances constatables
    HANDOFF.md                     # punto de entrada al siguiente agente
    limitations.md                 # limitaciones y no-soportado
    model_assumptions.md           # hipótesis explícitas
  crates/
    units/                         # cantidades SI, tolerancias, conversiones
    provenance/                    # metadatos de parámetros y datasets
    engine-types/                  # IDs, estados, eventos, errores, versiones
    assembly/                      # catálogo de piezas/instances/joints/procesos
    math/                          # numerics, ODE, root detection, interpolation
    kinematics/                    # biela-manivela, cam, transformaciones
    thermodynamics/                # propiedades mezcla, energía, sustancias
    flow-network/                  # volúmenes, caudales, válvulas, fugas
    combustion/                    # burn-models y release of heat
    crank-dynamics/                # inercia, torque, starter/load, pérdidas
    ecu/                           # scheduler, trigger decode, estrategias
    electrical/                    # supply, sensors, cables, actuators
    lubrication/                   # bomba, galerías, película simplificada
    cooling/                       # lumped thermal network
    degradation/                   # desgaste/daño, con incertidumbre
    diagnostics/                   # medidores/oscilloscope/scan/leak-down
    simulation/                    # orquestación, event loop, snapshots
    scenarios/                     # parser, runner, truth fixtures
    telemetry/                     # traces y métricas numéricas
    cli/                           # CLI headless
    desktop-bridge/                # opcional, interfaz native/IPC futura
    web-bridge/                    # opcional, WASM futura
  data/
    d16y7/reference_variant.yaml   # años y variante explícita
    parts/ ... materials/ ... sensors/ ... curves/ ...
    calibration/ ...
  benchmarks/
  tests/
    unit/ property/ regression/ metamorphic/ integration/
    conservation/ convergence/ reference/ faults/ replay/
  experiments/
    ideal_closed_cylinder/
    crank_no_combustion/
    baseline_idle/
    cylinder_leak/
    wrong_timing/
    weak_ckp/
  ui/                             # React/Three.js/Tauri SOLO más adelante
```

**Las crates son una propuesta de límites conceptuales**, no mandato de crear 19 crates vacías en el Sprint 0. Inicialmente usar pocas crates con fronteras limpias y extraer cuando exista lógica/protocolo estable. No crear microservicios ni colas de red por defecto; motor headless in-process. CLI y UI nunca escriben estados internos: solo envían comandos/lecturas tipadas.

## 10.1 Contratos de API (esquema ilustrativo)

```text
create_engine(variant_ref, parameter_pack, seed) -> EngineId
get_state(engine_id) -> ReadOnlyPhysicalSnapshot
list_components(engine_id) -> [PartInstance]
apply_operation(engine_id, Operation, expected_version) -> OperationResult
configure_simulation(engine_id, Profile) -> ConfigResult
step_simulation(engine_id, simulated_duration) -> StepResult
run_scenario(scenario_file, locked_run_manifest) -> RunBundle
measure(engine_id, InstrumentConfig, ProbeConnection) -> Observation
save_state(engine_id) -> VersionedSnapshot
load_state(snapshot) -> EngineId
replay(run_manifest, operation_log) -> RunBundle
```

- `get_state` para motor/experimentos, **no** para la pantalla del jugador sin permisos: el jugador debe utilizar instrumentos y no un inspector omnisciente.
- Separar `GroundTruthSnapshot` (tests/ingenieros) y `ObservedState` (jugador/instrumentos).
- `OperationResult` retorna si un comando se pudo aplicar por motivos estructurales; **no** si el usuario “lo hizo bien”.
- Contratos semánticos versionados: cambios materiales del modelo actualizan versionado y invalidan snapshots cuando corresponda o migran explícitamente.

## 10.2 Interfaz desktop y web

- **Core** compila nativo primero; benchmark y tests sin Tauri.
- **Tauri** shell desktop en fase visual [R13], Three.js para modelo 3D y UI de instrumentos; trabajar con snapshots a frecuencia razonable y event streams.
- **WASM opcional**, sin duplicar ecuaciones en TypeScript. Si alguna biblioteca es incompatible o rendimiento insuficiente, web soportará un subconjunto de perfiles validado y anunciará límites.
- **Contenido (CAD, esquemas, manuales)** versionado y licenciado independientemente del código.

## 10.3 Concurrencia y determinismo

- Una **autoridad** para estado de simulación. Paralelizar lotes independientes o dominios con orden de reducción estable cuando el rendimiento lo justifique.
- Evitar dependencia de resultados de carreras de hilos/GPU para una traza reproducible.
- Snapshot consistente por tick; event log monotónico; no depender de hora real para eventos físicos.
- Fijar versión de compilador, dependencias, flags de floating point y entorno de pruebas; registrar SHA256 de inputs y binario/revisión.
- Replay bitwise solo exigible bajo entorno *idéntico y controlado*; para cross-platform, comparaciones con tolerancias, curva y orden de eventos.

---

# 11. Datos, calibración, identificabilidad y restricciones legales

## 11.1 Datos por confianza y procedencia

Nivel de fuente preferido: **OEM manual exacto / ficha del fabricante del componente / ensayo medido y descrito > paper revisado > dataset comunitario con metodología > foro/video anecdótico**. La reputación no sustituye calibración; tomar datos de motores modificados solo si especificación y condiciones se conocen.

Para cada parámetro conservar el registro:

```yaml
parameter_id: d16y7.geometry.bore
value_si: null           # hasta verificación con página OEM
unit: m
uncertainty:
  type: unknown
  reason: source_page_not_reviewed
source:
  id: R01
  manufacturer: Honda
  publication: Civic Service Manual 1996-2000
  page_or_section: null
  variant: US_1996_1998_D16Y7_MT_UNCONFIRMED
  retrieval_date: 2026-10-09
measurement:
  method: documentation
  lab_environment: null
status: PENDING_VERIFICATION
```

**No convertir un dato faltante en `0.0`**, ni mezclar valor nominal y valor medido. `nominal_spec`, `manufacturing_distribution` y `as_built_measurement` son canales distintos.

## 11.2 Identificabilidad

Si solo conocemos potencia a 6600 rpm, **incontables combinaciones** de VE, fricción, timing, combustión y temperatura reproducirán ese dato. Eso es ajuste arbitrario, no validación. Para identificabilidad necesitamos observaciones diversas: curvas completas con condiciones, consumo, MAP, lambda, presión de aceite, temperatura, idealmente `p(θ)` y ensayos controlados de componentes. Usar juegos separados: `calibration_data` y `holdout_validation_data`.

**No calibrar y validar contra la misma curva.** Parametrizar incertidumbres (por fuente/tolerancia del sensor/variación por motor) y utilizar barridos de sensibilidad para detectar parámetros que no se pueden identificar.

## 11.3 Licencias y activos

- Manual Honda [R01]: **documento de referencia**; no empaquetar scans/páginas/diagramas OEM sin derechos adecuados. Resumir hechos y crear arte propio con trazabilidad y revisión legal antes de distribución.
- Speeduino [R04]: repositorio **GPL-2.0**; documentar contaminación accidental y hacer implementación independiente de conceptos; revisar con asesoría si se plantea vincular su código o derivarlo.
- Engine-Sim [R03a]: licencia **MIT** (cumplir aviso) y advertencia explícita de no aptitud científica.
- Cantera [R06a]: **BSD-3-Clause**; idóneo como verificador de modelos, pero examinar dependencias y datos de mecanismos químicos.
- OpenWAM de CMT-UPV [R07]: **licencia exacta no confirmada en esta revisión**; no integrar fuente ni binarios antes de auditarla. ¡No confundir con repositorios homónimos de IA!
- Si se usa un STL/CAD de tercero: capturar permiso explícito, restricciones comerciales, derivados, redistribución y atribuciones.
- Marca Honda y denominaciones: investigar uso nominativo, presentación, no inferir autorización comercial.

## 11.4 Degradación del modelo y gestión de cambios

Cada submodelo tiene `ModelCard`: objetivos, ecuaciones, dominio (RPM/MAP/T), fuentes, parámetros, pruebas, incertidumbre, estado, versión, responsable y límites conocidos. Al reemplazar un modelo L1→L2 mantener puertos e interfaces; ejecutar regresión de invariantes, revalidar resultados comparativos y subir versión semántica del `ModelPack`.

---

# 12. Reproducibilidad: requisitos verificables

## 12.1 Manifiesto de una simulación

```yaml
schema_version: 1
experiment_id: cranking_healthy_001
engine_variant: us_civic_d16y7_reference_pending
model_pack: physics-pack-v0.1
parameter_pack_sha256: "<sha256>"
assembly_snapshot_sha256: "<sha256>"
operation_log_sha256: "<sha256>"
seed: 20261009
software:
  git_commit: "<git-sha>"
  rust_toolchain: "<exact-version>"
  cargo_lock_sha256: "<sha256>"
  target_triple: "<target>"
  compiler_flags: "<recorded>"
integrator:
  name: "<solver>"
  abs_tol: "<specified>"
  rel_tol: "<specified>"
  max_step_s: "<specified>"
scenario:
  ambient_pressure_pa: "<specified>"
  ambient_temperature_k: "<specified>"
  initial_oil_temperature_k: "<specified>"
  initial_coolant_temperature_k: "<specified>"
  engine_initial_omega_rad_s: 0
  commands: []
outputs:
  timeseries: traces.parquet
  summary: metrics.json
  checksums: hashes.json
  validation_report: validation.json
```

**Los valores `<specified>` son campos obligatorios del runner, no valores textuales aceptados por el parser.** Evitar un `seed` global si distintos subsistemas requieren streams aleatorios independientemente: derivar semilla por nombre de dominio para que agregar sensor no cambie todas las tolerancias.

## 12.2 Niveles de tests

1. **Unit tests**: conversiones, geometría, propiedades de mezcla, formatos.
2. **Property-based**: dimensiones positivas, simetrías, relaciones de fase, invariantes energéticos.
3. **Golden math**: resultados analíticos de casos sencillos; no golden de resultados fabricados a mano.
4. **Conservation**: masa, especies, energía/trabajo de forma balanceada.
5. **Convergence**: paso temporal/ángulo refinado y estudio de orden del método.
6. **Metamorphic**: dirección de efectos en condiciones controladas (ej. menor conductancia de fuga → no puede aumentar caudal de fuga si Δp constante).
7. **Cross-model**: comparación contra Cantera/SciPy o referencias especializadas.
8. **Integration**: ECU→inyector→cylinder→crank, CKP→ECU→spark, arranque→ralentí.
9. **Fault injection**: cambiar estado/cableado y observar síntomas *calculados*, no hardcode.
10. **Snapshot replay**: reejecución bajo entorno fijado, checksums y orden estable.
11. **External validation**: curvas OEM/medidas, ensayo físico independiente, incertidumbre reportada.
12. **Performance**: run-time realtime, tolerancias, latencias, memoria y profiling.

## 12.3 Calidad: gates universales

Ningún sprint físico se cierra sin:

- ADR si hubo decisión nueva de modelo/stack.
- Modelo y ecuación con fuente, unidades, dominio y limitaciones.
- Parámetros versionados + prueba que falla sin el mecanismo (no “mock always passes”).
- Balance físico si aplica y estudio de convergencia para solver.
- Escenario CLI reproducible con manifiesto y baseline permitido.
- Protocolo de validación externa o etiqueta clara de **solo verificado**.
- Actualizar `SPRINTS.md`, `PROGRESS.md`, `HANDOFF.md`, `limitations.md`.
- Una demostración reproducible: comando `cargo ...` y output esperado, no screenshot como única evidencia.

**Criterio de aceptación:** los porcentajes de error cuantitativos **solo se fijan cuando se conocen error de referencia, incertidumbre del instrumento y sensibilidad del modelo**. Para invariantes matemáticas sí definir tolerancias numéricas según escala y precisión. No proponer “≤2% de HP” sin mediciones.

---

# 13. Plan de ejecución — FASE A: fundamentos reproducibles (S00–S13)

**Gran hito A = FOUNDATION-KERNEL**, no «motor completo». Los sprints se ejecutan secuencialmente, cada uno en branch/PR (o equivalente) independiente; paralelizar solamente trabajos sin interfaces inestables. **No saltar gates para mostrar progreso visual.** La numeración es estable y se mantienen historias de decisiones y cambios.

## S00 — Charter, alcance científico y reglas del repositorio

**Dependencias:** ninguna. **Objetivo:** acordar qué significan “realista”, “reproducible” y “terminado”.

**Tareas:**
1. Crear repositorio, `README.md`, `docs/foundation.md`, `docs/SPRINTS.md`, `PROGRESS.md`, `HANDOFF.md`, `limitations.md`, licencia a decidir y plantilla ADR.
2. Convertir las 12 ADR iniciales a registros de decisión con alternativas y triggers de revisión.
3. Definir roles (arquitectura, física, validación, datos/CAD, agentes), responsables y reglas de cambios.
4. Redactar la política de software de simulación **no apto para decisiones de seguridad real** hasta validaciones apropiadas. Incorporar el JTBD de §0.1 como charter, con prioridad **comprender → experimentar → diagnosticar**, control contrafactual y alcance L0–L4 explícito.
5. Definir `definition-of-done.md`, criterios de rechazo y protocolo “falta evidencia → marca PENDING, no inventar”. Registrar que el **criterio de utilidad** se evalúa por subsistema y dominio de validez; separar causalidad explicable de precisión predictiva.
6. Decidir procesos de code review, CI y vetting de código generado por agentes.

**Entregables:** repositorio inicial, DO-D/ADR, charter con JTBD (§0.1), referencias a ambos research, matriz de riesgos, backlog documentado.  
**Gate:** cualquier colaborador puede explicar qué es un resultado *verificado*, *calibrado* y *validado*, localizar la fuente, explicar por qué priorizamos observación y experimentos contrafactuales, y saber dónde aportar una nueva ecuación. No hay código de física obligatorio.

## S01 — Variante Honda, manual OEM y ficha maestra del D16Y7

**Dep.: S00. Objetivo:** identificar exactamente el motor de referencia, sin mezclar años y configuraciones.

**Tareas:**
1. Fijar año/modelo/mercado/transmisión, configuración del distribuidor/ECU, números de parte cuando estén disponibles.
2. Revisar manual Honda [R01] por secciones: General, Engine Mechanical, PGM-FI, Ignition, Fuel & Emissions, Cooling, Lubrication, Electrical.
3. Crear **índice de procedencia por dato**, sección/página/edición y notas sobre diferencias 1996–1998/1999–2000.
4. Cargar tabla OEM de diámetro, carrera, compresión, fuerza de apriete/orden, tolerancias, alineación de timing, mapa de pines y sensores existentes.
5. Separar `nominal`, `service_limit`, `measurement`, `distribution`, `unknown`. Los service limits **no** son automáticamente tolerancias de fabricación.
6. Construir un inventario de ~piezas y puertos con granularidad progresiva; no confundir total de piezas del motor real con instancias del primer sprint.
7. Crear `unverified_values.csv` (cantidad desconocida explícita) y lista de fotos/CAD/datos aún requeridos.

**Entregables:** `data/d16y7/reference_variant.yaml`, `D16Y7_DATA_DICTIONARY.md`, bibliografía versionada, tabla de discrepancias entre fuentes.  
**Gate:** cero valores numéricos esenciales sin fuente identificable o etiqueta `PENDING`. ECU/sensores están asociados a **una variante** verificable.

## S02 — Estructura de código, toolchain, CI y runner headless

**Dep.: S00. Objetivo:** crear entorno reproducible antes de simulación.

**Tareas:**
1. Rust workspace mínimo (`engine-types`, `physics`, `simulation`, `cli` como agrupación inicial; extraer luego).
2. Pin de toolchain en `rust-toolchain.toml`, lockfile, lints, format, tests, Cargo Deny/auditoría de dependencias (tras evaluación).
3. CLI de ejemplo: `init`, `validate-data`, `run-scenario`, `replay`, `report` (inicialmente sin física, retorno de error tipado donde corresponda).
4. `run_manifest.json`, trazas JSONL/Parquet (según decisión), SHA de inputs/artefactos.
5. CI Linux/Windows con prueba de build, schema validation y tests sin network.
6. Error reporting estructurado; no `unwrap` en rutas de simulación de producción sin razones revisadas.

**Entregables:** runner mínimo y CI verde.  
**Gate:** un segundo equipo clona, ejecuta escenario de ejemplo y obtiene mismo hash de entradas/reportes bajo entorno documentado.

## S03 — Unidades físicas, constantes, contratos y álgebra

**Dep.: S02. Objetivo:** prevenir bugs dimensionales a gran escala.

**Tareas:**
1. Tipos para longitud, ángulo, fuerza, presión absoluta/manométrica, temperatura K, energía, masa, caudales y tiempo.
2. API de conversión visual (psi/bar/kPa; °C/K; N·m/lbf·ft; hp/kW), sin introducir esos tipos en core.
3. Codificar qué significa presión absoluta vs gauge, potencia al cigüeñal vs ruedas, AFR vs lambda; moles vs kg.
4. Definir redondeo y serialización de `f64`; política NaN/inf/subnormal.
5. Funciones de interpolación con límites, domain checking y dimensionalidad.
6. Property tests de conversiones, simetrías y consistencia.

**Entregables:** `units` + `math` inicial, especificación de dimensiones.  
**Gate:** compilador o validadores impiden sumar torque a temperatura, tratar bar-gauge como Pa absoluto o pasar RPM como rad/s sin conversión.

## S04 — Catálogo estructural, instancias y grafo de montaje

**Dep.: S01–S03. Objetivo:** un bloque virtual vacío al que se le pueden agregar piezas con relaciones explícitas.

**Tareas:**
1. Tipos `PartDefinition`, `PartInstance`, `Interface`, `Joint`, `Port` y IDs persistentes.
2. Registrar topología de conectores: mecanizados, contacto, transmisión torque, flujo aceite/agua/aire, señales eléctricas.
3. Versionar esquemas y agregar migrations/fail-closed.
4. Aplicar operaciones atómicas `install/remove/connect/disconnect`; validar geometría nominal, orientación y compatibilidad donde existan datos.
5. Distinguir acción físicamente imposible de acción permitida pero posiblemente destructiva.
6. Tests: montaje fuera de orden posible si físicamente factible, pieza ausente no aparece mágicamente, serial se conserva tras desmontar.

**Entregables:** motor en estado `bare_block` que puede recibir cigüeñal/pistones/head como instancias, sin render.  
**Gate:** grafos persistentes y operaciones trazables; todavía **no** simular su comportamiento físico.

## S05 — Eventos, snapshots, replay y determinismo controlado

**Dep.: S02–S04. Objetivo:** toda evolución del motor es reconstruible.

**Tareas:**
1. Evento de tiempo físico con orden total determinístico en empate; `event_id`, `timestamp`, `actor`, `payload`, versión.
2. Savefiles inmutables/versionados con `EngineState` y esquema migrable.
3. Log append-only de comandos y snapshots periódicos.
4. Replay restaurando escenario, seed, estados y parámetros sin reloj/aleatoriedad del sistema.
5. Hash/checksum del estado canónico (excluir campos irrelevantes de UI).
6. Pruebas randomizadas/property de aplicar operaciones y volver a cargar.

**Entregables:** replay de montaje y desmontaje que obtiene mismo estado bajo entorno bloqueado.  
**Gate:** reproducir 100 secuencias de acciones generadas de forma controlada sin divergencia estructural; tolerancias numéricas se definirán cuando exista física.

## S06 — Geometría cinemática del cigüeñal/pistón/biela

**Dep.: S03–S05. Objetivo:** posiciones físicas exactas en el modelo geométrico declarado.

**Tareas:**
1. Implementar relación biela-manivela, diámetro, carrera, radio y longitud de biela, volúmenes y fases.
2. Diferenciación/derivación de velocidad y aceleración de pistón.
3. Definir PMS/PMI, referencia angular y desplazamientos de cilindros, incluyendo 720°.
4. Incorporar masa/inercia del cigüeñal y masa alternativa simplificada **separadas** de topología.
5. Test de límite `r/l`, posición, volumen mínimo/máximo, simetrías y gráficos de referencia.
6. Comparar derivadas analíticas con finite differences de orden alto a pasos elegidos.

**Entregables:** CLI `kinematics --angle/rpm` produce series `x,v,a,V,dVdθ` por cilindro.  
**Gate:** conservación geométrica, continuidad y tests analíticos; no RPM “simuladas” aún.

## S07 — Operaciones de herramientas y contactos geométricos básicos

**Dep.: S04, S06. Objetivo:** separar montaje físico y usuario.

**Tareas:**
1. Tipos para tool/operation: llave torque, galgas, dial gauge, air supply; sin UI.
2. Definir superficies de interferencia esenciales (pistón-válvula, biela-block, crankcase, pernos), primero geometría simplificada.
3. Asegurar orientación y posición de piezas afecta los contactos.
4. Implementar reglas de *unión físicamente imposible* con medición de profundidad/diámetro, no checklist de juego.
5. Reportar contactos con energía/penetración y estado `needs_material_model` para no inventar roturas.
6. Tests con geometrias sintéticas simples y límites analíticos.

**Entregables:** operaciones de medición/montaje y prototipo de detección de interferencias.  
**Gate:** ningún daño mecánico se reporta sin material/criterio activo; errores de montaje se mantienen detectables mediante estado físico.

## S08 — Pernos, precargas, rigidez y junta de culata (mecánica básica)

**Dep.: S01, S03–S05, S07. Objetivo:** representar el torque de forma causal, sin “penalizador”.

**Tareas:**
1. Registrar torque aplicado, ángulo, lubricante/rosca, historia de pasos por perno.
2. Derivar precarga con modelo L1 de torque-factor y rangos/incertidumbres; estudios de `K`.
3. Representar junta/head/deck con segmentos y rigidez equivalente; **no** prometer FEM.
4. Modelo de contacto/compresión y presiones de sello por segmento.
5. Interfaz `SealConductance` para futura fuga física, antes de simular gas/agua.
6. Cotejar secuencia OEM cuando se verifique la variante; probar patrones de subtorque, pero **no imponer** distorsión por secuencia sin el mecanismo capaz de representarla.

**Entregables:** `fastener_and_seal_test` con precarga, campo de presión simplificado y estado de sello.  
**Gate:** variar torque cambia precarga bajo hipótesis declaradas; variar lubricante con mismo torque cambia precarga según modelo; reporte reconoce calibración pendiente.

## S09 — Solver de gas ideal / cilindro sellado sin combustión

**Dep.: S03, S06. Objetivo:** demostrar masa, energía, volumen y p/T por crank angle.

**Tareas:**
1. Definir `ThermoState` y mezcla de gases simple `R,cp,cv` por material/fuente.
2. Integrar compresión/expansión de cilindro cerrado ideal reversible; ejercicio baseline.
3. Resolver primera ley con geometría variable y pérdidas Q=0 inicialmente.
4. Integración ODE independiente del reloj render, con rejilla angular/adaptación.
5. Propagación de errores numéricos y límites de validez: V,m,T,p > 0.
6. Comparar con soluciones adiabáticas idealizadas y convergencia al refinar paso.

**Entregables:** curvas P–V y P–θ reproducibles, auditoría energética por ciclo cerrado.  
**Gate:** coincide con ecuación ideal dentro de tolerancia numérica justificada; no llamar el resultado “D16 validado”.

## S10 — Combustión reducida y presión indicada

**Dep.: S09. Objetivo:** liberar energía por combustión física idealizada.

**Tareas:**
1. Definir masa de combustible, poder calorífico y especies en un dominio reducido.
2. Implementar burn fraction paramétrica (ej. Wiebe), rango permitido, inicio y duración de combustión.
3. Acoplar liberación de calor a primera ley sin doble conteo.
4. Derivar torque indicado `p·dV/dθ` y trabajo neto del cilindro.
5. Barrido controlado de avance/masa combustible/calor de pared (primero Qwall=0, luego modelo L1).
6. No introducir calibración fija a hp de catálogo hasta disponer de gas exchange, fricción y datos.

**Entregables:** `closed_cycle_burn`, curvas P–V/θ, IMEP indicado y balance.  
**Gate:** retrasar heat release desplaza presión/curva de trabajo de manera matemáticamente explicable; sensibilidad revisada; todavía sin engine dyno al freno.

## S11 — Distribución SOHC: árbol, válvulas y correa

**Dep.: S06–S07, S09. Objetivo:** modelar timing físico antes de intake/exhaust.

**Tareas:**
1. `CamProfile` por válvula (lift vs θ_cam), coeficientes y holguras; perfil OEM hasta documentarlo es `[P]`.
2. Cinemática 2:1 y offset de correa; engranaje, dientes y fase configurable.
3. Estado de valvetrain: apertura/cierre, rocker, asiento, muelle (inicialmente cuasiestático).
4. Cambio de timing altera elevación vs fase de pistón y clearance/volumen efectivo.
5. Detectar contactos pistón-válvula a geometría y T del modelo; daño **pospuesto** a modelo material.
6. Pruebas de fase por 720°, TDC y timing belt con offset.

**Entregables:** `valvetrain --phase`, curvas lift y eventos IO/IC/EO/EC, informes de clearance.  
**Gate:** los cuatro cilindros derivan eventos de un camshaft y crank compartidos; **sin** falsos daños automáticos.

## S12 — Flujos de admisión y escape (nodos 0D)

**Dep.: S09, S11. Objetivo:** abrir físicamente el cilindro a ambiente/puertos.

**Tareas:**
1. Nodos con `m,U,p,T,composition` y elementos de conducción de flujo.
2. Flujo compresible a través de válvulas y mariposa con parámetros de área/descarga documentados.
3. Conservación total entre nodos para masas y energías; gas residual.
4. Colector/volumen de admisión, ambiente y escape con contrapresión como condición de frontera.
5. Integración de eventos apertura/cierre sin discontinuidades numéricas falsas.
6. Tests de flujo reversible, cierre hermético, choked/unchoked y masa global.

**Entregables:** ciclo 720° no combustivo con gas exchange, tracing por cilindro.  
**Gate:** el efecto de abrir/cerrar válvula se transmite por caudal calculado, no modifica VE a mano; declarar qué efectos de ondas/puertos 1D faltan.

## S13 — Combustible: depósito/rail/bomba/inyector reducido

**Dep.: S03, S10, S12. Objetivo:** masa de combustible procedente de un sistema físico.

**Tareas:**
1. Representar combustible identificado (propiedades, energía, temperatura), presión de rail y regulador simplificado.
2. Implementar inyector masa vs presión diferencial, duración, deadtime y tensión como dependencia futura.
3. Registrar eventos de inyección por cilindro o agrupación conforme a estrategia ECU que se fijará.
4. Separar cantidad comandada de combustible efectivamente entregada, incluyendo saturación/caudal insuficiente.
5. Casos: bomba off, caudal insuficiente, inyector inactivo, conexión desconectada.
6. Balance masa combustible y mezcla/cilindro con versión anterior del burn model.

**Entregables:** `fuel_supply_test` y curva masa combustible/evento físicamente consistente.  
**Gate:** evitar rendimiento calculado desde target AFR sin verificar entrega real; no DTC automático por bomba sin presión.

---

# 14. Plan de ejecución — FASE B: motor operable y electrónica (S14–S22)

**Gran hito B = RUNNABLE-ENGINE**: un D16Y7 modelado en banco puede hacer cranking y, bajo hipótesis declaradas, arrancar, acelerar y reaccionar a sensores/acciones. No exige validación completa del torque OEM. Cada módulo mantiene nivel de confianza independiente.

## S14 — Dinámica de cigüeñal y pérdidas mecánicas

**Dep.: S06, S10–S13. Objetivo:** pasar de trabajo indicado a evolución de RPM.

**Tareas:**
1. Implementar `J(θ)`/inercia efectiva coherente con el esquema elegido, sin contar fuerzas alternativas dos veces.
2. Integrar torques de combustión y ciclo completo; definir formalmente pumping y pérdidas.
3. Fricción reducida por RPM/T/carga y accesorios modelados con curvas calibrables.
4. Integrar carga de freno/inercia; motor en reposo, desaceleración y rotación impuesta.
5. Tests: `no_combustion + positive_friction → ω disminuye`; energía rotacional y trabajo.
6. Comparar step size/tolerancia y registrar componentes de error.

**Entregables:** `crank_dynamics` con series `ω, torque_indicated, friction, load, angular_momentum`.  
**Gate:** no energía gratis; la aceleración se corresponde con torques e inercia dentro de error numérico.

## S15 — Arranque: starter, batería y sistema de encendido

**Dep.: S13–S14. Objetivo:** el motor pasa por cranking real, no bootstrap a RPM objetivo.

**Tareas:**
1. Fuente eléctrica (batería con Voc(SOC,T), R_int, terminal voltage).
2. Modelo starter torque vs ω, corriente y voltaje; acoplamiento giratorio.
3. Modelo de compresión resistiva + fricción de arranque/temperatura.
4. Simplificar bobina/igniter/dwell y verificar que haya evento de chispa disponible.
5. Fallas: batería descargada, masa abierta, starter ausente, chispa nula.
6. Pruebas de conservación eléctrica+mecánica a nivel de resolución declarada y tiempos de crank plausibles (sin calibrar a videos sin metadatos).

**Entregables:** `cold_crank` por CLI, RPM instantánea fluctuante con compresión.  
**Gate:** `crank` sin combustible/chispa no arranca; hay trazas eléctricas inspeccionables.

## S16 — Integración: arranque, combustión por cilindro y ralentí

**Dep.: S10–S15. Objetivo:** primer *motor que puede arrancar*, sin escondites.

**Tareas:**
1. Integrar cuatro cámaras independientes mecánicamente acopladas al mismo crank.
2. Modelar ignición/inyección mediante eventos explícitos, no `if rpm < idle then rpm=idle`.
3. Ajuste de mezcla/mariposa/carga base; estabilización por balance de torque.
4. Añadir fuente de aire de ralentí básica/actuador según referencia futura.
5. Procedimientos run-on y shutdown coherentes; motor se para por falta de torque/combustión.
6. Escenarios repetibles `healthy_crank_start_idle`, `no_spark`, `no_fuel`, `high_friction`.

**Entregables:** primer **end-to-end** físico headless, telemetría completa con balances.  
**Gate:** mantener ralentí bajo condiciones declaradas sin clamp de RPM ni torque mágico; resultante aún puede no coincidir con stock Honda.

## S17 — Hardware y transductores CKP/TDC/CYP exactos

**Dep.: S01, S05–S06, S15–S16. Objetivo:** señales reales de fase y velocidad.

**Tareas:**
1. Confirmar geometría/ubicación rotor y canales para el **año exacto** elegido [R01].
2. Modelar flancos y forma temporal base, ruido, umbrales, desfase, tensión, air gap y wiring.
3. Diseñar API de muestreo para scope a MHz/kHz apropiado según señal (medir necesidad; no muestrear cada canal a la tasa de GUI).
4. Definir y probar niveles de señal según RPM, pérdida flanco y señales intermitentes.
5. Comparar patrones producidos con manual, esquemas/mediciones publicadas o banco real; distinguir genérico vs OEM.
6. Escenarios `correct_pattern`, `missing_edge`, `signal_open`, `distributor_offset`, `low_amplitude`.

**Entregables:** generador virtual de CKP/TDC/CYP con fixtures, waveform CSV.  
**Gate:** patrón de referencia trazado a fuente; si no se consigue, marcar `GENERIC_TRIGGER` y **prohibir etiquetarlo OEM**.

## S18 — ECU de referencia: trigger decoder y scheduler

**Dep.: S13, S15–S17. Objetivo:** hacer la ECU reactiva a sensores.

**Tareas:**
1. Decodificación de flancos con sincronización 360°/720° según hardware documentado.
2. FSM: `power_off`, `booting`, `cranking_unsynced`, `cranking_synced`, `running`, `sync_lost`, `fallback` (si se soporta).
3. Mapas de combustible/ignición de referencia, sensor filtering, compensaciones térmicas y condiciones de activación.
4. Planificador de salida por θ/t para inyectores y encendido; latencias/dwell/deadtime.
5. Monitores/DTC configurables según versión; el DTC se genera por lectura temporal, nunca se fuerza por “fault_id”.
6. Contrastar algoritmo con Speeduino como *referencia conceptual* [R04] y Honda OEM como especificación objetivo, sin copiar código GPL.

**Entregables:** ECU virtual que toma señales y produce comandos medibles, con logs de decisiones.  
**Gate:** las señales erróneas alteran `sync/timing/actuation` causalmente; timing efectivo vs solicitado son visibles para pruebas.

## S19 — Sensores analógicos, cableado y lectura de valores

**Dep.: S12–S13, S15, S18. Objetivo:** la ECU opera sobre **mediciones**, no ground truth.

**Tareas:**
1. Modelo de `SensorChannel` con transferencia, tensión de referencia, retardo, ruido, offset, drift, saturación.
2. MAP, IAT, ECT, TPS y los confirmados por variante; tabulación calibrada cuando posible.
3. Conectores/cables con `R`, circuito abierto/corto a masa/corto alimentación y fallos intermitentes reproducibles.
4. Sample/quantization de ADC y filtros ECU; discrepancia estado real vs estimado.
5. Inyección de fallas por **estado del cable/transductor** y test de DTC/actuación subsecuentes.
6. Pruebas con oscilaciones/step input y aceptación de tiempos de respuesta dentro del modelo y rangos.

**Entregables:** batería de pruebas electrónica + tablas signal/reading/ECU action.  
**Gate:** ECT falso afecta decisión ECU sin alterar temperatura termodinámica; MAP falso no cambia mágicamente la presión real.

## S20 — Herramientas diagnósticas (sin pistas automáticas)

**Dep.: S05, S07, S17–S19. Objetivo:** el jugador deduce fallas a partir de datos.

**Tareas:**
1. Modelo de probe/punto de conexión, acceso físico, polaridad y referencia de masa.
2. Multímetro con rango/impedancia/medición de resistencia/voltaje.
3. Osciloscopio con timebase, muestra/aliasing/banda/calibración/trigger y posibilidad de grabar.
4. Herramienta de scan con lecturas y DTC **solo** los que ECU genera y protocolo de la variante permita.
5. Manual de servicio indexado como referencia (contenido legalmente autorizado o enlaces; no copiar libro completo).
6. Tests de fallo ambiguo: dos causas distintas pueden producir lectura similar hasta aplicar otra medición.

**Entregables:** API de instrumentación y casos de diagnóstico resueltos **por mediciones**.  
**Gate:** sin panel secreto de fallos ni notificación “te olvidaste de X”. Los tests internos conocen la causa únicamente para evaluación.

## S21 — Sistema térmico: bloque, culata, aceite y refrigeración L1

**Dep.: S09–S16, S19. Objetivo:** calentar/enfriar por transferencia de energía.

**Tareas:**
1. Nodos lumped térmicos para metal de block/head, piston, aceite, coolant y ambiente.
2. Transferencia combustión→paredes, metal→coolant/oil, radiador→aire; parámetros/calibración separados.
3. Termostato, bomba y caudales básicos como funciones físicas/estado.
4. Alimentar ECT/IAT y enriquecer arranque/cambios de fricción en circuito cerrado.
5. Tests de energía global, calentamiento transitorio y condiciones de borde.
6. Comparar escalas térmicas con bibliografía o medición; no inventar tiempos warmup stock.

**Entregables:** `cold_start_warmup` y `loss_of_cooling` con balances y señales de instrumentos.  
**Gate:** temperatura no cambia por comando directo del sensor ni por cambio de imagen; todo calor tiene una fuente/sumidero.

## S22 — Aceite/galerías/cojinetes y fricción dependiente de régimen

**Dep.: S14, S21. Objetivo:** el lubricante se convierte en una red de caudal/temperatura.

**Tareas:**
1. Bomba/relief, viscosidad vs T, nivel y galerías mínimas para cojinetes principales y de biela.
2. Modelar caudal por holgura/restricción y presión local (no solo global).
3. Efecto de temperatura/lubricación en fricción con modelo reducido documentado.
4. Alimentar módulo desgaste/falla futuro con `film_quality` o parámetros físicos mejor identificados.
5. Casos: aceite insuficiente, baja viscosidad caliente, cojinete con clearance excesivo, bomba defectuosa.
6. Tests: continuidad caudal, presión no negativa absoluta, límites de viscosidad y consistencia energética (trabajo bombeo/calor).

**Entregables:** `lubrication_test` con galerías, presión en puertos e incertidumbre.  
**Gate:** fallo de aceite afecta cojinete por ruta de presión/film/fricción, nunca una simple bandera de “durability=0”.

---

# 15. Plan de ejecución — FASE C: fallas emergentes y base auditada (S23–S30)

**Gran hito C = CAUSAL-DIAGNOSTIC-BASE.** Estas fases no son “detalles opcionales”: cumplen la visión de que un anillo fugando, un tornillo mal torqueado o una distribución errónea sean **estados físicos**, no eventos guionizados. Todavía puede ser un engine sin gráficos.

## S23 — Rutas de fuga generales: junta, válvulas, anillos, mangueras

**Dep.: S08–S12, S21–S22. Objetivo:** una única infraestructura conservativa de fugas.

**Tareas:**
1. Definir `FluidDomain`/`LeakPort` con dirección, área/conductancia efectiva, especies y entalpía.
2. Soportar fugas cámara→cárter, cámara→coolant, coolant→exterior, aceite→exterior, valve→intake/exhaust.
3. Derivar conductancia de junta desde `SealContactState`, no de daño booleano; conductancia de anillo desde holgura/condición L1.
4. Integrar caudal con redes y balances globales, incluyendo mezcla de fluidos si el modelo lo permite.
5. Instrumentos relevantes: leak-down, test de compresión, ECT/temperatura, presión cárter, observaciones visuales si se modelan.
6. Sensibilidad a Δp, T y tamaño de fuga con solución analítica simplificada y convergencia.

**Entregables:** tres escenarios `missing_head_gasket`, `underclamped_head`, `ring_leak_cyl3` conectados al mismo solver.  
**Gate:** ninguna de las tres fallas tiene un multiplicador directo de potencia ni una rutina que “rellena” síntomas; los flujos cambian estado termodinámico/mediciones de forma natural. Etiquetar fidelidad según parámetros obtenidos.

## S24 — Diagnóstico mecánico: compresión, leak-down y medidas geométricas

**Dep.: S20, S23. Objetivo:** medir las fallas reales de la simulación.

**Tareas:**
1. Compresímetro con volumen muerto/manómetro/válvula y ciclo de cranking.
2. Equipo leak-down con regulador, presión de suministro, orificios internos y medición en posición mecánica declarada.
3. Procedimiento de herramienta que conecte sondas a puertos reales (no reading global omnisciente).
4. Bore gauge y micrómetro: resolución, error de cero, temperatura, orientación y puntos de muestreo.
5. Experimento ambiguo: anillos vs válvula vs junta con lecturas parcialmente parecidas, test complementario separa causas.
6. Guardar `observation provenance` (instrumento, calibración, operador, timestamp y condiciones).

**Entregables:** protocolo `diagnose_low_compression` ejecutable sin GUI.  
**Gate:** tests internos demuestran que se diagnostica mediante al menos **dos mediciones independientes**, no lectura de flag de daño.

## S25 — Calibración del motor stock, sensibilidad e incertidumbre

**Dep.: S14–S24. Objetivo:** comparar con mediciones reales; no “fingir” 106 hp.

**Tareas:**
1. Definir campañas de adquisición: torque vs RPM con condiciones, consumo, MAP, lambda, temperaturas, arranque, compresión, P-V si disponible.
2. Elegir parámetros calibrables realmente identificables y fijar *priors* /rangos físicos.
3. Separar `train/calibration` y `holdout/external` por ensayo/motor; evitar leakage.
4. Fit de parámetros con informe de covarianzas/correlación y múltiples puntos operativos.
5. Sensibilidad local/global y perturbación de parámetros/tolerancias de fabricación.
6. Reportar intervalos, residuos, percentiles por régimen y condiciones, sin fijar accuracy universal.
7. Si no se consigue dataset de campo: cerrar como `NUMERICALLY_VERIFIED_ONLY`, bloquear marketing de precisión.

**Entregables:** `D16Y7_CALIBRATION_REPORT.md`, dataset manifest, plots y `ModelCard` por subsistema.  
**Gate:** todos los gráficos comparativos distinguen real/simulado y puntos usados en ajuste vs holdout; no inventar datos de dyno.

## S26 — Motor persistente: desgaste y mantenimiento bajo incertidumbre

**Dep.: S21–S25. Objetivo:** un mismo motor evoluciona después de horas de uso.

**Tareas:**
1. Definir *slow-state* persistente por anillo, cilindro, cojinete, cam, junta y pieza seleccionada.
2. Introducir leyes de desgaste simplificadas **solo** donde haya parámetros defendibles; demás mecanismos `UNSUPPORTED`.
3. Acoplar temperatura, presión de contacto, partículas/fricción, ciclos y lubricación.
4. Registrar eventos de servicio: reemplazar, medir, rectificar, bruñir, cambiar aceite y recalibrar.
5. Ensayos de aceleración de tiempo con validación de que no depende de FPS, refresh ni paso físico indebidamente.
6. Replicar desde snapshot/seed; comprobar que un motor nuevo y uno gastado no comparten estado indebidamente.

**Entregables:** `persistent_engine_50h` sintético etiquetado, saved game histórico y test de replay de desgaste.  
**Gate:** el desgaste sigue leyes/provenance y no depende de tasa de render; sin evidencia suficiente, solo *tendencias condicionadas*, no vida útil exacta.

## S27 — Contactos, plasticidad/fatiga y daños por montaje

**Dep.: S07–S08, S11, S22, S26. Objetivo:** habilitar **algunos** daños mecánicos no guionizados.

**Tareas:**
1. Delimitar contacto pistón-válvula, rosca arrancada, over-stretch de perno y pérdida película cojinete como primeros casos.
2. Elegir leyes reducidas de esfuerzo/rigidez/contacto con parámetros identificables, material y temperatura.
3. Introducir `DamageMechanism` con dominio de validez; evento irreversible solo si criterio físico se cumple según modelo.
4. Verificar energía/fuerza/contacto y continuidad de trayectoria antes y después del daño.
5. Pruebas negativas: timing desfasado sin colisión NO dobla válvula, par alto tolerable NO rompe perno automáticamente.
6. Probar sesgo de incertidumbre/parametrización y publicar límites.

**Entregables:** `piston_valve_collision_if_contact`, `bolt_yield`, `bearing_wear_under_oil_starvation` (este último solo si modelado/medido suficientemente).  
**Gate:** nunca inferir el instante de fallo real sin material/geometría/calibración; si no se puede sostener, conservar estado de riesgo en vez de una rotura teatral.

## S28 — Banco de pruebas y curvas de performance auditables

**Dep.: S14–S27. Objetivo:** separar torque real calculado de decoración del dyno.

**Tareas:**
1. Dinamómetro con inercia/carga frenante o control de velocidad, rampas, pérdidas y sensores.
2. Condiciones de ambiente/aceite/coolant, estabilización térmica y protocolización.
3. Medición consistente de par/potencia, incluyendo corrección estándar cuando esté justificada [R17].
4. Diferenciar potencia al freno, indicada, pérdidas auxiliares y correcciones.
5. Test con variaciones de restricción admisión, avance, temperatura, carburante/combustible y carga.
6. Documentar dónde son válidos los mapas del modelo y dónde se está extrapolando.

**Entregables:** `dyno_sweep_stock_d16y7` + curva y manifest; comparaciones con datos externos cuando existan.  
**Gate:** números de HP no pueden ser forzados desde “engine spec” de manera circular; se derivan del torque integrado.

## S29 — Suite adversarial de errores mecánicos combinados

**Dep.: S23–S28. Objetivo:** probar que el simulador no selecciona “un fallo dominante” arbitrariamente.

**Tareas:**
1. Escenarios pares: junta deficiente + cooling limitado, anillo fuga + ECT offset, timing offset + bajo voltaje, inyector abierto + baja compresión.
2. Invariantes bajo dos fallos simultáneos; probar conservación y consistencia de logs.
3. Identificar efectos compuestos no lineales frente a suma de penalizadores.
4. Diseñar test de observabilidad: lecturas idénticas parciales necesitan pruebas complementarias.
5. Comparar orden de aplicación cuando debería conmutar/no conmutar; registrar por qué.
6. Ensayos Monte Carlo con seed fija y distribución parametrizada; no usar generador de eventos sin justificación.

**Entregables:** `adversarial_fault_suite` y top 20 discrepancias catalogadas.  
**Gate:** cada síntoma se remonta a una ruta de ecuaciones; ningún `if fault_count>1` toma el control de la física.

## S30 — Auditoría científica y congelación de la base

**Dep.: S00–S29. Objetivo:** primer release `foundation-core-0.x` sin 3D obligatorio.

**Tareas:**
1. Auditar `ModelCards`, datos/fuentes/licencias, test coverage de invariantes (no solo líneas), violaciones CI, escenarios.
2. Ejecutar comprobación de reproducibilidad en dos máquinas y tres perfiles numéricos; cuantificar diferencias.
3. Auditabilidad de 4 cadenas: torque de culata→sellado; correa→lift/compression; CKP→spark; anillo→blow-by.
4. Revisar incertidumbre y comparaciones reales. Clasificar validaciones que siguen pendientes.
5. Benchmark CPU y determinar si el núcleo cumple realtime en hardware objetivo, o qué simplificar.
6. Publicar versionado, changelog, formato de snapshots y contrato de compatibilidad.
7. Exigir revisión independiente de experto de mecánica, idealmente termofluidos/controles, antes de declarar fidelidad L2+.

**Entregables:** `FOUNDATION_RELEASE_REPORT.md`, paquete headless y reproducible, limitaciones auditadas.  
**Gate:** sin etiqueta “predictivo de ingeniería” salvo evidencia. No inventar métricas para cerrar el sprint. Una base científicamente **incompleta pero honesta** puede pasar como *research foundation*, no como modelo plenamente validado.

---

# 16. Plan de ejecución — FASE D: taller 3D y expansión **después** de la base

Estas fases **no** deben competir con la prioridad del core. Se podrán prototipar CAD/UI en paralelo, pero **sin bloquear** ni “validar” ecuaciones con animaciones.

## S31 — CAD de referencia y jerarquía de piezas reales

- Crear requisitos mecánicos y contratos de montaje/escala/origen/pivots por pieza.
- Obtener/licenciar modelo propio; superficies orientadas y componentes por serial/función; tolerancias de display vs valores técnicos.
- Modelar bloque, crank, rods, pistones/anillos, culata, cams/rockers, correa/poleas, colectores, pernos y conectores.
- No usar malla como la fuente de verdad física salvo malla de contacto diseñada y validada.
- **Gate:** cada mesh enlaza a PartInstance/Interface, no introduce IDs paralelos ni controles de física desde Three.js.

## S32 — Taller con manipulación y herramientas

- Controles de mover/rotar/seleccionar/alinear, snap a interfaces, torque wrench, dinamómetro, instrumentos.
- Actions se envían como comandos físicos con valores y tolerancias; UI interpreta OperationResult sin revelar estado secreto.
- Mostrar desgaste observable conforme a modelo; si no hay relación visual calibrada, etiquetar genérico o no renderizar.
- **Gate:** reproducir en modo gráfico exactamente un escenario headless `wrong_torque`; logs/sim iguales bajo entorno fijado.

## S33 — Instrumentos 3D + experiencia de aprendizaje opcional

- Consola oscilloscope, compresímetro, leak-down, multímetro, scanner compatible y manual referencial.
- Modo hardcore sin hints; modo escuela con explicaciones **a petición**, no cambios de física.
- Guardar procedimientos y anotaciones del jugador, no soluciones predeterminadas.
- **Gate:** la respuesta de instrumentos corresponde a muestras reales del core + error instrumental versionado.

## S34 — Tuning NA (aspiración natural)

- Cambios de compresión, cams, intake, exhaust, ECU y combustible.
- Solo declarar efecto causal predictivo cuando flujos, curvas y propiedades sean suficientemente modelados/calibrados.
- Desarrollar mapas VE/flow y validación independiente por modificación; reportar extrapolación.
- **Gate:** no existe slider `+HP`; condiciones y límites explícitos. Cambiar una pieza sin datos no debe producir “mejora realista” artificial.

## S35 — Turbo e intercooler

- Modelo turbina/compresor basado en mapas, gases de escape, inercia rotor, eficiencia, presión absoluta, pérdidas e intercooler; mapas y límites Garrett [R15].
- Boost target es objetivo del control, no garantía de aire/HP. Efectos de temperatura de admisión, relación de compresión, combustible, backpressure, MAP, inyectores, alimentación y timing.
- Validación en mapas de operación y datos medidos cuando existan; sin auto-detonación arbitraria.
- **Gate:** reproducir surge/choke/retardo dentro del dominio del mapa o marcarlos no soportados; no inferir 300 hp por `boost psi`.

## S36 — Daño profundo, mecanizado y reconstrucción

- Modelo avanzado de hone, mecanizado, sobremedida, cilindros rayados/ovalidad, cojinetes, deformación térmica/plástica y fatiga.
- Integrar referencia FE/CFD offline **solo** en modos verificables y con geometrías/datos de materiales licenciados.
- Historial de motor usado, procedencia/desgaste de piezas, reparaciones, rutas de diagnóstico.
- **Gate:** los fallos graves se explican con mecanismo y nivel de confianza; un modelo no soportado debe permanecer declarado.

## S37 — Segundo motor/plataforma

- Antes de B18/4G63T, probar una segunda variante dentro de familia D (o motor de geometría distinta) sin duplicar el core.
- Demostrar que arquitectura es por datos/puertos y no “ifs para D16Y7”.
- Hacer inventario de problemas específicos (VTEC, DOHC, turbo, DIS/COP, distintas ECU y disparos) y abrir ADR nuevos.
- **Gate:** cada elemento compartido se reutiliza; motor nuevo tiene paquete propio de datos y validación, no copia cosmetizada.

---

# 17. Escenarios base y tests de aceptación rastreables

**Regla:** cada caso contiene `initial_engine_snapshot`, `parameter_pack`, `operation_log`, `environment`, `measurements`, `expected_invariants`, `validation_source` y `uncertainty`. Los síntomas enumerados son **hipótesis físicas**; cuando no haya submodelo completo se marcan `PENDING`.

| ID | Caso | Variable alterada (no resultado) | Evidencia física exigida | Instrumentos | Gate / afirmación permitida |
|---|---|---|---|---|---|
| T01 | bloque desnudo | ausencia crank y pistones | estructura y conexiones incompletas | inspección | no simular combustión en cámara inexistente |
| T02 | crank instalado sin rods | topología torque/inercia | transmisión solo por joints presentes | giro manual | no fingir pistones funcionando |
| T03 | crank correcto + cilindro cerrado | desplazamiento/volumen | p–V compresión ideal | presión virtual analítica | cerrar balance y convergencia |
| T04 | timing correcto | cam/crank 2:1 | lift/valves por ciclo | dial/timing marks | eventos por geometría |
| T05 | correa desplazada | fase cam | valve events y potencial colisión | indicadores/compresión | daño solo si contacto validado |
| T06 | cable bujía invertido | conexión salida bobina→cilindro | spark llega cilindro distinto | scope/ignition scope | no cambiar secuencia mecánica |
| T07 | gasolina ausente | presión/masa fuel | inyectores entregan poca/nula masa | gauge/scan | no-arranque si balance no sostiene combustión |
| T08 | batería descargada | Voc/R_internal | rpm arranque, voltaje, dwell | multímetro/scope | no motor a rpm prefijadas |
| T09 | CKP abierto | cable resistencia→∞ | señal/sync ECU alterados | scope/DTC | DTC solo si monitor configura |
| T10 | CKP intermitente | contacto/ruido | sync loss/actuator timing variable | scope | semilla fija y waveform medible |
| T11 | MAP bias | función sensor | mezcla comandada difiere de carga real | multímetro/scan | presión física no cambia directamente |
| T12 | ECT fría falsa | lectura sensor | estrategia ECU cambia, coolant físico igual | scan vs termómetro | cadena de software/actuator |
| T13 | anillo con gap alterado | fuga geométrica | blow-by y compresión dependientes Δp | leak-down/compresión | no aplicar `-HP` directo |
| T14 | rayado del cilindro | perfil bore | flujo de fuga/desgaste condicionado | bore gauge/compresión | solo si ley param. validada |
| T15 | perno culata subtorque | precarga | sello local y fuga condicional | torque log/compresión | sin fuga automática |
| T16 | junta ausente | seal geometry/material | compresión y flujos por contacto | leak-down/coolant | sin penalización ad hoc |
| T17 | refrigerante bajo | masa coolant/flujo | transferencia de calor afectada | temp gauge | temps por balances |
| T18 | presión aceite baja | bomba/holgura/lubricante | presión local/film/fricción | manómetro | sin rotura inmediata obligada |
| T19 | limitación de admisión | conductancia throttle/filter | menor masa admitida en condiciones controladas | MAP/flow/dyno | no tabla hp arbitraria |
| T20 | fricción extra | torque resisting | ω/energía por balance | dyno/load | sumatoria torque consistente |
| T21 | anillo fuga + MAP bias | dos cambios independientes | mezcla real + menor trabajo por cilindro | scope/gauge/leak-down | interacción sin selector de “fault winner” |
| T22 | junta marginal + alta carga | precarga y presión | evolución con Δp/T y tiempo | leak-down, temps | resultado condicionado, no scripted |
| T23 | aceite frío vs caliente | viscosidad/temperatura | cambios caudal/fricción | presión/temps | no sistema de aceite booleano |
| T24 | desgaste tras múltiples ciclos | slow-state+historial | divergencia reproducible de condición | inspección | no desgaste dependiente de FPS |
| T25 | dyno sweep stock | carga/ω bajo protocolo | par → potencia; IMEP/BMEP auditables | dyno | no uso circular de hp de catálogo |
| T26 | crash/replay | replay físico | traza y estados equivalentes | CLI/checksums | recuperación verificable |
| T27 | integración cross-target | mismo manifest en 2 targets | tolerancias y evento estables | scripts de comparación | no prometer bitwise universal |
| T28 | sensores desconectados varios | circuitos reales | DTC/actuadores según ECU | scope/scan | sin hints ni códigos programados a fault labels |
| T29 | cambio cam sin flow data | perfil modificado con parámetros faltantes | **error de modelo NO VALIDABLE** | log científico | negar predicción de potencia “fiable” |
| T30 | knock fuera dominio | MAP/T/fuel sin calibración | `UNSUPPORTED/EXTRAPOLATED`, no rotura adivinada | reporte limitaciones | honestidad cuantitativa |

**Suite de consistencia** para cada caso: balance de masa por fluido, energía/trabajo, inventario de partes/conexiones, fase mecánica, integridad de unidades, error numérico, provenance y rangos del solver. Un caso puede pasar físicamente pero no validarse experimentalmente; ambas etiquetas tienen que coexistir.

---

# 18. Herramientas para adquisición de datos y campaña física D16Y7

## 18.1 Documentación a recopilar antes del modelo predictivo

- Manual OEM de variante exacta con página identificada; boletines/fe de erratas si existen.
- Despiece y lista de piezas por número de parte; cambios por años/fabricación.
- Curvas de alzada de cam, valve timing, dimensiones de rod/piston/deck/chamber, valve reliefs.
- Parámetros y wiring de CKP/TDC/CYP, condiciones y patrón en osciloscopio; documentación de ECU OEM.
- Mapas/curvas de throttle/port/exhaust o bancos de flow con método y error.
- Presión de rail/inyector: dead time vs V, flujo vs Δp, modelo de atomización suficiente.
- Viscosidad de lubricante, mapas de bomba, alivio, volúmenes; sistema térmico y geometría circuitos.
- Masa/inercia de cigüeñal/volante/rod/piston, fricción al arranque y carga.
- Ensayos: torque/potencia estabilizados, respuesta RPM bajo escalón, MAP, AFR/lambda, presiones, temperaturas, consumo, CR test y leak-down.
- Si es posible, presión instantánea de cilindro/ángulo y condiciones; **gran salto** de identificabilidad para el burn model.

## 18.2 Procedencia y calidad de medición

En cada captura física: tipo/modelo de motor, número de serie anonimizado, kilometraje/estado, modificaciones, altitud/presión/temperatura/humedad, combustible, calibración de sensores, incertidumbre/resolución, tasa de muestreo, rig/load, protocolo, número de repeticiones, errores/anomalías y fotos permitidas. Distinguir **datos de fábrica** de motor usado o preparado.

**Estrategia si no hay acceso a un D16 real:** avanzar en geometría, conservación, electro-scheduler genérico y casos analíticos; mantener `REAL-D16-VALIDATION=BLOCKED`. No usar una curva sintética procedente del propio solver como evidencia externa. Modelos de terceros sin datos crudos pueden usarse como checks de plausibilidad, no validación independiente.

## 18.3 Pruebas de incertidumbre

- Parámetros principales: métodos OAT/local sensitivity, Latin hypercube/Sobol (si justifica complejidad) y rangos físicamente razonables.
- Separar **aleatoria** (dispersión entre motores y medidas) y **epistémica** (desconocimiento del modelo y parámetros).
- Reportar perfiles de error por estado operativo, no promedio único.
- Priorización de nuevos datos según sensibilidad + incertidumbre + relevancia de jugador.
- Fenómenos altamente dependientes de geometría/material (rupturas) pueden permanecer `UNSUPPORTED` durante varias releases.

---

# 19. Riesgos, costos reales y gestión de alcance

## 19.1 Riesgos críticos

| Riesgo | Impacto | Mitigación / gate |
|---|---|---|
| Datos OEM incompletos | Simulación calibrada a supuestos | S01 identifica faltantes y limita predicción; pruebas independientes |
| Falsa fidelidad | Confianza en curvas incorrectas | ModelCards + reportes + explícito `EXTRAPOLATED` |
| Modelo complejo sin datos | Overfitting/indeterminación | Modelo L1 pequeño primero; aumentar fidelidad con evidencia |
| Solvers inestables | Motores que ganan/consumen energía de la nada | conserv. + convergencia + error handling en CI |
| Integración muy temprana de 3D | Interfaces rígidas e infinidad de hacks | core headless como autoridad |
| Dependencias GPL/copyright | Bloqueos futuros al distribuir | inventario legal antes de reutilizar código/activos |
| Agentes generando números inventados | Deuda científica camuflada | provenance mandatory y “no datos → no conclusión” |
| Acoplamientos excesivos | Desfase de física, performance pobre | solver acoplado/adaptativo; profiling y V&V |
| Fabricar “emergencia” con scripts | Experiencia falsa e inconsistente | fault = estado físico, síntomas = ecuaciones |
| Explosión de escenarios combinatorios | pruebas insuficientes | propiedades + metamórficos + pares adversariales + seeds |
| Dificultad injusta | juego opaco sin instrumentos | instrumentación realista, manual/documentación, logs de jugador |
| Motor viejo desconocido | inferencias arbitrarias de desgaste | parámetros de condición con distribuciones y credibilidad |
| Mezclar versiones Honda | wiring/sensores erróneos | `VariantId`, catálogo por año/ECU y test de compatibilidad |
| Rendimiento realtime | sacrificio oculto de fidelidad | profiles certificados + benchmark + modo offline |

## 19.2 Orden de inversión recomendado

1. **Investigación, fuentes, tablas y diseño de tests.** Coste económico bajo, impacto enorme.
2. **Core reproducible + física verificable** sin 3D. Invierte tiempo en modelar ecuaciones/validación.
3. **Instrumentos/diagnóstico y datos reales**. Comprar/medir/acceder a un D16 puede valer más que assets premium.
4. **CAD e interfaz** tras APIs estables.
5. **Tuning profundo/CFD/FE/turbo** únicamente con datos que soporten predicción.

No se fijan plazos totales como hechos: son un programa de investigación posiblemente **multianual**, especialmente trabajando con pocos ingenieros y agentes. Un agente acelera implementación/documentación, **no inventa parámetros de materiales ni sustituye un dinamómetro**.

## 19.3 Qué NO hacer en la primera base

- No integrar un juego de conducción, garage económico, marketplace o multijugador.
- No crear 4G63T/B18/VTEC mientras el D16 no pase los gates.
- No obsesionarse con una representación CAD perfecta antes de implementar `PartInstance`/`Joint`.
- No “portar GT-Power” ni escribir un CFD desde cero sin necesidad/equipo.
- No copiar la ECU Speeduino y renombrarla “Honda OEM”.
- No programar una regla `if no gasket: engine_destroyed` ni `if high_boost: +X hp`.
- No afirmar saber exactamente cuándo se dobla una válvula o rompe una biela sin rigidez, geometría y materiales.
- No persistir output de agentes como fuente técnica sin revisión.
- No prometer cross-platform bit-perfect a toda costa.

---

# 20. Orden inmediato recomendado / paquete de handoff al primer agente

**Primer objetivo REAL: S00 y S01.** Antes de física hay que seleccionar la variante exacta y documentar qué sabemos y qué no.

### Brief de ejecución S00

> Trabaja **solo** en S00. Inicializa el repositorio con estructura documental mínima, ADRs, Definition of Done, límites científicos, archivo de fuentes y trazabilidad, CI de documentación y templates de sprints/progreso/handoff. Incluye en el charter el JTBD de §0.1 y referencia **sin reescribirlos** los dos archivos de investigación. No implementes ningún solver ni UI. No inventes datos del D16Y7. Prepara informe: decisiones, archivos, pruebas ejecutadas, deuda y gate. Si encuentras fuentes discrepantes, márcalas como `REQUIRES_VERIFICATION`.

### Brief de ejecución S01 (solo después de aprobar S00)

> Trabaja **solo** en S01. Fija una única configuración D16Y7 (año/mercado/modelo/transmisión/ECU), audita documentación OEM por sección y crea un diccionario tipado de geometría, tolerancias, sensores, torque, señales, piezas y requerimientos del núcleo. Cada cifra debe tener página/edición/estado/incertidumbre. No completes variables desconocidas con valores de motores parecidos. Entrega matriz `CONFIRMED / DISPUTED / UNKNOWN` y el protocolo para adquirir lo faltante. No desarrolles la física hasta pasar gate S01.

### Documentación mínima que debe sobrevivir reinicios de agentes

- `docs/SPRINTS.md`: dependencias, estado y acceptance gates.
- `docs/PROGRESS.md`: log objetivo de cambios ejecutados, tests y referencias de commits.
- `docs/HANDOFF.md`: instrucciones y estado científico/arquitectónico *actual*, fuentes principales, blockers, cómo reproducir.
- `docs/adr/*.md`: decisiones y motivación, cambios versionados.
- `docs/model_assumptions.md`: las aproximaciones, vigencia y condiciones que las invalidan.
- `docs/limitations.md`: todas las áreas `UNSUPPORTED` y dominios de extrapolación.
- `data/provenance_registry.*`: catálogo de fuentes y por-valor.
- `tests/scenarios/`: testcases reproducibles, solo cuando haya runner.

**Pregunta científica que gobierna cada PR:** “¿Podemos explicar y verificar con datos/ecuaciones por qué cambió ese resultado al modificar esa pieza?” Si la respuesta es «porque programamos que cambiara», el PR **no cumple** la visión.

---

# 21. Fuentes, clasificación y links de investigación (verificados online 2026-10-09)

> Los vínculos a manuales y repositorios sirven como **puntos de partida para lectura**, no certifican todas las cifras ni conceden permiso para redistribuir contenido. Estándares profesionales pueden estar bajo pago. No usar fuentes secundarias para sustituir las páginas OEM exactas. El modelo debe almacenar sección y página al transcribir parámetros reales.

### Fabricante / manuales de vehículo

- **[R01] Honda Civic 1996–2000 Service Manual**, índice de múltiples secciones y variantes D16Y7, reproducciones accesibles: [ManualsLib](https://www.manualslib.com/manual/464698/Honda-Civic.html); [Internet Archive — ficha bibliográfica Honda](https://archive.org/details/HondaCivic19962000). Manual de fabricante; verificar sección/página antes de transcribir; no redistribuir el PDF como asset.
- **[R02] Honda PGM-FI, CKP/TDC/CYP y CKF, manual 1996–2000**, en [ManualsLib, páginas del índice 374 y 442–457](https://www.manualslib.com/manual/464698/Honda-Civic.html). Importante **variación por año y transmisión**.

### Proyectos software y modelos

- **[R03] AngeTheGreat, Engine-Sim**, [repositorio](https://github.com/ange-yaghi/engine-sim). Admite que no es simulador científico para ingeniería/tuning.
- **[R03a] Engine-Sim LICENSE**, [MIT](https://github.com/ange-yaghi/engine-sim/blob/master/LICENSE).
- **[R04] Speeduino**, [repositorio](https://github.com/speeduino/speeduino), referencia de arquitectura ECU, mapas y decodificadores.
- **[R04a] Speeduino licencia**, [repositorio especifica GPL-2.0](https://github.com/speeduino/speeduino); su fuente `speeduino.ino` también explicita GPL v2 o posterior según archivo.
- **[R05] Ardu-Stim**, [repositorio](https://github.com/speeduino/Ardu-Stim), referencia para patrones de trigger/pruebas de ECU.
- **[R06] Cantera: ejemplo ICE**, [documentación oficial](https://cantera.org/stable/examples/python/reactors/ic_engine.html). Motor/ciclo de ejemplo es tipo diésel con supuestos ilustrativos; útil para metodología, no una validación del D16 gasolina.
- **[R06a] Cantera licencia**, [guía oficial de contribución](https://github.com/Cantera/cantera/blob/main/CONTRIBUTING.md) identifica BSD-3-Clause.
- **[R07] OpenWAM, gas dynamics CMT-UPV**, [GitHub del proyecto de motores](https://github.com/CMT-UPV/OpenWAM), [sitio UPV](https://openwam.webs.upv.es/docs/), [release 2.2.0 de 2016](https://github.com/CMT-UPV/OpenWAM/releases). **No confundir con World Action Model de IA homónimo**. Licencia por revisar en árbol/paquete concreto.
- **[R08] Haltech Trigger System**, [documentación oficial](https://support.haltech.com/portal/en/kb/articles/trigger). Principios generales de sync/home y requisitos ECU, no configuración OEM Honda.

### Credibilidad de modelos y metodología de ensayos

- **[R09] ASME VVUQ**, [Verification, Validation and Uncertainty Quantification](https://www.asme.org/codes-standards/publications-information/verification-validation-uncertainty), estándares V&V 10/20 y VVUQ 1. Orientación de verificación frente a realidad y cuantificación de incertidumbre.
- **[R10] NASA-STD-7009B**, [Standard for Models and Simulations](https://standards.nasa.gov/standard/NASA/NASA-STD-7009), marco de credibilidad para modelos de simulación.
- **[R11] ARP**, [General Torque Recommendations](https://www.arp-bolts.com/tech/torque-values), relación torque/apriete bajo condiciones indicadas. **No usar como torque OEM Honda.**
- **[R12] Fel-Pro**, [Understanding the Real Cause of Head Bolt Failures](https://www.felpro.com/technical/tecblogs/understanding-the-real-cause-of-head-bolt-failures.html), importancia de secuencia/carga desigual y gasket sealing.
- **[R13] Tauri 2**, [What is Tauri?](https://tauri.app/start/), [Architecture](https://tauri.app/concept/architecture/); backend nativo Rust + frontend WebView.
- **[R14] NASA Glenn**, [Engine Thermodynamic Analysis](https://www1.grc.nasa.gov/beginners-guide-to-aeronautics/engine-thermodynamic-analysis/); diferencia de ciclo Otto ideal vs real.
- **[R15] Garrett Motion**, [How a Turbo Works—Compressor Maps](https://www.garrettmotion.com/knowledge-center-category/oem/expert/) y [Selection / pressure ratio](https://www.garrettmotion.com/knowledge-center-category/racing-and-performance/how-to-select-a-turbo-part-2-understanding-calculations-to-turbo-any-engine/). Para expansión turbo.
- **[R16] NASA Glenn**, [Otto Cycle](https://www1.grc.nasa.gov/beginners-guide-to-aeronautics/internal-combustion-engine-otto-cycle/), diferencias fundamentales entre ideal y real.
- **[R17] SAE J1349_202511**, [Engine Power Test Code](https://saemobilus.sae.org/standards/j1349_202511-engine-power-test-code-spark-ignition-compression-ignition-installed-net-power-torque-rating). Norma revisada en noviembre de 2025: método repetible y condiciones de potencia neta; **consultar texto completo/licencia** si se implementan sus correcciones.
- **[R18] JSAE, Zero-dimensional Knock Modeling and the Livengood–Wu Integral**, [resumen](https://www.jstage.jst.go.jp/article/jsaeronbun/48/1/48_20174051/_article/-char/en), limitaciones en predicción knock con química simplificada; [Fuel, incertidumbre de timing de autoencendido](https://www.sciencedirect.com/science/article/pii/S0016236120320214).
- **[R19] Estudio abierto sobre fault diagnosis en motor turbo SI (Ng, Frisk, Krysander, Eriksson, 2020)**, [arXiv](https://arxiv.org/abs/2002.03201), ejemplo académico de modelo mean-value y escenarios con fallas; no asumir que proporciona motor Honda.
- **[R20] NASA Software Engineering Handbook**, [SWE-070 Models, Simulations, Tools](https://swehb.nasa.gov/spaces/7150/pages/16449960/SWE-070%2B-%2BModels%2BSimulations%2BTools); importancia de precisión numérica, sensibilidad, verificación y validación.

### Fuentes que requieren búsqueda complementaria en siguientes sprints

1. Especificaciones exactas y tabla de torque/compresión/clearance de **la variante congelada** con paginación verificable.
2. Honda PGM-FI: pinouts y diagramas de secuencia/decoders, patterns medidos en osciloscopio, sensores según año y ECU.
3. Manuales técnicos de MAHLE/Motorservice/Fel-Pro sobre segmentos, honing, deformación térmica/estanqueidad.
4. Publicaciones revisadas de Wiebe, Woschni/Hohenberg/Annand, modelo de flujo por válvula, gas-dynamics y FMEP; verificar rango antes de adoptar coeficientes.
5. Ensayos de torque y presión de cilindro de D16Y7 stock con condiciones/certificados.
6. Datos de mapa de bomba aceite/agua, termostato/radiador, sensor ECT/IAT y curvas de inyectores.
7. Bases de datos materiales por componente real, incertidumbres y vida de fatiga.

---

# 22. Registro final de decisiones pendientes para iniciar

| Prioridad | Cuestión | Estado / bloqueo |
|---|---|---|
| P0 | Año/mercado/transmisión/ECU exacta del D16Y7 | **Pendiente S01** |
| P0 | Derechos y fuente primaria con páginas de manual OEM | **Pendiente S01** |
| P0 | Esquemas data/provenance y test de validación | **S00–S03** |
| P0 | Política de serialización + replay + determinismo | **S02–S05** |
| P0 | Modelo de cilindro, balances, scheduler eventos | **S06–S16** |
| P1 | CKP/TDC/CYP real por año | **S17; adquisición pendiente** |
| P1 | Datos experimentales stock para calibrar | **S25; bloqueo externo** |
| P1 | Ley de junta torque→precarga→conductancia | **S08/S23; coeficientes pendientes** |
| P1 | Aros/liner leakage y wear law | **S23/S26; parámetros pendientes** |
| P1 | Auditar licencia del OpenWAM de motores | **Antes de integrar cualquier código** |
| P2 | WASM performance parity | **Tras core + benchmarks** |
| P2 | CAD completo y calidad visual | **Después de foundation audit** |
| P2 | Turbo, knock detallado, fallas graves predictivas | **Fase D + validación adicional** |

**Resumen ejecutivo para el siguiente agente/ingeniero:** 

- No desarrollar una demo con sliders de RPM/HP. **Primero** datos Honda, unidades, pruebas y núcleo reproducible.
- El objetivo **no** es modelar un “resultado correcto” inventado; es construir ecuaciones de conservación y componentes ensamblables cuyos estados generan síntomas observables.
- La física debe permanecer **independiente del renderer**, del frame rate y de la interfaz.
- Empezar en **S00**, cerrar por gates y luego **S01**, jamás adelantarse al motor 3D/turbo mientras falten protocolos de datos.
- La base puede ser amplia y durar docenas de sprints. Hacerla sólida es más importante que acelerar a una build que aparenta realismo.

# 23. ADENDA DE INVESTIGACIÓN — FOROS, EXPERIENCIAS REALES Y DATOS DE TALLER (2026-10-09)

> **Naturaleza de esta sección:** ampliación aditiva de Foundation v0.1. **No sustituye, reordena, elimina ni da por cumplido ningún sprint S00–S37.** Se agrega evidencia que no aparecía suficientemente desarrollada: medidas reales, variantes OEM, fallos tras montaje, electrónica específica, familias de piezas y condiciones de las pruebas. Las fuentes de foros **no** equivalen a calibración certificada. Las páginas del manual/catálogos proporcionan límites y referencias, pero su aplicabilidad debe congelarse por modelo, año, mercado y variante en S01.
>
> **Alcance de la búsqueda:** Honda-Tech (mecánica, machining/assembly, engine management y forced induction), ClubCivic, foros Hondata, discusiones Reddit con aplicación identificada, catálogos OEM comercializados por HondaPartsNow y páginas HTML del manual de servicio; contraste con fabricantes técnicos (King Bearings, Motorservice, Fel-Pro, ARP). D-series.org apareció como fuente potencial, pero algunas páginas no permitieron lectura automática en esta campaña: no citar publicaciones no leídas como si hubieran sido verificadas.

## 23.1. Reglas nuevas de procedencia: la diferencia entre una medición y una opinión

**Nueva clasificación complementaria** — no reemplaza `[F/D/H/P/X]` del documento:

| Etiqueta | Qué demuestra | Puede convertirse en fixture numérico directamente |
|---|---|---|
| `OEM-TEXT` | Procedimiento o rango aplicable en reproducción identificada de manual Honda | **Sí, condicionado** a año/variante/edición y método; confirmar en S01 |
| `OEM-EPC` | Existencia, número y aplicación catalogada de una pieza | Solo **identidad/compatibilidad**, no resistencia, capacidad o dinámica |
| `MFR-TECH` | Mecanismo físico o método declarado por fabricante técnico | Sí como **ley o restricción general**, no como coeficiente específico Honda sin validación |
| `FORUM-MEASURED` | Alguien declara un valor medido y describe su procedimiento | Como observación de campo, con incertidumbre; **no** como valor nominal |
| `FORUM-CASE` | Síntomas, intervención, diagnóstico reportado, condiciones | Como **caso adversarial o hipótesis de diagnóstico**; no como causalidad demostrada |
| `FORUM-CLAIM` | Consejo, anécdota, cifra sin método o causalidad presunta | Solo como pista para investigar; **no** como parámetro productivo |
| `MISSING-DATA` | Magnitud que requiere banco, despiece, metrología o ciencia | No; abrir tarjeta de adquisición y declarar rango no identificado |

Guardar por cada publicación: URL directa, fecha original del post, autor/identificador (opcional), texto resumido sin reproducción extensa, motor (Y7/Y8/Z6/otro), configuración ECU/transmisión, original/modificado, temperatura, RPM, método/instrumento, unidades, incertidumbre, estado del resultado (resuelto, probable, sin resolver), fotografías/gráficas disponibles y licencia del contenido. **No asumir que un hilo marcado «solved» confirma el mecanismo si no hay comprobación independiente.**

## 23.2. Datos del D16Y7 que ahora sí se pudieron localizar con referencia de taller

Los siguientes **no** son valores improvisados ni sacados exclusivamente de foros. Están publicados en **reproducciones HTML de información de taller**; S01 aún debe confrontarlos con la edición Honda exacta y registrar número de página/documento y variante. No sustituir especificaciones de un motor distinto por conveniencia.

| ID candidato | Dato / condición de medición | Valor localizado | Estado / fuente | Consecuencia para el simulador |
|---|---|---|---|---|
| `compress_nominal_1997_dx` | Compresión de arranque, 250 rpm, acelerador abierto | **1.270 kPa / 184 psi**, nominal | `OEM-TEXT` [CF01] | Fixture de instrumento de compresión, **no** presión instantánea de combustión ni relación geométrica |
| `compress_min_1997_dx` | Mismo ensayo | **930 kPa / 135 psi**, mínimo | `OEM-TEXT` [CF01] | Límite de servicio, nunca umbral que fuerce fallo binario |
| `compress_spread_limit` | Máxima diferencia entre cilindros en el mismo ensayo | **200 kPa / 28 psi** | `OEM-TEXT` [CF01] | Contrastar variación intra-motor; depende del protocolo |
| `oil_p_idle_80C` | Aceite a 80 °C, ralentí | **69 kPa / 10 psi, mínimo** | `OEM-TEXT` [CF02][CF03] | Diferenciar presión en galería, switch y presión de película hidrodinámica |
| `oil_p_3000_80C` | Aceite a 80 °C, 3.000 rpm | **340 kPa / 50 psi, mínimo** | `OEM-TEXT` [CF02][CF03] | Dependencia de rpm, viscosidad y holguras |
| `main_clearance_j1_j5` | Cojinetes principales núms. 1 y 5; nuevos | **0,018–0,036 mm** | `OEM-TEXT` [CF04] | Variación por apoyo, no un clearance global para todos |
| `main_clearance_j2_j4` | Cojinetes principales núms. 2, 3, 4; nuevos | **0,024–0,042 mm** | `OEM-TEXT` [CF04] | Simular asignación individual por bancada/color |
| `main_clearance_service_limit` | Límite de servicio, principales | **0,05 mm** | `OEM-TEXT` [CF04] | No equivale automáticamente a rotura |
| `rod_clearance_new` | Cojinete de biela, estándar nuevo | **0,020–0,038 mm** | `OEM-TEXT` [CF05] | Debe medirse por muñequilla y casquillo específico |
| `rod_clearance_service_limit` | Límite de servicio, biela | **0,05 mm** | `OEM-TEXT` [CF05] | Degradación y tolerancia antes de modelar fatiga |
| `lash_intake_cold` | Holgura de válvula admisión (motor frío) | **0,18–0,22 mm** | `OEM-TEXT` [CF06] | Modifica apertura efectiva/cierre, transferencia de carga y ruido |
| `lash_exhaust_cold` | Holgura de válvula escape (motor frío) | **0,23–0,27 mm** | `OEM-TEXT` [CF06] | Igualmente; diferenciar holgura fría y térmica |
| `fuel_p_reg_vac_off` | Presión de combustible al ralentí, vacío del regulador desconectado | **260–310 kPa / 38–46 psi** | `OEM-TEXT` [CF07] | Modelo debe incluir referencia manométrica/absoluta |
| `fuel_p_reg_vac_on` | Al ralentí, vacío regulador conectado | **200–250 kPa / 28–36 psi** | `OEM-TEXT` [CF07] | No un valor fijo de rail para todos los estados |
| `cam_holder_clearance` | Juego aceite árbol/soporte, nuevo, publicación de Civic DX 1999 | **0,050–0,089 mm** | `OEM-TEXT` [CF08] | Separar cojinete/holgura del levantamiento del perfil de leva |

**Precaución numérica:** presión de compresión *medida por compresímetro* no se obtiene de `CR × atm`. Depende de sellado, fase de cierre admisión, temperatura, rpm, válvula antirretorno y protocolo. Los relatos de foro hablan de 120, 145, 180 y 195 psi sin condiciones comparables. **Validar la predicción del compresímetro como instrumento, no imponer 184 psi a cualquier bloque que arranque.** [CF01][CF09][CF10][CF11][CF12]

**Observación de evidencia cruzada:** un comentario de foro atribuía **34–36 psi con vacío conectado** al Y7, mientras la reproducción del manual admite **28–36 psi**: la cifra de foro cae en ese rango, pero no lo define. Más importante: un post menciona inyectores OEM de **180 cc/min** frente a **240 cc/min** en el Y8 sin parte/norma de presión: **conservar ambos como afirmaciones no confirmadas** hasta medir o hallar especificación de referencia. [CF07][CF13][CF14]

## 23.3. Catálogos de piezas OEM: hallazgo mucho más útil que despieces anónimos

**Aplicación de ejemplo** para S01: Civic **1997 DX, transmisión manual 5MT**, catálogo HondaPartsNow. Los sufijos `KA`, `KR`, `KW` identifican configuraciones comerciales/mercado; **no mezclar** registros de esas vistas en un BOM único sin equivalencias documentadas. El catálogo muestra existencia, cantidades y referencias, **no** la curva física del componente. [CF15][CF16][CF17][CF18][CF19]

| Sistema | N.º de pieza observado | Hallazgo trazable | Entrada de modelado |
|---|---|---|---|
| Culata | `12100-P2A-000` | Head assembly del DX SOHC | Referencia de ensamble jerárquico |
| Junta de culata | `12251-P2J-004` | Junta catalogada para la aplicación concreta | Existencia y compatibilidad, **no** espesor comprimido/calidad térmica |
| Tornillos principales de culata | `90008-PZ1-003` | 10 unidades de bolt-washer 10×120,5 en esa vista | `FastenerInstance` × 10, cada uno con historial de apriete |
| Pistón estándar | `13010-P2E-000` | Juego STD del DX | Familia geométrica compatible; no inventar aleación ni peso |
| Pistones sobremedida | `13020-P2E-000`, `13030-P2E-000` | Opciones +0,25 y +0,50 mm de catálogo | Reacondicionamiento, bore/piston matching |
| Segmentos OEM | `13011-P07-004` (según configuración) | Set RIKEN STD; hay variantes de proveedor/mercado | La marca/proveedor debe ser parte de la entidad |
| Inyectores | `06164-P2A-000` | Fuel injector set, requiere 4 | Identidad sí; caudal/deadtime **no inferibles** del EPC |
| Regulador de combustible | `16740-P2E-A01` | Pressure regulator assembly | Presión base/referencia al múltiple |
| Colector de admisión | `17100-P2F-A00` | Manifold inlet listado | Geometría/volúmenes/caídas de presión todavía pendientes |
| Bomba de aceite | `15100-P7A-013` | Oil pump assembly | Curva Q–ΔP aún no disponible |
| Válvula de alivio de aceite | `15231-PM3-000` | Relief valve y muelle separado `15232-PM3-000` | Estado mecánico de bypass, no una presión inventada |
| Distribuidor TD-80U | `30100-P2E-A02` | Catalogado en una variante 1997 DX | No convertir su identificación en patrón de trigger sin documentarlo |
| Distribuidor TD-98U | `30100-P2F-A01` | **Otra variante** 1997 DX | Prueba real de por qué S01 exige mercado y aplicación |
| Bomba de agua | `19200-P2A-A01` | Water pump catalogada | Curva de caudal/carga pendiente |
| Termostato | `19301-P08-316` / reemplazo `19301-PAA-306` | Variantes/sustituciones catalogadas | Umbral de apertura y curva histerética pendientes |

Fuentes de catálogo, con páginas **específicas** (no home): [CF15] culata, [CF16] pistón/aros, [CF17] admisión/inyector, [CF18] bomba de aceite, [CF19] distribuidores por mercado, [CF20] refrigeración. **No descargar/republicar las imágenes de despiece ni venderlas como activos originales propios sin permiso.** Dibujar geometrías propias o contratar CAD con procedencia comprobada.

**Implicación crítica para el motor de montaje:** el inventario no es “hay una junta y diez tornillos”. Necesitamos instancia de pieza, identificación/serie, referencia, sustituciones válidas, montaje actual, número de ciclos de apriete, medición, procedimiento seguido y daño acumulado. Una pieza de segundo uso no debe generar las mismas condiciones iniciales que una OEM nueva si el modelo permite distinguirlo.

## 23.4. Investigación en foros — metrología del bloque, aros, culata y reconstrucción

Cada fila resume **qué se observó y qué experimento inspira**; ninguna autoriza incorporar un nuevo valor OEM sin S01.

| Caso | Evidencia encontrada | Qué añade a nuestros experimentos | Confianza |
|---|---|---|---|
| `CASE-M01` | Reconstrucción D16Y7: marcas de colores en cojinetes instalados **no coincidían** con códigos de bloque/cigüeñal; se recomienda contrastar holgura por medición [CF21] | Colores como selección nominal; medidas por muñequilla mandan. Detectar mezcla o rectificación previa | `FORUM-CASE`, útil |
| `CASE-M02` | Y7 desmontado reporta **0,003–0,004 in** de juego pistón/camisa, pregunta si es aceptable [CF22] | Simular juego real y consecuencias posibles, **sin** convertir esa cifra en tolerancia permisible | `FORUM-MEASURED` incompleto |
| `CASE-M03` | Reconstrucción advierte medir anillos, cilindro, ovalización, cigüeñal y elegir cojinetes por código/Plastigage [CF23] | Crear instrumentos y orden de comprobación, no solo una receta de piezas | `FORUM-CASE`, coincide con taller |
| `CASE-M04` | Tras junta de culata, tornillos no alcanzan torque y siguen girando; posible rosca dañada [CF24] | Rosca/débil material/anclaje: torque aplicado ≠ precarga lograda | `FORUM-CASE`; causalidad no confirmada |
| `CASE-M05` | Tras sustitución junta, autor atribuye primer fracaso a **aceite acumulado en agujeros de pernos** [CF25] | Falla potencial por líquidos/volumen atrapado al apretar; **requiere** validación del mecanismo y diseño de agujero | `FORUM-CLAIM`, no probar como certeza |
| `CASE-M06` | Caso de culata rectificada y pérdida de refrigerante con compresiones aún relativamente altas [CF26] | Una prueba de compresión razonable **no excluye** fuga al circuito de refrigeración | `FORUM-CASE` |
| `CASE-M07` | Pérdida de refrigerante, P0301, sospecha de junta o válvula; sin diagnóstico confirmado [CF27] | Modelo de diagnóstico con hipótesis concurrentes, no síntoma único | `FORUM-CASE` sin resolver |
| `CASE-M08` | Leak-down de un cilindro: participante **escuchó fuga por escape**; recomendación de comprobar asiento/holgura de válvula [CF28] | Leak-down debe ubicar vía efectiva de fuga (escape, admisión, cárter, refrigerante) | `FORUM-MEASURED` cualitativo |
| `CASE-M09` | Consumo de aceite + compresiones bajas en Y7 usado (110/95/90/100 psi reportados) [CF29] | Pérdida de compresión y consumo pueden coexistir; no asumir que coinciden siempre | `FORUM-MEASURED` no estandarizado |
| `CASE-M10` | Usuarios reportan tras pruebas 148/150/153/153 psi y 195/187/192/197 psi en Y7 [CF09][CF10] | Variabilidad real + condiciones insuficientes; diseñar dos datasets de campo distintos | `FORUM-MEASURED` |
| `CASE-M11` | Prueba hecha sin WOT devuelve ~120 psi uniformes; otro usuario detecta efecto del protocolo [CF11] | Un **error del usuario con el instrumento** puede parecer motor defectuoso | `FORUM-CASE` |
| `CASE-M12` | Reclamo de consumo de aceite relacionado con anillos, paredes y sello de vástago; debate sobre diagnóstico que no se resuelve solo con compresión [CF30] | No equiparar consumo de aceite a baja presión de compresión; modelar retención/arrastre de aceite | `FORUM-CASE` |
| `CASE-M13` | Tras cambio de culata Z6/Y8 se discuten incompatibilidades de tornillos/correa/ECU y procedimientos [CF31] | El montaje “mini-me” futuro exige grafo de compatibilidad exacto de piezas, no flag VTEC | `FORUM-CASE`; no es base OEM |

**Referencia física necesaria para estos casos:** Motorservice documenta mecanismos de **mal honing, microrebabas, desgaste de anillos y consumo de aceite**; MAHLE cubre posición de aros, juego axial/radial y verificación por ranura. King Bearings detalla relación de holgura, geometría, viscosidad y película lubricante; Fel-Pro explica influencia de planitud/rugosidad sobre junta. Son **fundamentos generales**, no identificación automática de daños en un Y7 [CF32][CF33][CF34][CF35].

**Nueva obligación del modelo:** representar **por separado** `bore_diameter(z,theta)` o una reducción (ovalidad, conicidad, rayas segmentadas), `piston_skirt_clearance`, `ring_end_gap`, `ring_groove_clearance`, `ring_tension`, `ring_sticking`, `ring_seal_effectiveness`, `oil_control_capacity`, `valve_seat_leakage`, `valve_stem_seal_leakage` y `head_gasket_path_conductance`. Se permite simplificar y no resolver microcontactos, pero nunca colapsar todo en `engine_health_pct`.

## 23.5. Investigación en foros — distribución, encendido, sensores, ECU y diagnóstico

| Caso | Observación | Especificación que revela | Clase |
|---|---|---|---|
| `CASE-E01` | **1996 D16Y7** sin arranque, distribuidor con CKP/TDC/CYP probados cerca de 331–337 Ω; el propietario citaba banda 350–700 Ω de su manual, sin resolución definitiva [CF36] | Sensores inductivos necesitan modelo R/L/amplitud vs velocidad; resistencia DC **no** prueba señal útil | `FORUM-MEASURED` |
| `CASE-E02` | Hilo ClubCivic ubica CKP/TDC/CYP en distribuidor para su aplicación y reporta ~340–345 Ω, con incertidumbre sobre la causa [CF37] | Separar hardware físico de diagnóstico; no declarar sensor roto por pocos ohmios fuera de un rango que requiere verificación | `FORUM-MEASURED` |
| `CASE-E03` | Un 1996 Y7 entrega una sola chispa inicial; autor revisa pulso de inyector, igniter y señal TDC hacia ECU [CF38] | Montar un scheduler de chispa controlado por referencia de posición, no solo «spark present» booleano | `FORUM-CASE` |
| `CASE-E04` | En otro no-start, tras cambiar distribuidor, el autor reporta que el vehículo arrancó [CF39] | Registrar **intervención + desenlace**, pero no inferir cuál subcomponente era el culpable | `FORUM-CASE` con resultado |
| `CASE-E05` | Motor con marcas de distribución de árbol/cigüeñal que no parecen cuadrar; conversación distingue marca PMS de marcas de encendido y un diente desplazado [CF40] | Dos ángulos distintos: fase de correa **mecánica** y avance de **encendido** | `FORUM-CASE` |
| `CASE-E06` | Discusiones D16Y7 mencionan encendido base **12° ±2° BTDC**, lámpara y procedimiento de puenteo del conector de servicio [CF41][CF42] | El valor necesita confirmación OEM; la ECU en modo ajuste no es igual a su avance dinámico | `FORUM-CLAIM` corroborado por manual citado por usuarios |
| `CASE-E07` | 1999 D16Y7: ralentí caliente oscila/surge; intervención en IACV 3 cables, resultados diferentes entre dos piezas [CF43] | IACV como **actuador físico**, errores eléctricos/obstrucción/retardo, no simple ajuste de rpm | `FORUM-CASE` |
| `CASE-E08` | Una incidencia P1509 se atribuye a **mala conexión en ECU** y quedó resuelta tras intervenir el conector [CF44] | DTC idéntico puede surgir de actuador, cable, pin, alimentación o comando ECU | `FORUM-CASE` con desenlace autodeclarado |
| `CASE-E09` | Fuga de vacío y cambio de colector Y7/Y8 generan ralentí alto, con fallas potencialmente múltiples [CF45][CF46] | Caudal de aire no medido/derivado + control inestable IACV; no atribuir siempre «IACV malo» | `FORUM-CASE` |
| `CASE-E10` | En foro técnico se explica ralentí como suma de bypass de mariposa, tornillo de base e IACV; advertencia sobre estrategias **2 vs 3 cables** [CF47] | Configurar hardware antes de escribir la ley de control | `FORUM-CLAIM` con ingeniería plausible |
| `CASE-E11` | P2E asociado a Y7; conversiones OBD2a/OBD2b y manual/automático tienen distinta compatibilidad, sufijos documentados informalmente [CF48][CF49][CF50] | `EcuVariant`: número completo, pinout, firmware, mercado, transmisión, arnés, actuadores, OBD | `FORUM-CLAIM` parcialmente respaldado por referencias cruzadas |
| `CASE-E12` | Casos de conversión a P06/P28 y tuning aftermarket Hondata [CF51][CF52] | Debe existir **otra ECU** separada del controlador P2E stock; nunca atribuir mapas aftermarket a OEM | `FORUM-CASE` |

**Dos correcciones conceptuales IMPORTANTES:**

1. **CKP/TDC/CYP/CKF no son sinónimos**. Una referencia de posición no ordena directamente la chispa; la ECU recibe flancos/patrones, estima fase/velocidad y programa eventos. Revisar colocación y presencia de cada sensor **según variante concreta**. No atribuir al Y7 OEM sensores o estrategias extra por analogía con motores modernos.
2. **El orden de cables de bujía** y el **orden de explosión** no son el mismo fenómeno que instalar mal una biela. Conectar cilindros equivocados puede causar misfire/backfire/no-start; una rotura de biela requiere **otro mecanismo físico** (golpe, falta de lubricación, sobrecarga, contacto, fatiga, etc.). El juego no debe inventar bielas destruidas por un simple orden de cables incorrecto.

Para entrada de señal crear `TriggerWheel`, `Pickup` (ubicación real, forma de onda, R, L, gap, ruido térmico), `Wire/Connector` (resistencia/contacto/fallo intermitente), `Conditioner` (umbral/histéresis), `Decoder` (RPM/phase/confianza), `IgnitionScheduler`, `InjectorScheduler`, `DTCMonitor`. Cada submódulo expone entradas/salidas y observabilidad con multímetro/osciloscopio sin incluir una etiqueta mágica de fallo.

## 23.6. Investigación en foros — combustible, dyno y turbo (útiles **después** de stock)

| ID | Declaración encontrada | Información que S25/S34/S35 podrían aprovechar | Lo que **no** autoriza |
|---|---|---|---|
| `CASE-T01` | D16Y7 con kit TD-04H a **6 psi**, colector Y8, ECU P06/OBD1 y 340 cc/min: primero ~142 whp con mezcla pobre declarada; tras ajuste ~163 whp y ~158 lb-ft; inyector reportado **100 % duty** [CF53] | Presión de sobrealimentación ≠ resultado fijo; influencia de mezcla y límites de duty | No es curva base D16Y7 stock ni ensayo controlado |
| `CASE-T02` | Y7 con pistones Vitara, bielas Eagle, turbo T3, ECU P06: reporta 235 whp@15 psi y 266 whp@17 psi en marcha 3; ~253 whp en 4ª; a 20 psi no aumentó potencia [CF54] | Necesidad de efficiency map, pérdidas, temperatura, escape y **protocolo de dyno/marcha** | No demuestra por sí solo causa del plateau ni límite de culata |
| `CASE-T03` | Y7 50k millas, 8 psi, ~167 whp y ~174 lb-ft, autor comenta detonación pese a AFR reportado ~11,5–12 [CF55] | AFR nominal **no garantiza** ausencia de knock; faltan presión/cámara/temp/avance/octanaje | No inferir umbral de knock exacto |
| `CASE-T04` | Distintos usuarios discuten potencia tolerable en bielas OEM; casos de Y7 cerca de 200 whp sobreviven y otros fallan con overboost [CF56][CF57] | Las roturas no deben usar un `if whp > 200`; falla es dependiente de carga, historial y dispersión | No derivar vida útil/curva S–N de relatos |
| `CASE-T05` | Colectores Y8/Z6, cabezas cambiadas, diferentes inyectores y ECU son comunes en publicaciones «Y7» [CF31][CF53][CF54] | Tipar **configuración completa**, no solamente familia del bloque | No mezclar swaps con referencia OEM |
| `CASE-T06` | D16Y7 atmosférico con modificaciones mínimas anuncia **117 whp**, y el propio hilo pregunta por ECU/otros cambios [CF58] | Datos de modificación observados y variables no registradas | No afirmar stock 117 whp |
| `CASE-T07` | Foro pregunta por **curva completa OEM D16Y7** y no aporta una tabla validada; otro usuario estima ~80 whp [CF59] | **Laguna confirmada**: pico de catálogo no calibra forma de curva | No reemplazar con una línea interpolada sin etiqueta |
| `CASE-T08` | Perfil de **leva regrind** Crower reportado como duración/lift para un Y7 [CF60] | Nomenclatura de cam lift/duración/medición y necesidad de perfiles reales | No convertirlo en perfil **stock**, ni interpolar lobes sin especificación de método |

**Modelo de admisión/combustible recomendado al registrar estas pruebas:** presión atmosférica → filtro → mariposa → plenum → runner → válvula → masa atrapada; combustible: tensión batería → bomba → rail/regulador (diferencia de presión) → características de inyector (`Q(ΔP)`, tiempo muerto, tensión, temperatura) → wall wetting → masa vaporizada → lambda → combustión. Boost requiere mapa compresor, contrapresión/turbina, wastegate, eficiencia térmica e intercooler; **presión de manifold sola no determina potencia**. No meter estos cambios dentro del dataset stock.

## 23.7. Conflictos concretos encontrados y política de resolución

| Conflicto | Evidencia | Resolución correcta |
|---|---|---|
| `184 psi` frente a `135 psi` | Hilos citan ambos [CF09][CF10]; manual especifica **nominal vs mínimo** [CF01] | Registrar **dos campos distintos**, no promediar |
| Compresión aparentemente buena + posible junta mala | Hilo con pérdidas de refrigerante y compresión relativamente alta [CF26] | Diagnostic likelihood no es certeza; ensayar presión de refrigerante/burbujas/leakdown |
| 120 psi en cuatro cilindros | Compresión sin WOT en un caso [CF11] | Método de medición debe afectar sensor/fixture |
| Inyectores Y7 180 cc/min y Y8 240 cc/min | Afirmación de usuario [CF14] | `PENDING`: hallar caracterización exacta por pieza/presión |
| 34–36 psi fuel rail | Post de foro [CF13] vs manual 28–36 psi con vacío [CF07] | Priorizar OEM y hacer curva de regulador |
| 5–10 psi caliente al ralentí en algunos Y7 | Relatos en foros [CF61] frente a mínimo **10 psi a 80°C** de manual [CF02] | Caso de límite/posible desgaste o sensor, **no** «normal OEM» |
| Torque correcto pero rosca que gira | Problema reportado [CF24] | Mantener torque aplicado y clamp load independientes; diagnosticar stripping |
| Números de ECU P2E diferentes | Discusiones identifican P2E y sufijos de mercado/transmisión [CF48][CF49] | Fijar ID **completo**, no aceptar `P2E` genérico |
| Distribuidores TD-80U/TD-98U | Catálogo por aplicación [CF19] | Congelar mercado/sufijo de vehículo; no mezclar harness y ECU |
| Cabezas Y7/Y8/Z6 en datos dyno | Participantes «Y7» montan culatas/IM/ECU distintas [CF53][CF54] | Comparar solo diseños con metadatos exhaustivos |
| «0,003–0,004 in PTW» | Medición reportada por usuario [CF22] | **No** un límite de servicio ni tolerancia aprobada |
| Un post afirma «AFR 11,4 seguro» | Valor aislado de caso turbo [CF54] | No usar AFR solo como predictor de knock o seguridad |

**Política automática de conflicto:** ante discrepancias no sobrescribir el parámetro OEM. Crear `ConflictRecord { parameter, claim_a, claim_b, units, conditions_a, conditions_b, variant_a, variant_b, uncertainty, decision, reviewer }`; si el conflicto no se explica por régimen, temperatura, variante o instrumento, bloquear la calibración de esa feature.

## 23.8. Ampliación técnica: fenómenos donde los foros permiten diseñar variables, pero no ajustar constantes

### 23.8.1 Culata, tornillos y junta como interfaz termo-mecánica

Los foros prueban que son frecuentes los problemas de roscas, errores de torque, daño térmico o procedimientos mezclados. Fel-Pro y ARP documentan principios generales de rugosidad, carga y lubricación de roscas [CF24][CF25][CF34][CF35][CF62]. **Modelo mínimo**:

- Por tornillo: longitud/diámetro/paso, material, zona elástica/plástica, fricción de rosca y asiento, torque aplicado, torque de arranque, ángulo, precarga y desgaste de rosca.
- Por interfaz culata–junta–bloque: área local, rugosidad/planitud, espesor comprimido, temperatura local, carga de contacto, geometría de pasos aceite/refrigerante/combustión, conductancia de fuga entre **pares de volúmenes**.
- Energía/degradación: carga cíclica, dilatación, erosión térmica/fuga e historia; falla **gradual si el modelo lo permite**. Un apriete incorrecto no activa inmediatamente un Boolean `blown_gasket`.
- No inventar relación determinada `49 ft-lb → X kN` sin conocer precarga/fricción; secuencias/torques citados informalmente en [CF24] deben validarse con manual OEM antes de entrar en operaciones del taller.

### 23.8.2 Cilindros, honed surface, anillos y aceite

- Un **cilindro** no es solo el número de diámetro: `bore(z, phi)` reducido a top/mid/bottom + ovalidad, conicidad, rayas localizadas, estado de acabado, temperatura; cada componente usa una resolución compatible con el solver.
- Separar **compresión** (sellado gas), **control de aceite** (película/arrastre/rascado) y **desgaste** (erosión/abrasión), con acoples pero sin un mismo «health». Motorservice describe daños por fuel washing y plateau honing, útil para definir rutas causales [CF32].
- Juego pistón-camisa y holguras de anillos son **variables iniciales con incertidumbre**. Reutilizar un mismo número para OEM e internos forjados de turbo es incorrecto.
- El leak-down se calcula con conductancias de rutas y diferencial de presión; la lectura del equipo depende de regulador/restrictores y presión aplicada. **No equiparar `% leakage` entre distintos testers sin calibración.**

### 23.8.3 Lubricación y cojinetes

- Modelar bomba/relief/pasos como red de presiones, caudales y temperatura; propiedades reológicas del aceite varían con temperatura y cizalla según nivel de fidelidad. Bearings necesitan carga dinámica, juego diametral, geometría y un modelo de película (reducido inicialmente) [CF02][CF04][CF05][CF33].
- **La presión en galería no es la presión hidrodinámica local del cojinete**: un manómetro puede mostrar lectura aceptable y existir una falla local por holgura, alineación, suministro o contaminación.
- Experiencias de aceite a 5–10 psi no definen una regla «sigue andando». Mecanismos de fallo necesitarán criterios energéticos/cargas y datos experimentales.

### 23.8.4 Comportamiento eléctrico observable

- Fuentes de voltaje reales, batería/arranque, caída de tensión por cable/masa/terminal, sensores inductivos al girar, amplitud según RPM, phase decoders, saturación de acondicionamiento, igniter, bobina y chispas por cilindro [CF36][CF38][CF39].
- Una lectura de `sensor_voltage` no sustituye `sensor_physical_signal` más `wire_voltage_drop` más `ecu_adc_value`; deben poder diferir si hay corrosión o cortocircuitos.
- La existencia de código OBD es consecuencia de monitor ECU/versionado; no es una pista autoral. Hay síntomas antes de DTC y DTC intermitentes sin síntoma claramente perceptible [CF44].

## 23.9. Esquemas propuestos para ingestión SIN modificar la arquitectura central

### Registro de evidencia de campo (ilustrativo YAML; no API definitiva)

```yaml
id: CASE-M10-A
kind: FORUM-MEASURED
vehicle:
  engine_code: D16Y7
  year: null                 # no inventar
  market: null
  ecu_id: null
  modifications: unknown
measurement:
  instrument: compression_gauge
  cylinder_order: [1, 2, 3, 4]
  values: [148, 150, 153, 153]
  unit: psi
  engine_temp: unknown
  cranking_rpm: unknown
  throttle_open: unknown
  strokes: unknown
provenance:
  url: https://honda-tech.com/forums/tech-misc-15/compression-test-results-798869/
  source_class: FORUM-MEASURED
  reviewed_against_oem: false
  use_as_calibration_target: false
allowed_uses:
  - adversarial_instrument_test
  - measurement_metadata_training
```

### Registro de pieza física (ejemplo de referencia sin curva inventada)

```yaml
id: HONDA-1997-DX-5MT-INJECTOR-SET
source_class: OEM-EPC
part_number: 06164-P2A-000
application: 1997_CIVIC_DX_5MT_KW
quantity: 4
properties:
  nominal_flow_rate_m3_s: null    # requiere ΔP y combustible
  deadtime_map_vs_voltage: null   # pendiente caracterización
  electrical_resistance_ohm: null # pendiente especificación/medida
provenance_url: https://www.hondapartsnow.com/parts-list/1997-honda-civic--2dr_dx-kw_5mt/engine/intake_manifold_sohc.html
```

### Conjunto de datos de dyno (no aceptar únicamente pico)

```yaml
session_id: unknown
engine_configuration_hash: required
vehicle_and_gear: required
dyno_type_and_calibration: required
ambient_temp_K: required
barometric_pressure_Pa: required
humidity_percent: required
fuel_grade_and_ethanol: required
boost_abs_Pa_vs_rpm: required_for_turbo
afr_or_lambda_vs_rpm: desired
spark_deg_btdc_vs_rpm: desired
rpm_points: required
torque_Nm_points: required
power_W_points: derived_check
measurement_uncertainty: required
forum_peak_whp_only: cannot_be_gold_standard
```

## 23.10. Escenarios adversariales adicionales: incorporar a backlog de TESTS, no declarar que ya se implementaron

| ID | Variación controlada | Observable esperado, *sin respuesta rígida* | Sprint propietario |
|---|---|---|---|
| `FIELD-01` | Dos motores idénticos salvo `ring_end_gap` | Diferencia dependiente de temperatura/ΔP en fuga y compresión | S23/S24 |
| `FIELD-02` | Aumentar ovalidad y conicidad de bore, mantener media | Cambio de contacto/aceite/blow-by según modelo | S23/S26 |
| `FIELD-03` | Cambiar superficie de honing conservando diámetro | Desgaste/break-in del aro, consumo de aceite; marcar incertidumbre | S26/S36 |
| `FIELD-04` | Un cojinete de bancada fuera de límite, otros sanos | Cambio en red de lubricación y potencial pérdida de película local | S22/S26 |
| `FIELD-05` | Apriete con torque indicado pero rosca dañada | Clamp insuficiente pese a lectura de llave aparentemente correcta | S08/S23 |
| `FIELD-06` | Junta presente pero irregularidad de superficie | Fuga emergente si presión de contacto no alcanza umbral modelado | S08/S23 |
| `FIELD-07` | Compresión con mariposa cerrada vs WOT | Lectura cambia por masa admitida; mismo estado mecánico | S24 |
| `FIELD-08` | Dos compresímetros diferentes/arranque lento | Lecturas distintas por volumen muerto/valva/velocidad | S24 |
| `FIELD-09` | CKP salida marginal durante cranking | Decodificador puede perder sincronía/intermitencia observable en osciloscopio | S17/S18 |
| `FIELD-10` | CKP cable intermitente, sensor intacto | Mismo DTC posible, forma de onda distinta por punto de medición | S17/S19/S20 |
| `FIELD-11` | IACV físicamente atascado vs pin ECU abierto | Ambos alteran ralentí, pero mediciones eléctricas distintas | S19/S20 |
| `FIELD-12` | Pérdida de vacío en junta del colector | Aumenta caudal no gobernado; regulador/ECU responden físicamente | S12/S19/S20 |
| `FIELD-13` | Presión de rail con vacío conectado/desconectado | Debe cambiar con referencia del regulador sin `if vac_off then 42 psi` mágico | S13/S20 |
| `FIELD-14` | Sensor de aceite «OK», cojinete local con suministro bajo | Medida de galería distinta a riesgo local; no false assurance | S22/S26 |
| `FIELD-15` | Desfase mecánico 1 diente, avance eléctrico ajustado correctamente | Distribución de válvulas sigue desfasada | S11/S18 |
| `FIELD-16` | Intercambio de cables de encendido | Chispa llega a cilindro en fase incorrecta: combustión anómala o ausente | S18/S29 |
| `FIELD-17` | Cambiar inyector sin actualizar ECU | Cambios de lambda/transitorios limitados por closed-loop/actuación concreta | S13/S18 |
| `FIELD-18` | Incremento de boost sin mejorar intercooler | Dinámica térmica de admisión y masa atrapada responden; no bonus fijo | S35 |
| `FIELD-19` | Incremento de boost con duty limitado | AFR/lambda y torque cambian según caudal disponible | S35 |
| `FIELD-20` | Dos dynos, misma configuración, distinta marcha/corrección | Diferencias interpretables sin recalibrar física al pico más alto | S25/S28 |
| `FIELD-21` | Fuga compresión→refrigeración sin aceite contaminado | Pérdida de refrigerante no requiere «milky oil» | S23/S24 |
| `FIELD-22` | Ausencia junta con culata físicamente asentable | Geometría y rutas de contacto cambian; resultado depende de tolerancias y contacto | S08/S23/S27 |

**No añadir estos casos como “tests que pasan”.** Son diseños falsables pendientes, cada uno requiere un oráculo: conservación, monotonicidad bajo hipótesis declaradas, comparación manual, banco controlado o sensibilidad de un modelo alternativo.

## 23.11. Qué es recuperable de foros y qué sigue bloqueado por datos experimentales

> **Nota cronológica (actualización v0.2):** este inventario refleja el estado anterior a `D16Y7_RESEARCH_v0.1.md` y `D16Y7_RESEARCH_DONANTES_v0.1.md`. Se han localizado curvas comunitarias de flujo de admisión y escape **con limitaciones de muestra y protocolo** (§24). Por tanto, «no encontrada» significa «no encontrada en aquella campaña», no «no existe dato nuevo». Siguen sin resolverse las curvas OEM certificadas, el perfil stock Y7 completo y la calibración predictiva.

| Parámetro / objeto | Estado tras investigación | Mejor siguiente adquisición |
|---|---|---|
| Lista detallada de componentes/PN compatibles del 1997 DX | **Parcialmente recuperado** en EPC | Exportar manualmente PN y revisión, equivalencia por mercado, cross-check Honda |
| Tolerancias de bancada/biela/culata y ensayos nominales | **Parcialmente recuperadas en manual HTML** | Captura con pág./edición Honda para dataset S01 |
| Torque OEM, secuencia y ángulos exactos de cada perno | **Manual accesible**; aún **no transcritos exhaustivamente** | Página concreta Honda, herramientas/rosca limpia y condiciones |
| Curva de leva **stock** Y7 (lift vs ángulo, rampa, duration @ lift especificado) | **No encontrada** completa | Perfilador de levas/CAD metrología, fabricante OEM, laboratorio |
| Flujo culata OEM (CFM vs lift, ΔP, temperatura) | **No encontrada con protocolo completo** | Flow bench con culata/puertos/valvetrain identificado |
| Dimensiones completas del puerto/colector/plenum | **Solo piezas identificadas**, geometría incompleta | Escaneo/despiece/CAD propio + medidas de volumen |
| Mapa de compresor/turbina turbo específico | No es OEM; Garrett útil conceptualmente | Selección turbo con mapa de fabricante y pruebas a futuro |
| Curva de presión **indicada por ángulo** de cilindros Y7 | **No encontrada** | Transductor in-cylinder + encoder de cigüeñal/banco |
| FMEP / fricción vs RPM, T y carga del D16Y7 | **No encontrada** | Motoring dyno o identificación inversa con amplia incertidumbre |
| Constantes desgaste real de aros/honing Y7 | **No encontradas** | Ensayos de durabilidad/materiales; modelos acotados |
| Distribución clamp/contact pressure culata-junta-bloque | **No encontrada** | Elementos finitos offline + geometría y metrología |
| Forma de onda exacta CKP/TDC/CYP/CKF, dientes/índice, offset | **Casos indirectos**, no oscilogramas certificados para variante congelada | Captura de 2 canales en motor stock y documentación OEM |
| Firmware P2E OEM, mapa combustible/avance y estrategias de fallback | **P2E identificado**; maps completos no confirmados/licencias por revisar | Servicio OEM, captura diagnóstica, reimplementación observable y pruebas |
| Curva de inyectores `Q(ΔP)`, deadtime/batería/atomización | **PN identificado**, caudal de foro contradicho | Banco de inyectores y pruebas de voltaje/ΔP |
| Mapa de bomba de aceite Q–ΔP/temperatura y alivio | **PN identificado**, presión de servicio sí | Banco de bomba o identificación paramétrica documentada |
| Mapa bomba agua/termostato/intercambio térmico | **PN identificado**, mapa no | Caracterización por caudal, ΔP, T y posición válvula |
| Curva stock de torque completa corregida | **No identificada con cadena metrológica sólida** | Dyno sano OEM sin swaps, SAE/reporte bruto/CSV |
| Fractura predictiva de biela OEM a sobrecarga combinada | **No identificada** | Material, proceso, geometría, historial de fatiga y validación |
| Sensor O2 y control closed loop exacto por ECU | Descripción general; **datos específicos pendientes** | Diagrama OEM y datalog con transitorios |
| Masas/inercia de pistón-biela-cigüeñal/volante individual | Catálogo muestra identidad, **no propiedades dinámicas** | Pesaje, péndulo/inercia, modelos CAD y balanceo |

**Conclusión operativa:** los foros cubren muy bien **qué falla, cómo se diagnostica, qué se mide y qué combinaciones existen**; cubren imperfectamente **cuánto** cambia una ley termodinámica con un parámetro. La estructura física se aprende ahí; sus coeficientes científicos no se obtienen honestamente de anécdotas.

## 23.12. Inserciones recomendadas en sprints **existentes** (sin cambiar S00–S37)

- **S00:** añadir `EVIDENCE_POLICY.md` con las etiquetas de 23.1, decisiones de fuente y revisión de licencias/contenido de foros.
- **S01:** abrir `OEM_CATALOG_1997_DX_5MT.csv` con PN por variante, y `D16Y7_OEM_SPECS.csv` con compresión, aceite, clearances, lash y presión rail; registrar edición y página **antes de declarar `[F]` definitivo**.
- **S03:** tipos dimensionales (`kPa`, `Pa_abs`, `Pa_gauge`, psi, kPa, mm, m, °BTDC, °CA, cc/min a ΔP de referencia) + tests de conversión.
- **S04:** tipo `CatalogPartDefinition` separado de `InstalledPartInstance`, registro de marcas/proveedor/sobremedida y sustituciones.
- **S07/S08:** torquímetro con lectura ≠ precarga; roscas, contacto, sellado de varias cavidades, orden de montaje y precaución sobre aprietes de foro.
- **S11:** diferencia estricta PMS/timing de válvulas/timing de chispa; medición futura lift-vs-CA en lugar de camshaft genérico.
- **S13:** montar primero rail+regulador según vacío y modelo físico; dejar deadtime/caudal de inyector como dato pendiente.
- **S17/S18:** catálogo de distribuidores TD-80U/TD-98U por aplicación; P2E OEM y ECU tunable posterior como entidades separadas.
- **S19/S20:** osciloscopio en sensor y pin ECU, multímetro con cargas, fallos de masa/terminal, IACV 2/3 cables según variante y DTC sin oráculos.
- **S22:** incorporar presión galería vs película local, holguras por muñequilla y mediciones comparables a 80 °C.
- **S23/S24:** compresión nominal/mínima con **condiciones** 250 rpm + WOT y leakdown por camino de fuga; enriquecimiento con casos M05–M12.
- **S25:** cargar testimonios como `field_observations`; solo datos validados como `calibration_golden`; falta curva stock completa, iniciar plan de adquisición.
- **S26/S27:** desgaste sin números milagrosos; incluir plano/acabado de superficies, giro del tornillo y múltiples causas para mismo síntoma.
- **S29:** añadir `FIELD-01`–`FIELD-22` como backlog adversarial, con oráculos a redactar antes de implementar.
- **S34/S35:** conservar T01–T08 como candidatos para pruebas **posteriores**, no benchmarks actuales de stock.

## 23.13. Protocolo para seguir investigando foros con mucho más rendimiento

En vez de búsquedas generales `D16Y7 HP`, priorizar búsquedas de **instrumentos y trazas**: `D16Y7 compression WOT rpm warm`, `D16Y7 leakdown intake exhaust crankcase`, `D16Y7 plastigage measured bearing color`, `D16Y7 oil pressure hot 3000 rpm gauge`, `D16Y7 distributor CKP TDC CYP oscilloscope 1997`, `P2E Honda ECU OEM pinout 1997 manual`, `D16Y7 flowbench head lift CFM pressure`, `D16Y7 stock dyno chart SAE correction`, `D16Y7 injector part 06164-P2A-000 characterization`, `D16Y7 cam degree lift table`, `D16Y7 cylinder bore taper dial bore gauge`, `D16Y7 crank main journal measurements`.

**Criterios de prioridad para captura de un foro:** (1) foto/tabla de instrumento legible; (2) motor y modificaciones identificados; (3) condición operativa conocida; (4) método reproducible; (5) fuente y fecha; (6) posterior confirmación tras reparación; (7) originales sin marca de agua o derechos esclarecidos para uso como activo. Preferir el hilo con medición completa a cincuenta respuestas de «yo también».

**Campaña mínima recomendable** cuando haya acceso a un D16Y7 OEM funcional (futuro, no asumir disponible):

1. Fotografiar placa y números de motor/ECU/distribuidor, verificar market/trim/manual; medir condiciones base y aceite.
2. Compresión por cilindro siguiendo manual **250 rpm + WOT**, anotando rpm real, batería, temperatura, compresímetro, variación y orden de prueba.
3. Compresión/leakdown con condiciones instrumentales, identificando rutas de fuga y cuantificando incertidumbre.
4. Capturar CKP/TDC/CYP simultáneo a arranque/ralentí/distintas RPM y pulso de encendido/inyección, con escala y trigger documentados.
5. Datalog de MAP/IAT/ECT/TPS/O2, AFR/lambda externa, tensión de batería y ralentí frío/caliente; comparar lectura en ECU vs instrumentación externa.
6. Presión rail con y sin vacío, aceite a 80°C en ralentí y 3k rpm, temperaturas, carga auxiliar, comportamiento termostato y ventilador.
7. Si hubiera medios: curva OEM de torque a carga controlada, caudal de admisión/escape, presión in-cylinder y banco de inyectores. Estos ensayos requieren equipo y procedimientos seguros.

Crear una carpeta `research/field_cases/` con metadatos (datos hechos, no copias de fotografías ajenas sin licencia), `research/oem_specs/` y `research/unsolved_coefficients/`. Versionar cada registro por hash, mantener `raw != normalized != validated` y exigir revisión de otro investigador o agente que no haya extraído el dato.

## 23.14. Referencias adicionales verificadas (enlaces directos)

Las siguientes fuentes se incorporan **como índice de investigación** para regresar al post o página concretos. Se evita afirmar que la reproducción web constituye documentación original licenciada para redistribución. Número de `CF` separado de `R` para no alterar las referencias existentes.

### Taller / catálogos para contraste con foros

- **[CF01]** 1997 Civic DX: compresión nominal/mínima y condiciones: https://workshop-manuals.com/honda/civic_dx_hatchback/l4-1590cc_1.6l_sohc_mfi/engine_cooling_and_exhaust/engine/tune-up_and_engine_performance_checks/compression_check/system_information/specifications/
- **[CF02]** 1997 Civic DX: presión de aceite 80 °C: https://workshop-manuals.com/honda/civic_dx_hatchback/l4-1590cc_1.6l_sohc_mfi/engine_cooling_and_exhaust/engine/engine_lubrication/engine_oil_pressure/component_information/specifications/page_1936/
- **[CF03]** Honda manual 1996–2000, aceite p.236: https://www.manualslib.com/manual/464698/Honda-Civic.html?page=236
- **[CF04]** 1997 Civic DX: cojinetes de bancada: https://workshop-manuals.com/honda/civic_dx_hatchback/l4-1590cc_1.6l_sohc_mfi/engine_cooling_and_exhaust/engine/cylinder_block_assembly/crankshaft_main_bearing/component_information/specifications/page_1770/
- **[CF05]** 1997 Honda Civic workshop manual, tablas block/biela: https://www.carmanualsonline.info/honda-civic-1997-6-g-workshop-manual/?srch=crankshaft+bearing
- **[CF06]** 1996 Civic SOHC procedimiento de válvulas: https://workshop-manuals.com/honda/civic_hatchback/l4-1.6l_sohc_%2816_valve%29/engine_cooling_and_exhaust/engine/cylinder_head_assembly/valve_clearance/system_information/specifications/page_1736/
- **[CF07]** 1998 Civic LX SOHC presión rail, vacío conectado/desconectado: https://workshop-manuals.com/honda/civic_lx_sedan/l4-1590cc_1.6l_sohc_mfi/powertrain_management/fuel_delivery_and_air_induction/fuel_pressure/system_information/technical_service_bulletins/page_4115/
- **[CF08]** Honda DX 1999, juego árbol y soporte: https://workshop-manuals.com/honda/civic_dx_coupe/l4-1590cc_1.6l_sohc_mfi/engine_cooling_and_exhaust/engine/camshaft_lifters_and_push_rods/camshaft_engine/component_information/testing_and_inspection/page_1733/
- **[CF15]** Honda EPC 1997 DX 5MT, culata: https://www.hondapartsnow.com/parts-list/1997-honda-civic--2dr_dx_abs-kw_5mt/engine/cylinder_head_sohc.html
- **[CF16]** Honda EPC 1997 DX 5MT, pistones/arillos: https://www.hondapartsnow.com/parts-list/1997-honda-civic--2dr_dx_a_1_c-kw_5mt/engine/crankshaft_piston.html
- **[CF17]** Honda EPC 1997 DX 5MT, inyectores/regulador/admisión: https://www.hondapartsnow.com/parts-list/1997-honda-civic--2dr_dx-kw_5mt/engine/intake_manifold_sohc.html
- **[CF18]** Honda EPC 1997 DX 5MT, bomba de aceite/relief: https://www.hondapartsnow.com/parts-list/1997-honda-civic--2dr_dx_abs-kw_5mt/engine/oil_pump_oil_strainer_sohc.html
- **[CF19]** Honda EPC distribuidor TD-80U/TD-98U: https://www.hondapartsnow.com/parts-list/1997-honda-civic--2dr_dx_a_1_c-kw_5mt/engine/distributor_tec_westec.html y https://www.hondapartsnow.com/parts-list/1997-honda-civic--2dr_dx-ka_5mt/engine/distributor_tec_westec.html
- **[CF20]** Honda EPC bomba de agua/termostato: https://www.hondapartsnow.com/parts-list/1997-honda-civic--2dr_dx_abs-kw_5mt/engine/water_pump_thermostat_sohc.html

### Foros, piezas y reconstrucción

- **[CF09]** Compresión 148/150/153/153: https://honda-tech.com/forums/tech-misc-15/compression-test-results-798869/
- **[CF10]** Compresión 195/187/192/197: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/compression-test-2586317/
- **[CF11]** Compresión sin WOT 120/118/120/120, procedimiento discutido: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/d16y7-compression-numbers-2370986/
- **[CF12]** D16Y7 115k millas, compresión desigual y wet test: https://honda-tech.com/forums/tech-misc-15/1st-question-compression-pistons-rebuilding-my-bottom-end-921366/
- **[CF13]** Discusión fuel pressure: https://honda-tech.com/forums/tech-misc-15/d16y7-stock-fuel-pressure-778540/
- **[CF14]** Afirmación de caudal inyectores sin banco: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/injector-question-2709234/
- **[CF21]** Colores de cojinetes D16Y7: https://honda-tech.com/forums/engine-machining-assembly-164/d16y7-main-bearing-confusion-3343605/
- **[CF22]** PTW reportado 0,003–0,004 in: https://honda-tech.com/forums/engine-machining-assembly-164/d16y7-piston-wall-clearance-3366917/
- **[CF23]** Discusión de metrología al reconstruir D16: https://honda-tech.com/forums/all-motor-naturally-aspirated-44/need-overview-how-swap-pistons-d16-1640189/
- **[CF24]** Pernos que siguen girando en reparación Y7: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/head-bolts-wont-torque-down-3157347/
- **[CF25]** Falla reportada por aceite en agujeros de tornillos: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/d16y7-still-white-smoke-after-head-gasket-job-3195501/
- **[CF26]** Culata rectificada y refrigerante: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/d16y7-head-milled-past-spec-3190426/
- **[CF27]** P0301/refrigerante/junta vs válvula: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/d16y7-head-gasket-burnt-valve-3341462/
- **[CF28]** Leak-down con fuga por escape: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/please-help-me-diagnose-cause-my-misfire-d16y7-3135976/
- **[CF29]** Compresión baja y consumo de aceite: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/d16y7-compression-test-2019137/
- **[CF30]** Debate retenes válvula/aros/cilindros: https://honda-tech.com/forums/tech-misc-15/d16y7-valve-seal-question-3223869/
- **[CF31]** Swap Y7↔Y8 compatibilidad por piezas: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/%5Bfaq%5D-d16y7-y8-mini-me-write-up-pics-56k-maybe-not-735196/page10/

### Electrónica, encendido y control

- **[CF36]** 1996 Y7 TD-80U y resistencias de CKP/TDC/CYP: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/96-honda-civic-distributor-issue-3194538/
- **[CF37]** ClubCivic CKP/TDC/CYP y resistencia reportada: https://www.clubcivic.com/forum/threads/tdc-cyp-and-ckp-sensor-question.224420/
- **[CF38]** 1996 Y7 arranca con chispa inicial, inspección TDC/ECU: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/weird-no-start-issue-96-civic-non-vtec-d16y7-3287150/
- **[CF39]** No-start resuelto tras distribuidor según autor: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/d16y7-no-start-779465/
- **[CF40]** Timing mecánico y marcas PMS: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/d16y7-crankshaft-tdc-not-matching-up-cam-sprocket-tdc-3183943/
- **[CF41]** Timing base nominal referido 12°±2°: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/d16y7-ignition-timing-help-3014677/
- **[CF42]** Lámpara/conector de servicio/timing: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/ignition-timing-d16y7-3304844/
- **[CF43]** Y7 1999 M/T IACV 3 hilos y ralentí: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/d16y7-high-idle-low-idle-iacv-issues-3374649/
- **[CF44]** P1509 atribuido a conector ECU: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/d16y7-trouble-3334765/
- **[CF45]** Intercambio colector/admisión y oscilación ralentí: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/ex-intake-mani-d16y7-idle-issues-im-lost-3285479/
- **[CF46]** Posible fuga colector/IACV Y7: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/d16y7-high-idle-problem-2978745/
- **[CF47]** Bypass de aire/IACV y diferencias de hardware: https://honda-tech.com/forums/tech-misc-15/idling-idling-adjustment-question-259267/
- **[CF48]** P2E y OBD2a/OBD2b: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/96-civic-ex-d16y7-swap-need-obd2a-obd2b-p2e-ecu-3277640/
- **[CF49]** ECU P2E y sufijos automático/transmisión: https://honda-tech.com/forums/engine-management-tuning-124/what-ecu-used-auto-d16y7-2315941/
- **[CF50]** Discusión reciente de referencia ECU P2E manual, parte completa: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/what-ecu-do-i-need-please-help-3381158/
- **[CF51]** Conversión P06 para motor non-VTEC: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/ecu-d16y7-motor-obd2-obd1-583573/
- **[CF52]** Hondata foro tuning turbo Y7 y mapa/ECU no OEM: https://www.hondata.com/forum/viewtopic.php?p=42937

### Tuning/dyno y mecanismos de fabricantes

- **[CF53]** Y7 6 psi dyno + duty 100 % reportado: https://honda-tech.com/forums/forced-induction-16/d16y7-back-dyno-need-bigger-injectors-940562/
- **[CF54]** Y7 15/17/20 psi, internals, dyno gear: https://honda-tech.com/forums/forced-induction-16/dynoed-d16y7-pleased-results-so-far-1265160/
- **[CF55]** D16Y7 detonación pese a mezcla reportada: https://honda-tech.com/forums/forced-induction-16/detonation-occuring-i-dont-know-why-help-please-840229/
- **[CF56]** Historias de límite stock/overboost: https://honda-tech.com/forums/forced-induction-16/turbo-i-have-stock-d16y7-non-vtec-how-much-psi-can-i-boost-safe-1909029/
- **[CF57]** Discusión límites Y7, no equivalen a resistencia estadística: https://honda-tech.com/forums/forced-induction-16/d16y7-stock-turbo-uh-oh-not-again-_-2977084/
- **[CF58]** Y7 bolt-ons 117 whp reportado: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/d16-y7-dyno-results-117whp-3046018/
- **[CF59]** Búsqueda infructuosa de curva stock en foros: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/factory-dyno-torque-curves-stock-d16y7-3061394/
- **[CF60]** Regrind de leva Y7, duración/lift declarados: https://honda-tech.com/forums/all-motor-naturally-aspirated-44/understanding-specs-681557/
- **[CF61]** Lecturas de presión aceite caliente Y7: https://honda-tech.com/forums/forced-induction-16/oil-pressure-around-5psi-idle-35psi-3500-rpms-help-1578612/page2/
- **[CF32]** Motorservice, microgeometría/honing/anillos (HTML): https://www.ms-motorservice.com/int/en/technipedia/piston-damage-increased-oil-consumption-2
- **[CF33]** King Bearings, holgura y película lubricante: https://www.kingbearings.com/faq/how-much-clearance-should-my-bearings-have/
- **[CF34]** Fel-Pro, limpieza/acabado de superficies: https://www.felpro.com/en-ca/technical/tecblogs/surface-finish.html
- **[CF35]** ARP, instalación de fijaciones y precarga: https://arp-bolts.com/tech/fastener-installation-overview
- **[CF62]** ARP, lubricación, rosca, estudios de torque/precarga: https://arp-bolts.com/tech/fastener-installation-faq

### Avisos de calidad/copyright

- Los números `CF` no indican calidad creciente: **solo identifican el link**.
- No reutilizar fotografías, gráficos de dyno o textos largos de foros/catálogos como contenido del juego sin permisos; sí conservar URLs, hechos verificables y parámetros bajo sus condiciones.
- Las piezas Honda y las características de una ECU aftermarket no son intercambiables; cada prueba debe contener una `configuration_hash` completa.
- Los datos publicados en foros han sido leídos como testimonios: **ninguna frase del tipo «mi motor duró X meses a Y psi» es evidencia suficiente para fijar resistencia de piezas en Newtons, MPa o ciclos de fatiga**.

**CIERRE DE ADENDA 23: la información de campo ya permite crear una batería mucho más realista de ensamblaje y diagnóstico, y resolver parte de las especificaciones piloto; no resuelve por sí sola los mapas de flujo, presión indicada, fatiga o coeficientes de desgaste. El fundamento sigue siendo un programa de medición y validación progresiva.**

---

**FIN — Foundation Research v0.1**

<!-- Referencias Markdown para enlazar cada marcador [Rxx] a una fuente verificable. -->
[R01]: https://www.manualslib.com/manual/464698/Honda-Civic.html
[R02]: https://www.manualslib.com/manual/464698/Honda-Civic.html
[R03]: https://github.com/ange-yaghi/engine-sim
[R03a]: https://github.com/ange-yaghi/engine-sim/blob/master/LICENSE
[R04]: https://github.com/speeduino/speeduino
[R04a]: https://github.com/speeduino/speeduino
[R05]: https://github.com/speeduino/Ardu-Stim
[R06]: https://cantera.org/stable/examples/python/reactors/ic_engine.html
[R06a]: https://github.com/Cantera/cantera/blob/main/CONTRIBUTING.md
[R07]: https://github.com/CMT-UPV/OpenWAM
[R08]: https://support.haltech.com/portal/en/kb/articles/trigger
[R09]: https://www.asme.org/codes-standards/publications-information/verification-validation-uncertainty
[R10]: https://standards.nasa.gov/standard/NASA/NASA-STD-7009
[R11]: https://www.arp-bolts.com/tech/torque-values
[R12]: https://www.felpro.com/technical/tecblogs/understanding-the-real-cause-of-head-bolt-failures.html
[R13]: https://tauri.app/start/
[R14]: https://www1.grc.nasa.gov/beginners-guide-to-aeronautics/engine-thermodynamic-analysis/
[R15]: https://www.garrettmotion.com/knowledge-center-category/oem/expert/
[R16]: https://www1.grc.nasa.gov/beginners-guide-to-aeronautics/internal-combustion-engine-otto-cycle/
[R17]: https://saemobilus.sae.org/standards/j1349_202511-engine-power-test-code-spark-ignition-compression-ignition-installed-net-power-torque-rating
[R18]: https://www.jstage.jst.go.jp/article/jsaeronbun/48/1/48_20174051/_article/-char/en
[R19]: https://arxiv.org/abs/2002.03201
[R20]: https://swehb.nasa.gov/spaces/7150/pages/16449960/SWE-070%2B-%2BModels%2BSimulations%2BTools


<!-- Enlaces de fuentes adicionales de la adenda 23. -->
[CF01]: https://workshop-manuals.com/honda/civic_dx_hatchback/l4-1590cc_1.6l_sohc_mfi/engine_cooling_and_exhaust/engine/tune-up_and_engine_performance_checks/compression_check/system_information/specifications/
[CF02]: https://workshop-manuals.com/honda/civic_dx_hatchback/l4-1590cc_1.6l_sohc_mfi/engine_cooling_and_exhaust/engine/engine_lubrication/engine_oil_pressure/component_information/specifications/page_1936/
[CF03]: https://www.manualslib.com/manual/464698/Honda-Civic.html?page=236
[CF04]: https://workshop-manuals.com/honda/civic_dx_hatchback/l4-1590cc_1.6l_sohc_mfi/engine_cooling_and_exhaust/engine/cylinder_block_assembly/crankshaft_main_bearing/component_information/specifications/page_1770/
[CF05]: https://www.carmanualsonline.info/honda-civic-1997-6-g-workshop-manual/?srch=crankshaft+bearing
[CF06]: https://workshop-manuals.com/honda/civic_hatchback/l4-1.6l_sohc_%2816_valve%29/engine_cooling_and_exhaust/engine/cylinder_head_assembly/valve_clearance/system_information/specifications/page_1736/
[CF07]: https://workshop-manuals.com/honda/civic_lx_sedan/l4-1590cc_1.6l_sohc_mfi/powertrain_management/fuel_delivery_and_air_induction/fuel_pressure/system_information/technical_service_bulletins/page_4115/
[CF08]: https://workshop-manuals.com/honda/civic_dx_coupe/l4-1590cc_1.6l_sohc_mfi/engine_cooling_and_exhaust/engine/camshaft_lifters_and_push_rods/camshaft_engine/component_information/testing_and_inspection/page_1733/
[CF09]: https://honda-tech.com/forums/tech-misc-15/compression-test-results-798869/
[CF10]: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/compression-test-2586317/
[CF11]: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/d16y7-compression-numbers-2370986/
[CF12]: https://honda-tech.com/forums/tech-misc-15/1st-question-compression-pistons-rebuilding-my-bottom-end-921366/
[CF13]: https://honda-tech.com/forums/tech-misc-15/d16y7-stock-fuel-pressure-778540/
[CF14]: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/injector-question-2709234/
[CF15]: https://www.hondapartsnow.com/parts-list/1997-honda-civic--2dr_dx_abs-kw_5mt/engine/cylinder_head_sohc.html
[CF16]: https://www.hondapartsnow.com/parts-list/1997-honda-civic--2dr_dx_a_1_c-kw_5mt/engine/crankshaft_piston.html
[CF17]: https://www.hondapartsnow.com/parts-list/1997-honda-civic--2dr_dx-kw_5mt/engine/intake_manifold_sohc.html
[CF18]: https://www.hondapartsnow.com/parts-list/1997-honda-civic--2dr_dx_abs-kw_5mt/engine/oil_pump_oil_strainer_sohc.html
[CF19]: https://www.hondapartsnow.com/parts-list/1997-honda-civic--2dr_dx_a_1_c-kw_5mt/engine/distributor_tec_westec.html
[CF20]: https://www.hondapartsnow.com/parts-list/1997-honda-civic--2dr_dx_abs-kw_5mt/engine/water_pump_thermostat_sohc.html
[CF21]: https://honda-tech.com/forums/engine-machining-assembly-164/d16y7-main-bearing-confusion-3343605/
[CF22]: https://honda-tech.com/forums/engine-machining-assembly-164/d16y7-piston-wall-clearance-3366917/
[CF23]: https://honda-tech.com/forums/all-motor-naturally-aspirated-44/need-overview-how-swap-pistons-d16-1640189/
[CF24]: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/head-bolts-wont-torque-down-3157347/
[CF25]: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/d16y7-still-white-smoke-after-head-gasket-job-3195501/
[CF26]: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/d16y7-head-milled-past-spec-3190426/
[CF27]: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/d16y7-head-gasket-burnt-valve-3341462/
[CF28]: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/please-help-me-diagnose-cause-my-misfire-d16y7-3135976/
[CF29]: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/d16y7-compression-test-2019137/
[CF30]: https://honda-tech.com/forums/tech-misc-15/d16y7-valve-seal-question-3223869/
[CF31]: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/%5Bfaq%5D-d16y7-y8-mini-me-write-up-pics-56k-maybe-not-735196/page10/
[CF32]: https://www.ms-motorservice.com/int/en/technipedia/piston-damage-increased-oil-consumption-2
[CF33]: https://www.kingbearings.com/faq/how-much-clearance-should-my-bearings-have/
[CF34]: https://www.felpro.com/en-ca/technical/tecblogs/surface-finish.html
[CF35]: https://arp-bolts.com/tech/fastener-installation-overview
[CF36]: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/96-honda-civic-distributor-issue-3194538/
[CF37]: https://www.clubcivic.com/forum/threads/tdc-cyp-and-ckp-sensor-question.224420/
[CF38]: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/weird-no-start-issue-96-civic-non-vtec-d16y7-3287150/
[CF39]: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/d16y7-no-start-779465/
[CF40]: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/d16y7-crankshaft-tdc-not-matching-up-cam-sprocket-tdc-3183943/
[CF41]: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/d16y7-ignition-timing-help-3014677/
[CF42]: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/ignition-timing-d16y7-3304844/
[CF43]: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/d16y7-high-idle-low-idle-iacv-issues-3374649/
[CF44]: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/d16y7-trouble-3334765/
[CF45]: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/ex-intake-mani-d16y7-idle-issues-im-lost-3285479/
[CF46]: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/d16y7-high-idle-problem-2978745/
[CF47]: https://honda-tech.com/forums/tech-misc-15/idling-idling-adjustment-question-259267/
[CF48]: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/96-civic-ex-d16y7-swap-need-obd2a-obd2b-p2e-ecu-3277640/
[CF49]: https://honda-tech.com/forums/engine-management-tuning-124/what-ecu-used-auto-d16y7-2315941/
[CF50]: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/what-ecu-do-i-need-please-help-3381158/
[CF51]: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/ecu-d16y7-motor-obd2-obd1-583573/
[CF52]: https://www.hondata.com/forum/viewtopic.php?p=42937
[CF53]: https://honda-tech.com/forums/forced-induction-16/d16y7-back-dyno-need-bigger-injectors-940562/
[CF54]: https://honda-tech.com/forums/forced-induction-16/dynoed-d16y7-pleased-results-so-far-1265160/
[CF55]: https://honda-tech.com/forums/forced-induction-16/detonation-occuring-i-dont-know-why-help-please-840229/
[CF56]: https://honda-tech.com/forums/forced-induction-16/turbo-i-have-stock-d16y7-non-vtec-how-much-psi-can-i-boost-safe-1909029/
[CF57]: https://honda-tech.com/forums/forced-induction-16/d16y7-stock-turbo-uh-oh-not-again-_-2977084/
[CF58]: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/d16-y7-dyno-results-117whp-3046018/
[CF59]: https://honda-tech.com/forums/honda-civic-del-sol-1992-2000-1/factory-dyno-torque-curves-stock-d16y7-3061394/
[CF60]: https://honda-tech.com/forums/all-motor-naturally-aspirated-44/understanding-specs-681557/
[CF61]: https://honda-tech.com/forums/forced-induction-16/oil-pressure-around-5psi-idle-35psi-3500-rpms-help-1578612/page2/
[CF62]: https://arp-bolts.com/tech/fastener-installation-faq

---

# 24. ACTUALIZACIÓN OPERATIVA — RESEARCH D16Y7 Y DONANTES (2026-10-09)

> **Naturaleza:** adenda **aditiva**. Selecciona solo los hallazgos relevantes para iniciar o preparar sprints; no altera numeración, dependencias ni gates de S00–S37. Los documentos completos `D16Y7_RESEARCH_v0.1.md` y `D16Y7_RESEARCH_DONANTES_v0.1.md` permanecen como fuente documental primaria de esta **síntesis**, con tablas, vínculos originales, metodología y contradicciones. **Ninguna cifra de esta adenda equivale por sí sola a una medición OEM certificada**: registrar fuente puntual y condiciones antes de entrar a un `ModelPack`.

## 24.1 Jerarquía y transferencia de evidencia

- Mantener juntas las categorías de Foundation `[F/D/H/P/X]`, las clases de campo de §23.1 (`OEM-TEXT`, `OEM-EPC`, `MFR-TECH`, `FORUM-MEASURED`, `FORUM-CASE`, `FORUM-CLAIM`, `MISSING-DATA`) y la **relación con el motor objetivo**: `EXACT_OEM` (año/parte/condición), `SHARED_PART` (identidad y revisión demostradas), `ADAPTED_PROXY` (diferencias identificadas, transformación y sensibilidad declaradas) o `EXPLORATORY_PROXY` (comparación cualitativa o de orden de magnitud).
- El research de donantes usa adicionalmente `DIRECTA / ESCALADA / ANALOGÍA`: **no interpretar `DIRECTA` como `EXACT_OEM`** si no existe identidad de pieza documentada. En especial, mismo código de fundición o geometría 75×90 mm **no demuestra** misma leva, válvula, asiento, distribuidor, inyector o ECU.
- Para cada transferencia guardar `target_variant`, `donor_variant`, `part_number`, `assumed_shared_features`, `known_differences`, `original_test_conditions`, `adaptation_transform`, `uncertainty_basis`, `source_id`, `evidence_class`, `supersession_rule` y pruebas de sensibilidad. Un rango extrapolado sin medición no debe convertirse en tolerancia OEM.
- Una validación **de plausibilidad** (resultado en rango de comunidad o catálogo) no es calibración. Rechazar `calibration_golden` sin protocolo/metrología, y no calibrar simultáneamente todas las variables para acertar una sola potencia de catálogo.

## 24.2 Evidencia útil del D16Y7 — síntesis de `D16Y7_RESEARCH_v0.1.md`

| Tema | Hallazgo de investigación (no nueva certificación) | Uso permitido / precaución |
|---|---|---|
| Flujo de admisión | Tabla **CFM vs alzada** para culata rotulada Y7, 0.050–0.500 in, a **28 in H₂O**, publicada por [Bad Guys, D-series shootout](https://the-bad-guys.com/2020/04/13/d-series-cylinder-head-shoot-out/?v=84de8e2b14bb) | Dataset observacional `FORUM-MEASURED`; verificar preparación de asientos, válvulas y si el encabezado «stock» significa OEM intacto; guardar puntos **crudos**, no solo Cd transformado. La división **Q/2 por válvula** es una inferencia geométrica que debe documentarse y contrastarse con la metodología original del banco. |
| Distribución | Lash frío reportado **0.18–0.22 mm adm. / 0.23–0.27 mm esc.**; rocker ratio **1.60 adm. / 1.80 esc.** atribuido a fabricantes/perfiles relacionados | Candidato de servicio + ratio de referencia, sujeto a variante/pieza del tren de válvulas. No dar por demostrado el ratio Y7 OEM solo por catálogos aftermarket. |
| Compresión de arranque | **1270 kPa nominal / 930 kPa mínimo / 200 kPa diferencia máxima**; condiciones **250 rpm y WOT** según reproducción de información Honda | Fixture del **procedimiento de instrumento**, no presión pico de combustión. Revisar **presión absoluta vs manométrica**, calibración, acumulación por ciclos, llenado, IVC y fuga antes de inferir exponente politrópico o compresión efectiva. |
| Aceite y combustible | Referencias de taller: **≥70 kPa al ralentí y ≥340 kPa a 3000 rpm con aceite 80 °C**; rail **270–320 kPa sin vacío / 200–250 kPa con vacío** | Verificar páginas, variante y referencia manométrica. Distinguir `p_rail_abs`, `p_manifold_abs` y `Δp_inyector = p_rail_abs - p_manifold_abs`. Un regulador manifold-referenced ideal conserva aproximadamente Δp aunque baje presión del rail con vacío. |
| Encendido y ECU | Base timing ~**12° BTDC** reportado para aplicación Y7 con procedimiento específico; P2E identificada, mapas/ROM OEM **no recuperados por el research** | Confirmar con edición Honda del año elegido. Implementar ECU propia **P2E-like** etiquetada, nunca atribuir mapas aftermarket o de P28 al software OEM. Diferenciar CKP/TDC/CYP, variantes OBD2a/OBD2b y configuración real del distribuidor. |
| Inyector Y7 | PN de kit `06164-P2A-000` citado; **180–190 cc/min sin ΔP de referencia declarada**, fuente comunitaria | **No** fijar flujo OEM por inferencia de BSFC o duty. Un duty relativamente bajo no demuestra que un inyector sea incompatible. Mantener `reference_pressure: null` en fuente cruda; el valor `185 cc/min @ 3 bar` es **hipótesis elegida**, no medición Y7. |
| Combustión/fricción | Priors publicados de **Wiebe, Woschni y Chen–Flynn**; FMEP y transferencia térmica necesitan coeficientes/dominio propios | ModelCards `EXTRAPOLATED`, sensibilidad, balances y comparación con alternativas. Valores de literatura externa no equivalen a coeficientes calibrados del D16Y7. Evitar predictor de knock o fallo estructural concluyente sin validación. |
| Potencia y diagnóstico | Referencia ~**106 hp @ 6200 rpm**, **103 lb-ft @ 4600 rpm** por fuentes secundarias; test de foro ~**91.1 whp** sin protocolo completo; varios casos de misfire, sensores y reparación | Envolvente amplia de plausibilidad, **no objetivo único de calibración** ni curva stock completa. Casos `FORUM-CASE` conservan resolución/inconclusión; no convertir reparaciones autodeclaradas en mecanismos demostrados. |

## 24.3 Investigación de donantes — síntesis de `D16Y7_RESEARCH_DONANTES_v0.1.md`

| Donante / área | Datos recuperados | Regla de transferencia al Y7 |
|---|---|---|
| **D16A6 SOHC non-VTEC** / leva | Diámetro/carrera 75×90 mm; lobe lift reportado **0.225 / 0.193 in**, duración @0.050 **194°/196°** (esta última con evidencia comunitaria menos fuerte); LSA ~110° reportado | **Proxy inicial de leva**, no leva Y7 OEM ni perfil lift-vs-angle. Su alzada de válvula calculada depende del rocker ratio **real de cada variante**, no necesariamente del Y7. No fijar ICL straight-up como dato medido. [Hilo A6](https://honda-tech.com/forums/honda-crx-ef-civic-1988-1991-3/specs-stock-sohc-non-vtec-cam-94750/) |
| **D15B7 / D17A1** / sensibilidad de leva | Alzadas y duraciones comunitarias permiten comparar perfiles menos agresivos; atribución A1/A2 del dato D17 requiere confirmar | Son **escenarios de sensibilidad**, no cotas demostradas del Y7. La duración advertised depende de la altura de medición; no sustituir perfil de válvula por una duración aislada. |
| **D14 con culata reportada P2A-06** / escape | Serie I/E @28 in H₂O, lift 0.100–0.450 in; ratio comunitario **~0.77–0.82** para la mayoría de puntos, según [hilo dodo-upgrades](https://dodo-upgrades.nl/phpBB3/viewtopic.php?t=363) | **Proxy de escape especialmente valioso**, pero la supuesta identidad de fundición y estado de preparación necesitan verificarse; banco casero e incertidumbre no calibrada. Registrar todos los puntos y el dip de admisión a 0.250 in. La cota **±10%** del research es una propuesta, no intervalo estadístico validado. |
| **Keihin Honda CBR600RR** / inyección | Deadtime reportado **1.29 ms @10 V, 0.93 @12 V, 0.62 @14 V, 0.50 @15 V** | Solo `EXPLORATORY_PROXY` para sensibilidad; diferente inyector, mecánica y posible presión. **No atribuir al P2A-000** y no tratar ±0.3 ms como incertidumbre calibrada del Y7. |
| **Bosch y otros inyectores** / ley de caudal | Mediciones comunitarias Q(ΔP) coherentes con **Q ∝ √ΔP** en condiciones ensayadas; ejemplos aftermarket de deadtime(V, ΔP) | Ley reducida candidata para flujo **de presión diferencial**, no usar la presión del rail manométrica como ΔP si el regulador compensa el vacío. Mantener no linealidad de pulsos cortos y límites de validez por inyector. |
| **Otros SI atmosféricos** / FMEP, IMEP, BSFC | Pipitone (1.24L) y fuentes de motores de distinta arquitectura aportan orden de magnitud; BMEP calculados de familia D ~9.6–10.4 bar en pico de potencia | Solo **cross-check localizado** de condiciones, no límite universal; si el resultado sale del rango, investigar **modelo y condiciones**, no concluir automáticamente «bug». `p(θ)` y FMEP propios Y7 siguen sin caracterizar. |

## 24.4 Guardrails matemáticos antes de convertir proxies en fixtures

1. **Área y coeficiente de descarga:** una tabla `Cd(L)` derivada usando área de cortina **no se reutiliza automáticamente** multiplicándola por `min(A_cortina, A_garganta)`: cambia el área de referencia y puede **contabilizar doble** la limitación del puerto. Conservar el caudal bruto a ΔP y la definición explícita de área/normalización; validar conservación y tendencias antes de reducir a `Cd`.
2. **Presiones y sensores:** tipar presión absoluta, manométrica y diferencial. Los datos rail con vacío/sin vacío no justifican por sí solos un error de 8.7% en caudal. Modelar `ΔP` real sobre inyector, regulador, MAP, bomba y retornos, con escenarios de avería.
3. **Compresión medida:** no inferir un único `n` directamente de relación 9.4:1 y **184 psi manométricos** sin convertir a absoluto ni conocer IVC, condiciones térmicas y el protocolo del compresímetro. El IVC calculado desde *advertised duration* es aproximación dependiente de definición de lift, fases de apertura y retroflujo; no es cierre efectivo identificado.
4. **Identificabilidad:** una curva de flujo medida, un pico de potencia o un BSFC supuesto no determinan por sí solos VE, geometría de levas, p(θ), FMEP, curvas de inyectores ni vida de fatiga. Mantener parámetros no identificados y tests contrafactuales; separar datasets de plausibilidad de datos de calibración y holdout.
5. **Donantes y temporalidad:** no usar números de otras variantes sin `VariantId` ni asumir transferencia directa porque un motor sea D16 SOHC. La ECU `P2E-like` es una implementación de referencia **no OEM**. Si aparecen mediciones Y7 auténticas, **versionar/sustituir** los proxies y ejecutar regresión sin alterar API de submodelos.

## 24.5 Relación con los sprints existentes (ningún sprint nuevo)

| Sprint | Hallazgo que debe consultar | Restricción de aceptación |
|---|---|---|
| **S00** | Charter §0.1; política de evidencia y transferencia §24.1; ambos research archivados por separado | Referencias presentes; alcance comprendido; **sin implementación física** |
| **S01** | Condiciones OEM de §24.2, PN, año/ECU, contradicciones | No aprobar dato `EXACT_OEM` solo por observación de foro o catálogo aftermarket |
| **S03** | `Pa_abs`, `Pa_gauge`, `Δp`, unidades de lift y caudal CFM/ΔP | Conversiones y bases de medición explicitadas y testeadas |
| **S09–S12** | Compresión/IVC, perfil A6 con limitaciones, flujos admisión/escape | Reproducibilidad + conservación; **no** convertir proxy en curva OEM Y7 |
| **S13 / S17–S20** | Regulador referenciado, deadtime analógico, P2E-like, CKP/TDC/CYP y síntomas de campo | Señales observables causales; sin PIN/DTC inventados atribuidos a Honda |
| **S22–S25** | Presiones de servicio, compresímetro, FMEP, BMEP y banco | Tests de instrumento + sensibilidad; ni potencia ni compresión solas prueban validación predictiva |
| **S26–S29** | Casos de montaje y diagnóstico + incertidumbres materiales | Fallos originados en estados, sin `if` punitivo ni tasas de desgaste arbitrarias |

**Pendientes que NO bloquean S00 ni un L1 honesto:** curva alzada-vs-ángulo stock Y7, identificación verificada de flujo de escape OEM, deadtime P2A-000 vs V/ΔP, mapas P2E, curva completa stock, presión indicada `p(θ)`, FMEP D16Y7 y coeficientes de desgaste/fatiga. No declarar que la búsqueda de foros prueba su inexistencia: el estado es **«no localizado en los dos research»**. Priorizar adquisición conforme a sensibilidad del modelo y valor de cada medición.

**Control de cambios:** los research siguen siendo documentos de investigación sin condición de verdad normativa; el Foundation define reglas de uso. Esta adenda **no modifica** S01–S37, y sus valores son **candidatos con evidencia y supuestos**, no fixtures aprobados automáticamente.
