# Motor integrado end-to-end — S16

## Acoplo

Bucle temporal: flujos S12 (orificios + Cd + nodos 2 especies) por
cilindro y fase S11, inyección S13 agrupada al IC con rail del plenum,
chispa S15 al cruce del avance, quemado S10 por reloj local, cigüeñal
S14 (τ de presiones vivas + fricción + starter S15 + accesorios).
Control open-loop (ηv 0.75, AFR 13.5, avance 15° fijos, declarados).

## Verificación (ilustrativo, 2026-10-10)

Punto ralentí (mariposa 0.05 + bypass 0.02 cm²): ~1026 rpm estables a
8 s; mariposa mayor ⇒ 1920/3102/5552 rpm (speed-density emergente).
no_spark/no_fuel ⇒ 0 rpm; fricción ×3 ⇒ 699 rpm; corte en marcha ⇒
decae. Masa ~1e−11/1e−14; energía 7e−5 (sano) / 1e−3 (arrastre).
Convergencia dt < 10%; snapshot bit-idéntico para las configuraciones/host/
horizonte ensayados (no es garantía universal; ver addendum R02 abajo).

## Remediación R01/R02 (2026-10-10)

- **Especies (R01-F01):** el combustible se advecta con la masa entre
  cilindros y nodos (plenum/escape), se reintegra en el reflujo y se
  contabiliza en la frontera ambiente. Inventario químico cerrado:
  `inyectado = almacenado + quemado + emitido`, con `0 ≤ m_fuel ≤ m_total`.
- **Historia de presión (R01-F07):** `pmax_pa` (máximo de presión de cilindro
  que alimenta la fricción de Chen–Flynn) se persiste en el snapshot; el
  replay dividido coincide con el continuo también con `c2 ≠ 0`.
- **Determinismo (R02):** bit-identidad demostrada en este host y horizonte,
  no universal; `docs/snapshot-contract.md` detalla la compatibilidad v1/v2 y
  la validación física fail-closed.

## Dominio y deuda

Sin lazo cerrado, sin AFR→quemado, sin multicilindro diferenciado,
sin periódico convergido multiciclo más allá de 8 s, sin arranque en
caliente/frío parametrizado, sin control de ralentí (S19), sin
diagnóstico (S20). Régimen reportado tal cual (gate S16).

## Cambios numéricos por la remediación

El transporte de especies y la persistencia de `pmax_pa` alteran la
trayectoria nominal (el combustible que sale/refluye cambia masa quemada y
el término de presión de fricción deja de reiniciarse). Los valores
nominales anteriores de RPM/trabajo ya no aplican; no se han reajustado
parámetros para preservarlos. Ver el reporte de batch R01–R02.

## Remediación R01-C1/RV01 y R02-C1/RV05 (2026-10-10)

- **Presupuesto químico (RV01):** la energía liberada por Wiebe consume el
  inventario químico en la **misma convención temporal** (`dm = dQ/(LHV·η)`).
  El remanente no quemado (completitud < 1) permanece en la cámara como
  especie transportable; no se suma calor liberado y combustible intacto como
  energías independientes. El ralentí nominal cambia ~1e-5 relativo por el
  orden de operaciones; sin reajuste de parámetros.
- **Fronteras y atomicidad (RV05):** las entradas se validan (áreas ≥ 0,
  Wiebe, fricción, volúmenes) y `resume` valida estado/ledgers y avanza sobre
  una copia, confirmándola solo si termina bien. Un error de fricción se
  propaga; no se convierte en torque cero. `restore` valida también historia
  de eventos y contadores.

## Remediación R02-C3/C2-01 y C2-02 (2026-10-10)

- **Frontera de leva/inyector:** `validate_inputs` valida los parámetros
  públicos activos (q_ref/p_ref/deadtime/incertidumbre y duración/at050/alzada/
  flanco/centerline/LSA de ambas levas). Una configuración inválida se rechaza
  en run/restore/resume; no se degrada a alzada o inyección cero.
- **Termodinámica derivada:** además de los operandos, se valida
  `T = U/(m·cv)` y presión/densidad derivadas en las fronteras, incluido el
  avance nulo, donde el bucle dinámico no llega.
