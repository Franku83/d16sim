# Hardware CKP/TDC/CYP — S17

## Modelo

Reluctores 24/4/1 a velocidad de árbol (12/2/0.5 pulsos por rev de
cigüeñal, sin dientes faltantes); VR `v = ±S·ω·sin + ruido (seed)`;
acondicionador con umbral+histéresis; cableado (abierto/intermitente/
polaridad/gap); muestreo a tasa explícita; flancos interpolados.
Fases: TDC en PMS de encendido, CYP en cilindro 1 (+ offset de carcasa).

## Verificación (candidato, 2026-10-10)

24/4/1 flancos por ciclo; espaciado CKP 30°; TDC ±6° de los PMS;
CYP identificado ±20° (retardo documentado); borde saltado 23;
abierto 0; gap ×8 pierde a 100 rpm y recupera a 3000; offset 10°
desfasa los 3 canales; niveles ∝ ω.

## Dominio y deuda

Sin geometría/posición/pines OEM de la variante (scope + manual S01);
sensibilidad/umbral ilustrativos; régimen constante en muestreo
(transitorios S18); CKF no modelado; sin decoder (S18).
