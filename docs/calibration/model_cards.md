# ModelCards por subsistema — S25 (estado 2026-10-10)

Estado global: **NUMERICALLY_VERIFIED_ONLY**. Nada aquí está validado
contra el D16Y7 real (P1 bloqueante externo).

| Subsistema | Qué modela | Verificación | Calibración | Validación |
|---|---|---|---|---|
| mech (cranking/leak-down) | pico L1, % fuga L1 | tests S24 + self-fit exacto | maquinaria lista; 2 params identificables | PENDIENTE (C2/C3) |
| leak (S23) | red con suministro regulado | 4 tests, cierres 1e-15 | p_ref/Cd con priors | PENDIENTE (C3) |
| lube (S22) | galerías + película | 3 tests, continuidad exacta | disp/k_film con priors | PENDIENTE (C4) |
| thermal (S21) | 4 nodos + ECT | 10 tests, auditoría ramificada | wall_frac/hA con priors | PENDIENTE (C5) |
| combustion (S10) | Wiebe cerrado | orden 3.9, Q err 5e-8 | BLOQUEADA (sin p-θ) | PENDIENTE (C1/C7) |
| crank (S14) | dinámica + fricción | cierre 2.7e-8 | BLOQUEADA (sin motored) | PENDIENTE (C1) |
| fuel (S13) | comando/entrega | tests S13 | BLOQUEADA (sin banco) | PENDIENTE (C6) |
| ecu (S18) | P2E-like REF-* | tests S18 | BLOQUEADA (sin mapa medido) | PENDIENTE (C1) |

Advertencia: cualquier cifra de precisión (106 hp, curvas, consumos)
citada del simulador hoy es ilustrativa. El marketing de precisión está
bloqueado hasta el cierre de P1 con datos reales.
