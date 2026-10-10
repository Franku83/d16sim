# ADR-033 — Desgaste persistente con leyes efectivas y soporte declarado

- **Fecha:** 2026-10-10 · **Estado:** ACEPTADO (S26)
- **Decisiones:**
  1. Slow-state por motor (gaps, camisa, 9 holguras, oxidación,
     relajación) con seed + hash + replay; servicios con registro.
  2. K de Archard EFECTIVO oiled (1e-8, con ×(1−film)): el K seco
     ingenuo daba metros/50 h (fallo intermedio documentado). Cojinete
     Kb 5e-18, aceite A 5e4/Ea 80 kJ, junta creep logarítmico — todo
     ilustrativo P1, salidas TREND_CONDITIONED.
  3. Creep integrado en forma cerrada: el Euler con 1/(1+t) dependía de
     la subdivisión (fallo real; el gate de escala temporal lo capturó).
  4. Test de subdivisión exige acuerdo 1e-9, no bits idénticos (fp
     acumula distinto con 50 vs 5000 sumas; documentado).
  5. Scuffing/fatiga/fretting = UNSUPPORTED sin números; demo acopla
     gap→fuga vía S23 sin banderas.
- **Trigger:** tribología medida, K/H/Ea reales, retén aceite→exterior.
- **Refs:** Foundation S26; FIELD-02/03/04/14; P1.
