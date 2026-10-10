# Matriz de riesgos — S00

Derivada de `docs/foundation.md` §19.1. Sin cuantificación inventada;
severidad = juicio cualitativo para priorizar, no probabilidad calibrada.

| ID | Riesgo | Impacto | Mitigación / gate propietario |
|---|---|---|---|
| R-01 | Datos OEM incompletos | Simulación calibrada a supuestos | S01 identifica faltantes, limita predicción |
| R-02 | Falsa fidelidad | Confianza en curvas incorrectas | ModelCards + `EXTRAPOLATED` explícito |
| R-03 | Modelo complejo sin datos | Overfitting/indeterminación | L1 pequeño primero; fidelidad con evidencia |
| R-04 | Solvers inestables | Energía de la nada | Conservación + convergencia + error handling en CI |
| R-05 | Integración 3D temprana | Interfaces rígidas, hacks | Core headless como autoridad (Fase D después) |
| R-06 | Dependencias GPL/copyright | Bloqueo distribución | Inventario legal antes de reutilizar (ver EVIDENCE_POLICY §5) |
| R-07 | Agentes generan números inventados | Deuda científica camuflada | Provenance mandatory; «no datos → no conclusión» |
| R-08 | Acoplamientos excesivos | Desfase física, mal performance | Solver acoplado/adaptativo; profiling + V&V |
| R-09 | «Emergencia» con scripts | Experiencia falsa | Fault = estado físico, síntomas = ecuaciones |
| R-10 | Explosión combinatoria | Tests insuficientes | Propiedades + metamórficos + adversariales + seeds |
| R-11 | Dificultad injusta | Juego opaco | Instrumentación realista + manual + logs |
| R-12 | Motor usado desconocido | Desgaste arbitrario | Condición con distribuciones + credibilidad |
| R-13 | Mezclar versiones Honda | Wiring/sensores erróneos | `VariantId` + catálogo por año/ECU + test compatibilidad |
| R-14 | Realtime | Sacrificio oculto de fidelidad | Profiles certificados + benchmark + modo offline |

Riesgo S00 específico: declarar S00 terminado por existir un tipo/slider/test
contra valores fabricados. Mitigación: gate de comprensión + DoD documental.
