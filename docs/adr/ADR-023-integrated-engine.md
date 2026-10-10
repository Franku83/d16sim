# ADR-023 — Motor integrado S16 sin clamp ni bootstrap

- **Fecha:** 2026-10-10 · **Estado:** ACEPTADO (S16)
- **Decisiones:**
  1. Splitting explícito (gas con (θ,ω) congelados + Euler simpléctico)
     con convergencia medida; sin RK4 pleno (documentado, no silencioso).
  2. Eventos por flanco con previo (`crossed_edge`): los contadores floor
     sobre ángulo periódico disparaban 2×/ciclo (bug corregido).
  3. Inyección agrupada al IC con deadtime real (rebanar el pulso lo
     anulaba: bug corregido). Control open-loop declarado (ηv fijo, AFR
     objetivo, avance fijo); AFR→quemado sin acoplar (limitación).
  4. Auditoría con Q_rel efectivo + ΔKE (dos bugs de auditoría: E_chem
     inyectada y ΔKE omitido; la física cerraba a 3e-5).
  5. Snapshot reanudable bit-idéntico (prev_local incluido).
- **Trigger:** lazo cerrado/VE aprendida (S18), AFR→quemado, gas exchange
  multicilindro periódico, implícito si la rigidez lo exige.
- **Refs:** Foundation §S16; sprints S06–S15 acoplados.
