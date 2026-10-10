# Instrumentos mecánicos — S24

## Modelos (L1, NO calibrados)

Cranking: `pico = p_ideal·fill·V/(V+V_dead)/(1+C·A·t·n)` con C = 8e5
ilustrativa. Leak-down: `fuga% = 100·A/(A+A_orificio)`. Galgas: sesgo +
cuantización. Toda lectura lleva procedencia (instrumento, calibración,
operador, timestamp, condiciones).

## Verificación (2026-10-10)

WOT vs cerrada difieren (mismo motor, FIELD-07); dos galgas difieren
(mismo motor, FIELD-08); 1 medición inconclusa para 3 causas; 2
mediciones separan anillos/válvula/junta (4 tests). Demo cableada:
sano 1400 kPa/2.6 % (sin falta), anillo×2 bajo umbral (límite honesto),
subapretada colapsa con firma de anillos (ambigüedad documentada),
ausente a junta por burbujeo.

## Dominio y deuda

Sin causa válvula modelada en S23 (solo tests), sin curva de
compresímetro real, umbrales de taller sin calibrar, tercer test
(reapriete/química) prescrito pero no implementado.
