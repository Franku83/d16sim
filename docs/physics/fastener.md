# Pernos, precarga y junta L1 — S08

## Ecuaciones

- Precarga: `F = T/(K·d)`; `F_lo` con `K_hi`, `F_hi` con `K_lo`,
  `F_nom` con `K` medio. Rango ilustrativo: seco 0.18–0.22, aceitado
  0.13–0.17, anti-gripante 0.10–0.14 (literatura genérica, NO OEM).
- Utilización a fluencia (solo con área+límite del llamante):
  `u = (F/A)/Sy`; `u_hi > 1` es estado, no rotura.
- Junta: `p_i = F_total·(w_i/Σw)/A_i`; no-uniformidad `max/min`;
  estado siempre `SealingIndeterminate` (S23 decide con ley de fuga).

## Verificación

- Mismo torque + distinto lubricante ⇒ distinta F (25.0 vs 33.3 kN a
  50 N·m/d=10 mm, demo arbitraria).
- Barrido monótono; `F_hi/F_lo = K_hi/K_lo` exacto.
- Paso por ángulo sin torque: rechazado; subapriete ⇒ no-uniformidad
  observada, no distorsión impuesta.

## Dominio y deuda

L1 sin memoria de trayectoria ni interacción elástica; sin temperatura de
junta, sin fluencia transitoria, sin fuga (S23). Secuencia OEM solo con
página verificada (S01 pendiente). Calibración pendiente declarada.
