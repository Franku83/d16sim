# Matriz de capacidades — S30 (lo que el Engine Core realmente puede)

Leyenda: ✅ verificado numéricamente · 🔧 implementado no calibrado ·
🟡 parcialmente integrado · ⏳ pendiente de medición externa ·
❌ no soportado. Nada es `EXPERIMENTALLY_VALIDATED` (P1 abierto).

| Capacidad | Estado | Evidencia / límite |
|---|---|---|
| Cinemática biela-manivela | ✅ | S06, 8 tests analíticos |
| Cilindro sellado / Wiebe / gas exchange / inyección | ✅ numérico | S09–S13; sin p-θ/banco (⏳ C1/C6/C7) |
| Dinámica cigüeñal + arranque + ralentí emergente | ✅ numérico | S14–S16; ralentí ~1026 rpm ilustrativo |
| Trigger GENERIC + ECU P2E-like + sensores | ✅ numérico | S17–S19; sin scope/mapa OEM (⏳) |
| Diagnóstico eléctrico sin panel secreto | ✅ | S20 |
| Térmico L1 + ECT emergente | 🔧 | S21; hA/C ilustrativos (⏳ C5) |
| Lubricación por red + película | 🔧 | S22; bomba/k sin medir (⏳ C4) |
| Fugas con ley explícita | 🔧 | S23; p_ref/Cd sin calibrar (⏳ C3) |
| Compresímetro/leak-down + protocolo 2 mediciones | ✅ (protocolo) | S24; causa válvula sin modelo |
| Fitter + sensibilidad + manifiesto | ✅ (maquinaria) | S25; **sin datos: NVO** |
| Desgaste persistente + servicios | 🔧 tendencias | S26; tribología ⏳ |
| Daño: PV riesgo, perno genérico, resto riesgo/❌ | 🟡 | S27; válvula/gripado/rosca ⏳/❌ |
| Dyno virtual + manifiesto | 🔧 | S28; dyno real ⏳ C1; Unsettled |
| Pares adversariales + MC | ✅ maquinaria | S29; blow-by motor 🟡 |
| Cadenas causales auditadas (4) | ✅ | S30 `release-check` |
| Potencia al freno predictiva / 106 hp | ❌ declarar | Bloqueado hasta C1+M2 |
| Vida útil / instantes de fallo / prob. fallo | ❌ declarar | Bloqueado siempre sin evidencia |
| CFD/FEA, 1D flujos, closed-loop, J(θ) medido | ❌ | Fuera de alcance L1 |

**Veredicto de uso:** base de investigación causal y numéricamente
verificada; NO predictiva del D16Y7 de fábrica; NO certificable.
