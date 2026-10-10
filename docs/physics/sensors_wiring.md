# Sensores analógicos y cableado — S19

## Modelo

Canal: transferencia (lineal/NTC/narrowband) + Vref + retardo de filtro
(τ) + ruido seed + offset/deriva + saturación + ADC(cuantización).
Cable: R serie vs Zin + abierto/corto-masa/corto-alimentación/
intermitente con seed. Monitor de rango REF-* (3/2 persistencia).

## Verificación (2026-10-10)

Escalón al 63.2% en τ; ruido determinista por seed; NTC 2.5 V a 25 °C
con pull-up igual; O2 conmuta en λ=1; R 100 Ω/1 MΩ exacto; intermitente
reproducible; monitor confirma al 3.º y cura al 2.º. Batería S19:
signal/reading/action con ECT falso (enrich 1.00→1.25) y MAP
abierto/corto (carga 0.065→0.0/1.0 + monitor), físicos intactos.

## Dominio y deuda

Sin curvas OEM (transductores ilustrativos), sin heater/retardo de O2,
sin PWM de IACV, sin closed-loop, sin masa/corrosión de terminales,
sin diafonía. Ground truth y lectura: tipos disjuntos por diseño.
