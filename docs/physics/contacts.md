# Contactos y herramientas — S07

Prototipo geométrico sin modelo material. Ecuaciones en
`crates/contacts/src/interference.rs`.

## Modelos

- Pistón–válvula coaxial: `clearance = gap0 − lift·cos(cant) −
  (Dv/2)·sin(cant) + x(θ)`. Con corona plana el offset lateral no cambia
  el huelgo axial (documentado); el ángulo de válvula sí (borde bajo).
  Alzada impuesta (perfil real en S11).
- Biela–bloque L0: `huelgo_min = pared − (r + semiancho)`.
- Rosca: paso coincidente + macho ≤ hembra, si no `Impossible`.
- Medición L1: `lectura − cero`, incertidumbre resolución/2 + térmica.

## Contrato de no-daño (gate)

`ContactReport.needs_material_model` siempre true; `damage` siempre
`NotAssessed`, incluso con 10 mm de penetración (testeado). Las
herramientas aplican magnitudes; el daño lo decidirá S27 con criterio.

## Dominio y deuda

Geometría simplificada coaxial/simétrica; sin deformación térmica,
sin dinámica de válvula (S11), sin asiento/holgura térmica, sin material.
`mates_with` de S04 sigue nominal.
