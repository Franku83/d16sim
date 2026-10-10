# Dyno auditable — S28

## Método

Freno virtual de carga declarada sobre el motor S16; por punto se
promedia la cola (último 20 %) y se separan P_ind/P_fric/P_aux/P_brake/
P_accel; corrección ambiente explícita (observado + corregido);
manifiesto con hash. Criterio Covered: residuo ≤ 10 %, deriva ≤ 2 %,
400–5500 rpm.

## Verificación (2026-10-10)

Identidad global < 1 %; más carga ⇒ menos rpm; corrección 1 en
estándar; determinismo por hash; punto carga-8 ⇒ Unsettled (rechazado).
Barrido demo: 1.67–2.88 kW al freno con cargas ligeras (ilustrativo).

## Dominio y deuda

Sin dyno real (C1 pendiente); corrección L1 no certificada; cola con
muestreo 20 ms (aliasing documentado); inestabilidad carga-8 candidata
a S29; mapa válido solo 400–5500 rpm.
