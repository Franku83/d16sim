# ADR-024 — Trigger 24/4/1 GENERIC_TRIGGER + VR documentado

- **Fecha:** 2026-10-10 · **Estado:** ACEPTADO (S17)
- **Decisiones:**
  1. Patrón 24 CKP / 4 TDC / 1 CYP a mitad de velocidad como
     `GENERIC_TRIGGER` familiar (TRIG-2026-10-10: hondabase + rusEFI;
     TD-80U/98U sin confirmar; 16° de era OBD1). Imposible por
     construcción etiquetarlo OEM (un solo constructor de procedencia).
  2. VR L1 con amplitud ∝ ω, ruido determinista, histéresis; polaridad
     modelada (drift documentado, no calibrado); gap multiplica.
     Sensibilidad/umbral ilustrativos de orden VR real.
  3. Retardo de umbral fijo dependiente de amplitud (TDC ~3.7°, CYP ~15°):
     comportamiento documentado, no error de fase.
  4. CKF registrado como concepto no modelado (presencia PENDING).
- **Trigger:** scope en motor + página OEM fijan geometría/fase/pines.
- **Refs:** research/raw/trigger-distributor-2026-10-10.md; S01 C-19/C-20.
