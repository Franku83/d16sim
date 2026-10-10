# Charter de producto científico — S00

Fuente normativa: `docs/foundation.md` §0.1. Este archivo no añade sprints
ni obliga a implementar módulo educativo/evaluador en S00.

## Job-to-be-Done

Permitir que una persona **comprenda, experimente y diagnostique** fenómenos
mecánicos, termodinámicos y electrónicos de motores de combustión mediante un
laboratorio virtual **reproducible, transparente, auditable y con dominio de
validez explícito**.

Ventaja frente al motor físico: **control contrafactual** — observar estados
internos, congelar/rebobinar, modificar una sola variable, mantener las demás
y repetir con el mismo estado inicial y semilla.

Propósito secundario: mejorar progresivamente la correlación cuantitativa con
motores reales. **No** sustituir mediciones, decisiones de tuning ni
diagnósticos sobre un motor físico concreto.

## Tres capacidades (prioridad: comprender → experimentar → diagnosticar)

1. **Comprender:** inspeccionar ecuaciones, estados internos, hipótesis y
   balances de masa/energía. Una representación L1 explicable y honestamente
   etiquetada puede ser útil aunque no prediga todos los detalles del D16Y7.
2. **Experimentar:** intervenciones controladas y contrafactuales (A/B,
   snapshot, replay, misma seed, una variable a la vez). Las demás variables
   quedan fijadas o su acoplamiento se declara explícitamente. No basta un
   slider que fuerce la salida.
3. **Diagnosticar:** formular hipótesis, seleccionar instrumentos, medir y
   discriminar causas distintas con síntomas semejantes, **solamente** dentro
   de los fallos cubiertos por el modelo. Sin ayudas automáticas en
   exploración libre; un evaluador **externo y opcional** puede contrastar
   diagnósticos contra la verdad de un *fixture* al finalizar un escenario,
   sin alterar el estado simulado ni imponerse como requisito del core.

## Condición de utilidad (por subsistema)

Un alcance es útil cuando sus relaciones son observables y explicables, sus
experimentos se reproducen, sus mediciones distinguen hipótesis pertinentes y
el sistema comunica explícitamente qué está verificado, extrapolado,
pendiente o no soportado. No exige completitud física ni predicción exacta;
tampoco permite declarar validez física por tener solo tests de software.
Mantener separados **verificación**, **validación** y **calibración**.

## Límites de alcance

- No es videojuego de misiones, ni calibrador certificado, ni herramienta de
  reparación sin verificación externa, ni modelo que inventa un único
  diagnóstico por síntoma.
- El usuario puede provocar fallos mediante estados físicos de ensamblaje;
  los resultados emergen de modelos e interacciones.
- Este JTBD arbitra decisiones, **no añade sprints**.

## Límite epistemológico

«Modelar todos los factores» requeriría multiescala con datos no disponibles.
**No es promesa honesta.** Sí podemos inventariar fenómenos relevantes,
asignarles representación explícita, declarar aproximaciones y ampliar
fidelidad sin alterar contratos. No confundir cobertura del estado con
exactitud predictiva.

## Declaración de no aptitud para seguridad real

Hasta validaciones apropiadas (S30), este software **no es apto** para:

- decisiones de tuning sobre un motor físico,
- diagnósticos o reparaciones sin verificación externa,
- certificación, homologación o uso safety-critical,
- predicción de roturas, knock dañino o vida útil.

Cada release publica su dominio de validez en `docs/limitations.md`.
Sin datos medidos, el hito BASE-REPRODUCIBLE es *base computacional validada
solo parcialmente*, y debe llamarse así.
