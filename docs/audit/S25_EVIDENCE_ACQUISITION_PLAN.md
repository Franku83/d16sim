# Plan independiente de adquisición de evidencia S25

Fecha de consulta: **2026-10-10**. No se adquirieron mediciones experimentales ni se inventó un dataset D16Y7. Los ocho vacíos del informe S25 permanecen abiertos. Las fuentes públicas siguientes son documentación de métodos, instrumentación o proxies explícitos; ninguna demuestra potencia, combustión, fricción o desgaste del D16Y7 auditado.

La adquisición debe fijar primero variante/año/ECU/distribuidor, identificación física M1/M2, configuración de admisión/escape, estado de mantenimiento y condición inicial. D16Y7 distinto de variante desconocida no se mezcla como nominal sin justificación. Los valores OEM publicados pueden acotar protocolos, pero no sustituyen registros de ensayo con incertidumbre y procedencia. Antes de ajustar el Core hay que revisar F01/F03–F10/F15/F18: un parámetro puede absorber errores de software y parecer calibración exitosa.

## Prioridad por incertidumbre, accesibilidad e identificabilidad

| Orden práctico | Vacío | Reducción esperada de incertidumbre / identificabilidad | Accesibilidad, coste y dificultad | Dependencias |
|---|---|---|---|---|
| 1 | 5: inyectores | Alta: separa entrega de combustible de eficiencia de combustión; caudal y deadtime identificables por barrido | Banco dedicado accesible; coste bajo–medio; dificultad media | Identificación exacta, presión diferencial y combustible medidos |
| 2 | 4: MAF/MAP | Alta: limita llenado y dosificación; MAP solo no identifica masa | Sensores/calibración de caudal; medio; dificultad media | Banco inyector para contrastar balance con lambda; T y estado estable |
| 3 | 1: dinamómetro | Alta para desempeño global; identificación de mecanismos pobre si está solo | Alquiler/instrumentación; medio–alto; dificultad media–alta | Fuel/air, ambiente, encoder, protocolo de estabilidad |
| 4 | 7: warmup | Media–alta para térmica si entradas térmicas conocidas; confusión C/hA si no | Registrador/sensores; bajo–medio; dificultad media | Consumo, carga, ambiente y operación de termostato/ventilador |
| 5 | 6: aceite | Alta para hidráulica/viscosidad; limitada para película o vida útil | Presión/T y laboratorio de aceite; medio; dificultad media | Aceite identificado, caudal/curva de bomba y geometría cuando se inferirán parámetros |
| 6 | 2: p(θ) | Muy alta para separar indicada, combustión y pérdidas; IMEP/fase directamente observables | Banco de investigación, sensor/encoder; alto; dificultad alta | Dyno, fuel/air, calibración p/TDC, geometría |
| 7 | 3: motored | Alta para pérdidas bajo condiciones de arrastre; no identifica automáticamente fricción fired | Banco que pueda impulsar; alto; dificultad alta | p(θ) motored, estados térmicos y pérdidas del banco/accesorios |
| 8, reservar desde inicio | 8: segundo motor | Alta para falsar transferencia; no añade identificabilidad a un train mal diseñado | Segundo ejemplar y campañas; medio–alto; dificultad logística alta | M1 calibrado, modelo/versiones/protocolo fijados, datasets mínimos por capacidad |

El orden es una decisión cualitativa, no una puntuación calibrada ni una obligación serial. Dyno+p(θ)+motored pueden adquirirse juntos si hay acceso a laboratorio; M2 se reserva antes de entrenar y se sella para evitar selección por resultados. Warmup y aceite pueden registrarse durante campañas principales. Priorización propuesta, no contratación ni ejecución autorizada de ensayos físicos.

## 1. Dinamómetro: potencia y torque externos

**Observables y parámetros.** Torque absorbido en eje, RPM sincronizada, potencia como τ·ω, MAP, caudal de aire, lambda, consumo másico, ECT/IAT/aceite, presión/T/humedad ambiente. Restringe torque neto por régimen; con p(θ) y motored puede separar contribuciones de Wiebe, llenado y fricción. Una curva de torque sola admite compensaciones entre esas familias y no las identifica de manera única.

**Fuentes/instrumentación.** Información OEM de servicio sirve para condiciones/configuración, sin curva cruda de validación hallada. Banco de motor con celda de carga/torquímetro y encoder es preferible para potencia efectiva del Core. Banco de chasis mide en ruedas: transmisión/neumáticos/temperatura y coastdown son factores de confusión, no se convierten mediante un porcentaje arbitrario. [SAE J1349_202511](https://saemobilus.sae.org/standards/j1349_202511-engine-power-test-code-spark-ignition-compression-ignition-installed-net-power-torque-rating) es referencia de ensayo de potencia neta; solo se consultó metadata pública, no texto completo ni un certificado del simulador.

**Adquisición y calidad.** Medir ceros/calibración antes y después, accesorios, ambiente, combustible y estado térmico. Registrar series originales con tiempos, holdpoints a varias cargas/régimen y repeticiones; separar barrido transitorio de punto estacionario. Criterio de estación usa deriva de RPM/temperaturas y presupuesto de incertidumbre prefijado, con ventana resuelta, no ajuste oportunista de umbrales. Retener potencia observada y corrección con método/versiones independientes; no aplicar una corrección si sus condiciones no se cumplen.

**Incertidumbre, uso y suficiencia.** La incertidumbre se obtiene de certificado de torque/encoder, deriva, repetibilidad, corrección ambiente y configuración; no se promete un porcentaje sin banco. Confunden pérdidas de transmisión, carga accesoria, lambda, combustible y thermal drift. Entrenar en M1 y reservar cargas/regímenes/campañas; validar transferencia con M2. Mínimo: datos crudos repetidos y trazables, incertidumbre de torque/RPM/ambiente, rango de operación explicitado y capacidad de rechazar puntos no estacionarios; torque solo permite validar torque neto en ese dominio, no toda combustión.

## 2. Presión en cilindro frente a ángulo p(θ)

**Observables y parámetros.** Presión absoluta vs ángulo por ciclo/cilindro, encoder y TDC físico, RPM, geometría medida, masa/fuel, lambda, temperaturas. De p·dV se obtiene trabajo indicado/IMEP; fase y tasa aparente de liberación de calor acotan Wiebe/avance/duración y variación cíclica. La tasa aparente depende de γ, pérdidas térmicas y fugas: no es química medida directamente.

**Fuentes/instrumentación.** Sensor de combustión, acondicionador/charge amplifier, encoder y referencia dinámica TDC, laboratorio capaz de instalar sin alterar cámara de forma no cuantificada. [Kistler, medición de presión](https://www.kistler.com/DK/en/cylinder-pressure-measurement/C00000104) y [sistema TDC 2629D](https://www.kistler.com/INT/en/cp/top-dead-center-sensor-systems-2629d/P0001160) documentan instrumentación; [AVL TDC](https://www.avl.com/documents/10138/2075058/Solution%2BSheet_TDC%2BSensor) es otra referencia de método. Son fabricantes, no mediciones D16.

**Adquisición y calidad.** Calibrar sensibilidad y offset/pegging absoluto; comprobar desfase de canales y TDC antes/después. Registrar fired y motored en puntos conjuntos con dyno, suficientes ciclos para intervalos de repetibilidad, sin guardar solo una media. Elegir resolución angular mediante refinamiento hasta que IMEP/fase/liberación aparente sean menores que el presupuesto de error. Conservar señal original, filtros, timestamps, certificados, geometría y montaje.

**Incertidumbre, uso y suficiencia.** Sensibilidad/offset/thermal shock, localización TDC, resonancia, volumen muerto y propiedades del gas dominan; no transferir precisión declarada del fabricante a todo el montaje. Train M1, holdout de ciclos/campañas/regímenes no usados para elegir Wiebe y finalmente M2. Coste/dificultad altos. Mínimo: p(θ) calibrada y sincronizada con TDC y masa/fuel, presupuesto propagado a IMEP/fase, repetición independiente. Sin ello C7 no es opcional para sostener identificación/validación específica de combustión.

## 3. Ensayos motored: pérdidas y bombeo

**Observables y parámetros.** Torque requerido para impulsar motor sin inyección/ignición, RPM, p(θ), MAP/exhaust backpressure, presión/T aceite, refrigerante, tensión/accesorios. Permite estimar pérdidas netas de arrastre; restando trabajo de bombeo/compresión medido se restringen coeficientes de fricción reducida. El término dependiente de presión fired de Chen–Flynn no queda identificado por extrapolar un motored frío.

**Fuentes/instrumentación.** Banco reversible/motor eléctrico con torque calibrado, encoder y presión indicada. No se encontró ni descargó una curva motored pública D16Y7 con procedencia suficiente. Instrumentación p/TDC de la sección anterior orienta el diseño; fuentes genéricas son referencias de método, nunca datos de este motor.

**Adquisición y calidad.** Medir pérdidas en vacío del banco/acoplamiento; fijar qué accesorios giran. Arrastre a velocidades/cargas de bombeo controladas, varias temperaturas de aceite/refrigerante y repeticiones tras estabilizar. Cortar fuel/ignición de forma verificable. Evitar confundir desaceleración libre con torque motored estacionario sin inercia conocida. Registrar p(θ) y calor/térmica conjuntamente.

**Incertidumbre, uso y suficiencia.** Confunden pumping, sellado, temperaturas de contacto, oil shear, accesorios e inercia del banco. Coste/dificultad altos; dependencia fuerte de p(θ) y óleo. Mínimo: torque y pressure work sobre igual ventana, presupuesto de pérdidas de banco, temperatura estable y rango con excitación suficiente para rango del Jacobiano. Train de fricción M1; holdout de otros estados térmicos y M2. Resultados válidos solo en condiciones motored medidas hasta verificar transferencia fired.

## 4. MAF/MAP: llenado y aire atrapado

**Observables y parámetros.** Flujo másico de aire calibrado, MAP absoluta e IAT sincronizadas, RPM, throttle, presión escape, lambda/fuel, humedad y ambiente. Masa por ciclo y eficiencia volumétrica acotan dosificación/llenado; con perfil de alzada y presión de puertos pueden restringir Cd efectivo. MAP solo no determina masa atrapada ni reparto entre cilindros ni residual.

**Fuentes/instrumentación.** Medidor de referencia másico/volumétrico corregido por densidad, transductor absoluto de presión y T; instrumentos de [Bosch Motorsport](https://www.bosch-motorsport.com/products/) son catálogo de posibilidades, no caracterización MAF Honda. El modelo de sensor se selecciona por rango/pulsación y certificado, sin asumir que instalar uno crea evidencia válida. No se halló dataset MAF/MAP D16 con calibración independiente adecuada en la búsqueda acotada.

**Adquisición y calidad.** Calibrar caudal en su montaje, leak-check, evaluar restricción introducida por medidor. Registrar simultáneamente estacionario y transitorios throttle a varios regímenes, con T/ambiente/fuel. Conservar bandwidth y filtros; integrar flujo sobre ciclos/ventanas completos. Para atribución por cilindro se necesita observación adicional, no repartir siempre 25% como dato medido.

**Incertidumbre, uso y suficiencia.** Pulsaciones/reversión, filtro, fugas, humedad, calentamiento y time-lag confunden; coste medio, dificultad media. Requiere fuel calibrado para balance lambda independiente; Cd no es identificable sin geometría/perfil y excitación de Δp. Mínimo: trazas calibradas que cierren masa de admisión por ventana dentro de incertidumbre y distingan estados operativos; curvas MAP aisladas no bastan. Train M1 y holdouts de transitorio/régimen/campaña y M2.

## 5. Caracterización de inyectores

**Observables y parámetros.** Masa entregada por pulso/tiempo, presión rail y manifold para Δp real, tensión y corriente en inyector, ancho efectivo, temperatura/densidad/viscosidad combustible; dispersión entre los cuatro inyectores. Identifica Q nominal vs Δp, offset/deadtime vs V/Δp y no linealidad de pulsos cortos, con región útil explícita.

**Fuentes/instrumentación.** Banco de inyección con fuente programable, driver registrado, balanza trazable, transductores absolutos y T. [Bosch MS 3 Sport](https://www.bosch-motorsport.com/media/downloads/ecu_ms_3_sport_en.pdf) distingue referencias de presión y offset por tensión. [Bosch EV14](https://www.bosch-motorsport.com/media/catalog_content/downloads_catalog/pdf_catalog/basic_information_67843467_injection_valve_ev_14.pdf) es documentación de otra familia: sus caudales no se transfieren a P2A-000.

**Adquisición y calidad.** Identificar número de parte/estado/lote de cada pieza; medir flujo estático y varios anchos/duties desde umbral de no apertura hasta régimen lineal, varias tensiones y Δp, repetir y alternar orden para controlar calentamiento. No utilizar cc/min sin Δp, fluido y T. Registrar comandos y corriente efectiva, tiempos, masas acumuladas y cálculo de masa por pulso.

**Incertidumbre, uso y suficiencia.** Pesaje pequeño/evaporación, presión dinámica, driver, transitorios y limpieza/variación de piezas confunden. Coste bajo–medio/dificultad media. Mínimo: trazas/matrices repetidas con incertidumbre suficientes para distinguir pendiente/offset y detectar no linealidad; reservadas condiciones V/Δp/ancho no usadas en ajuste. M2 debe usar sus inyectores caracterizados, no compartir un ajuste oculto que elimine variación independiente.

## 6. Presión, temperatura y propiedades del aceite

**Observables y parámetros.** Presión en galerías, RPM, aceite T en puntos conocidos, nivel, flujo/bomba si accesible, viscosidad vs T, densidad y composición/estado de uso. Datos p/Q/T restringen desplazamiento de bomba, relief y resistencias; viscosidad medida restringe ley térmica. Presión sola no identifica holgura, film_quality o coeficiente de desgaste. Vida útil exige tribología/mediciones longitudinales de superficies adicionales.

**Fuentes/instrumentación.** Transductores calibrados y T, curvas de bomba en banco si se desmonta, muestra de aceite analizada en laboratorio. [Catálogo técnico Shell](https://www.epc.shell.com/?Countryid=SG) ofrece propiedades declaradas de productos: referencia del producto exacto, no propiedades garantizadas del aceite usado ni presión D16. No se consultó un TDS específico para fijar números.

**Adquisición y calidad.** Identificar producto/lote/horas, medir nivel y contaminación/fuel dilution; series a frío/caliente con distintas RPM/cargas, sin alterar circuito no cuantificado. Obtener viscosidad/densidad en muestras reales y comparar con especificación; documentar puntos de presión, resolución y latencia. Para film considerar medición tribológica/contacto o proxy con incertidumbre explícita, nunca tratar presión como espesor medido.

**Incertidumbre, uso y suficiencia.** Aireación, temperatura local, relief, desgaste previo, geometría y aceite usado confunden. Coste medio/dificultad media–alta. Mínimo hidráulico: p/T/Q o información independiente de bomba/holguras con suficiente excitación y incertidumbre; mínimo de desgaste queda pendiente hasta materiales/film/contacto. Train M1, holdout térmico/aceite apropiado y M2; no extrapolar una marca/grado a todo aceite.

## 7. Curvas de calentamiento

**Observables y parámetros.** ECT, temperaturas reales coolant/wall/oil/IAT/ambiente, RPM/carga, consumo/fuel, calor indicado cuando disponible, estado termostato/ventilador, flujo refrigerante. Con entradas térmicas conocidas y varias condiciones restringe capacidades C, hA, reparto wall_frac y constantes de sensor. ECT sola mezcla sensor con física y no identifica cuatro nodos.

**Fuentes/instrumentación.** Registrador sincronizado y termopares/RTD independientes del sensor ECU, medidor de flujo cuando sea necesario; información de servicio [Honda](https://techinfo.honda.com/Rjanisis/pubs/Web/SvcExp_QS_CHE.pdf) orienta procedimientos y componentes, no curva cruda experimental. No se encontraron curvas D16Y7 instrumentadas con incertidumbre en las fuentes consultadas.

**Adquisición y calidad.** Definir soak inicial y registrar temperaturas de todos los nodos; repetir desde frío y caliente, a cargas conocidas, ambiente controlado o medido. Registrar fan/thermostat y calentamiento/enfriamiento para separar C/hA; no confundir «30 s estabilizado» con equilibrio térmico total. Medir flujo/consumo y mantener balance de energía con fronteras, incluido calor al ambiente (F10).

**Incertidumbre, uso y suficiencia.** Inercia de sensor, ubicación, mezcla coolant, flujo aire, fan y variación de combustible confunden. Coste bajo–medio/dificultad media. Mínimo: entradas y almacenamientos observados que den parámetros identificables con sensibilidad/rango; curva única ECT no basta. Train de unas condiciones, holdout de otra campaña/ambiente/carga y M2, con especificación clara del dominio de validación térmica.

## 8. Segundo motor independiente

**Observables y parámetros.** Identidad y metrología/condición de M2 más los observables de cada capacidad a validar. No se usa para ajustar los parámetros de M1 ni elegir estructura/umbral; comprueba transferencia y dispersión entre ejemplares. Si se ajusta un pack específico M2, esa campaña es calibración y se necesita otro holdout para validarla.

**Fuentes/instrumentación.** Segundo D16Y7 físico de variante documentada, conjunto de instrumentos y protocolos equivalentes, fechas/operadores registrados. Ningún motor de otra familia ni dataset público de otro SI sustituye M2. Coste medio–alto, dificultad logística alta; instrumentación especializada suma coste.

**Adquisición y calidad.** Reservar identidad/datos antes de calibrar; custodia/permiso de acceso separado, hash del dataset y release del modelo bloqueados. Caracterizar estado sin seleccionar el motor que mejor ajuste; documentar piezas/reparaciones y exclusiones predefinidas. Repetir ensayos suficientes en régimen/térmica/condición cubiertos por el claim, con mediciones y presupuesto comparables.

**Incertidumbre, uso y suficiencia.** Estado usado, tolerancias, sensores/driver, montaje, combustible y diferencias de variante confunden. Mínimo para torque es C1+fuel/air/contexto independiente; para combustión también p(θ); para aceite/térmico sus campañas; C1+C3 solos no validan todo el Core. Separar error de medición, variación de ejemplar y discrepancia de modelo. Dos motores permiten una falsación útil de transferencia; no certifican una población ni una probabilidad de fallo.

## Control de procedencia y criterio común de suficiencia

Cada dataset debe incluir archivos crudos preservados, identificador de motor/instrumento/calibración/campaña, fecha/operador, unidades, frecuencia y sincronización, incertidumbre (incluidas covarianzas comunes cuando importen), combustible/aceite/configuración/ambiente, protocolo y exclusiones fijadas antes de ajustar, licencia/derechos y hash SHA-256. Mantener tabla de transforms/filtros y versionar derivados. Un texto de provenance no vacío no satisface por sí solo estos campos (F15).

Antes de incorporar datos: verificar finitud y dominios físicos; comparar replicados y cierres de masa/energía por ventanas; estimar rango del Jacobiano, correlaciones y sensibilidad considerando incertidumbre; declarar parámetros no identificables. Reservar holdout por motor/campaña y dominio; no usarlo para elegir priors, criterios o correcciones sin reclasificarlo. Prefijar criterios de discrepancia compatibles con objetivo e incertidumbre, con intervalos por régimen, no accuracy universal. Si no se cumplen, evidencia insuficiente; no se fabrica un PASS.

## Registro de fuentes públicas consultadas

Todas accedidas 2026-10-10. No se suscribió, descargó contenido privado ni consultó texto íntegro SAE de pago. No se usaron precios actuales para estimar coste. Los costes/dificultades anteriores son juicios cualitativos de planificación.

| Fuente y URL | Clase / contenido consultado | Uso y límite |
|---|---|---|
| [Honda ServiceExpress guía](https://techinfo.honda.com/Rjanisis/pubs/Web/SvcExp_QS_CHE.pdf), [FAQ](https://techinfo.honda.com/rjanisis/pubs/web/RJAAI001_FAQ.htm) | OEM: acceso/documentación de servicio; no dataset de ensayo | Ruta para verificar variante/procedimientos; no dyno ni p(θ) independiente |
| [SAE J1349_202511](https://saemobilus.sae.org/standards/j1349_202511-engine-power-test-code-spark-ignition-compression-ignition-installed-net-power-torque-rating) | Organismo de estándar, metadata pública; revisión 2025-11-19 | Referencia de potencia neta; protocolo íntegro requiere acceso, no se atribuye certificación |
| [Kistler presión](https://www.kistler.com/DK/en/cylinder-pressure-measurement/C00000104), [TDC 2629D](https://www.kistler.com/INT/en/cp/top-dead-center-sensor-systems-2629d/P0001160) | Fabricante: método y sensor | Diseño de instrumentación, precisión del sistema real aún desconocida |
| [AVL TDC](https://www.avl.com/documents/10138/2075058/Solution%2BSheet_TDC%2BSensor), [catálogo sensores 2018](https://www.avl.com/documents/10138/2699442/Sensorenkatalog2018_web.pdf/b920e93c-47b8-458d-b51c-7c1320b38f32) | Fabricante: hoja/catálogo histórico | Sin garantía de stock/precio ni exactitud del montaje; no mediciones D16 |
| [Bosch MS 3 Sport manual](https://www.bosch-motorsport.com/media/downloads/ecu_ms_3_sport_en.pdf), [EV14](https://www.bosch-motorsport.com/media/catalog_content/downloads_catalog/pdf_catalog/basic_information_67843467_injection_valve_ev_14.pdf), [catálogo](https://www.bosch-motorsport.com/products/) | Fabricante: referencia de Δp/V/entrega y instrumentos | Familia genérica diferente de inyector/ECU OEM D16; no transferencia de parámetros |
| [Sandia ECN portal](https://ecn.sandia.gov/data/), [búsqueda gasolina](https://ecn.sandia.gov/gasoline-data-search/), [Spray G](https://ecn.sandia.gov/data/sandia-spray-g-data/), [tutorial SI](https://ecn.sandia.gov/tutorials/combustion-regime/spark-ignition/) | Institución de investigación: mediciones/datasets y condiciones de otros experimentos | Datos/protocolos de spray/óptica SI, incluidos DI muy distintos de PFI D16. Solo EXPLORATORY_PROXY o verificación de métodos compatibles; no se descargó dataset para ajustar el D16 |
| [Shell EPC](https://www.epc.shell.com/?Countryid=SG) | Fabricante: portal de fichas de producto | Propiedades de producto exacto pueden ser prior; muestra usada exige medición propia; no validación hidráulica |

La búsqueda fue acotada a estos portales y documentación primaria. No hallar un dataset adecuado aquí no demuestra que no exista en otros laboratorios. Cualquier dato futuro se clasificará OEM/tercero, medición/proxy y calibración/holdout antes de usarse. S31 permanece fuera de esta misión.
