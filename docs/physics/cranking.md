# Arranque arrastrado — S15

## Modelo

Batería `V = Voc(SOC,T) − I·R(SOC,T,masa)`; starter `τ/I vs ω` lineal +
reductora 12:1 + acoplo V/V0; compresión resistiva = traza motored S12
(dependencia de velocidad pendiente); fricción S14; bobina `I = Vt/L`,
`E = ½LI²`, chispa si `E ≥ 30 mJ` y dwell > 0. RK4 con asistencia
(`spin_with_assist`, S14 intacto).

## Verificación (ilustrativo, 2026-10-10)

Sana: 236.8 rpm medias, rizo 295 rpm por compresión, chispa 134 mJ,
cierre 2e−15. Descargada gira menos; masa abierta y starter ausente ≈ 0;
dwell 0 ⇒ sin chispa; con fuel+chispa ⇒ `started: false` (diseño).
Eficiencia ≤ 1 en todo el rango.

## Dominio y deuda

Sin integración con combustión (S16), sin gas exchange vivo a rpm de
arranque, sin temperatura de arranque en fricción (S21/S22), sin
transitorios de Bendix, parámetros eléctricos ilustrativos.
