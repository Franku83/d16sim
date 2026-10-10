# Rutas de fuga generales — S23

## Formulación

Dominios gas/aceite/refrigerante + puertos orificio (gas, S12) o laminar
(`Q = k·Δp/μ`, explícito). Junta desde `SealContactState` (S08 encadenado:
torque→precarga→campo→conductancia); anillo desde gap×profundidad×Cd.
Suministro regulado a presión de prueba; respiradero PCV; refrigerante a
presión de tapón. Balances auditados con frontera.

## Verificación (leyes etiquetadas, 2026-10-10)

Masa/energía ~1e−15; reparto 1:3 por áreas; estrangulado satura;
incompresible ±2%; missing 16.8 kg/min > under 2.2 kg/min > anillo×2 >
sano (direcciones robustas); sensibilidad p_ref y Cd reportada, no
ajustada. Tres bugs reales: desigualdad invertida, Pa-como-kg (otra vez),
falta de respiradero; más rediseño a suministro regulado.

## Dominio y deuda

Leyes NO calibradas (magnitudes no predictivas); sin dinámica de anillo,
sin fretting térmico, sin aceite→exterior (retén S26), sin acople al
motor vivo (S24/S29), primer ciclo estático (sin compresímetro aún).

## Remediación R01 (2026-10-10)

- **Especies frescas (R01-F08):** el gas que cruza un puerto lleva la
  fracción fresca del dominio donante y se mezcla con el receptor
  (`fresh` receptora = masa fresca recibida / nueva masa). El suministro
  regulado entra fresco y el venteo sale con la fracción del nodo; se añade
  `fresh_closure` al ledger (≈0). Marca pasiva 1→0 verificada.
- **Unidades de líquidos (R01-F09):** el caudal laminar `k·Δp/μ` es
  volumétrico (m³/s); se convierte a masa con la densidad del fluido
  (`Fluid::liquid_density_kg_m3`: aceite 870, refrigerante 1060 kg/m³,
  ilustrativos). El gas no aplica densidad dos veces (su rama ya da kg/s).
