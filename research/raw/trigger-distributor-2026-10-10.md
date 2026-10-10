# Trigger distribuidor Honda — reporte bruto 2026-10-10

Pregunta: dientes CKP/TDC/CYP, velocidad del distribuidor y CKF para la
variante `US_1997_CIVIC_DX_2DR_5MT_OBD2A_P2E_FAMILY` (TD-80U vs TD-98U).

## Fuente A (leída íntegra)

- URL: https://www.hondabase.com/cars/electronics/honda-dbh-distributor-trigger-decoding
- Fecha pub: 2026-08-09 (actualización); consultada 2026-10-10.
- Autor: VIRUXE (contribuidor comunitario); adaptado de 3 hilos rusEFI.
- Afirma: 3 sensores VR en distribuidor D/B/H OBD0/OBD1/OBD2; CKP 24
  dientes (12 pulsos/rev cigüeñal); TDC 4 dientes (2 pulsos/rev, pares
  1/4 y 2/3); CYP 1 diente (0.5 pulsos/rev, sinc secuencial); giro a
  mitad de velocidad del cigüeñal; modos 24+1 (recomendado) y 4+1.
- Taxonomía: incluye Civic 6ª gen EK 1996–2000 y familia D-series.
- NO nombra TD-80U/TD-98U (cita TD-41U/42U/44U/68U...). Referencia de
  calado 16° BTDC (marca roja OBD1) ≠ base Y7 12° (diferencia de variante).
- CKF no cubierto.
- Clase: `FORUM-CLAIM` (conocimiento comunitario adaptado de foros
  aftermarket; taxonomía EK/D-series como indicio, no prueba por variante).

## Fuente B (primaria de A, leída íntegra)

- URL: https://rusefi.com/forum/viewtopic.php?t=2013 (blundar, 2021-05-12).
- Afirma: 88–01 D/B/H/F con 3 VR; CKP 8 o 12 pulsos/rev cigüeñal (16/24
  dientes a velocidad de árbol; 95+ usa 24, 88–91 usa 16); TDC 4 dientes
  (2 pulsos/rev); CYP/"snail" 1 por rev de árbol.
- CKF (segundo post): añadido en ALGUNOS OBD2 mercado USA; misma forma de
  onda que CKP; VR sobre rueda de 12 dientes en la BOMBA DE ACEITE
  (confirmado en serie B; H tardía similar con distribuidor de solo
  CYP+TDC). Propósito: ayuda a detección de misfire.
- Clase: `FORUM-CASE`/`FORUM-CLAIM` (experiencia de practicante con
  standalone; útil como hipótesis de arquitectura, no spec OEM).

## Fuente C (no leída: 403)

- URL: https://www.carmanualsonline.info/honda-civic-1997-6-g-workshop-manual/?srch=ignition+timing
- Estado 2026-10-10: HTTP 403 (protección bot). NO citar como verificada.
- Búsquedas web de corroboración TD-80U + dientes: sin resultados.

## Evaluación para la variante (S17)

| Candidato | Valor | Soporte | Estado |
|---|---|---|---|
| CKP 24 dientes @ ½ velocidad cigüeñal | 12 pulsos/rev | Familia 95+ (A+B), EK en taxonomía | CANDIDATO fuerte, NO OEM |
| TDC 4 dientes (pares 1/4, 2/3) | 2 pulsos/rev | A+B | CANDIDATO, NO OEM |
| CYP 1 diente (sinc secuencial) | 0.5 pulsos/rev | A+B | CANDIDATO, NO OEM |
| Distribuidor a mitad de velocidad | — | Arquitectura (arrastre por árbol) [D] | CANDIDATO por mecanismo |
| CKF separado junto al cigüeñal/bomba | rueda 12 dientes, solo algunos OBD2 USA | B (serie B confirmada; D sin confirmar) | PENDING presencia en 1997 DX |
| TD-80U vs TD-98U: ¿cuál trae qué? | — | Sin fuente | DISPUTED (S01) |
| Base timing 12° vs 16° | 12° Y7 (research) | Conflicto de variante registrado | PENDING edición OEM |

Decisión: el patrón 24/4/1 entra a S17 como `GENERIC_TRIGGER` de familia
(útil para decoder/scheduler de referencia), prohibido etiquetarlo OEM de
la variante hasta osciloscopio + página de manual. CKF: concepto aparte
registrado, presencia PENDING.
