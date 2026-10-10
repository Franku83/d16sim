# ADR-030 — Fugas con ley explícita no calibrada + suministro regulado

- **Fecha:** 2026-10-10 · **Estado:** ACEPTADO (S23)
- **Decisiones:**
  1. Junta: `A = A_seg·max(0,1−p_c/p_ref)²` con `p_ref` = contacto a clamp
     completo calculado con S08 (ancla autoconsistente, NO ajuste). Forma
     ilustrativa, magnitud intermedia sin calibrar (S25: leak-down vs
     torque). Dirección exacta por construcción.
  2. Anillo: área geométrica × `CD_RING = 0.03` (laberinto real reduce
     10–100×; declarado pendiente S25). Sin Cd, el sano daría ~1 g/s.
  3. Protocolo de suministro regulado (como tester real): se mide caudal
     estacionario, no vaciado (el vaciado satura y oculta discriminación).
     Drenar un nodo es fail-closed, nunca salto silencioso.
  4. Severidades robustas (missing > under; anillo×2 > ×1) valen para toda
     calibración > 0; entre mecanismos solo se reporta lado a lado.
  5. Respiradero PCV modelado (sin él no hay estacionario); refrigerante a
     presión de tapón como sumidero regulado (declarado).
- **Trigger:** p_ref y Cd medidos (S25), dinámica de anillo, fretting
  dinámico de junta (S27), puerto aceite→exterior con retén (S26).
- **Refs:** Foundation S23; FIELD-21/22; CF26 (compresión sana + junta mala).
