# ADR-020 — Inyección con comandado/entregado separados y sin DTC

- **Fecha:** 2026-10-10 · **Estado:** ACEPTADO (S13)
- **Decisiones:**
  1. `PulseCommand` vs `DeliveredMass` son tipos distintos; la entrega
     depende de ΔP real (rail − múltiple), deadtime(V), tensión y
     capacidad de bomba. `DeliveryStatus` sin variante de DTC.
  2. Q_ref 185 cc/min @3 bar = hipótesis de trabajo (claim sin ΔP);
     ley √ΔP ±2% [D]; deadtime analógico CBR600RR ±0.3 ms (no P2A-000).
     Zona < 2.0 ms efectivos marcada no lineal, sin adder inventado.
  3. Regulador ideal referenciado (ΔP cte); bomba apagada ⇒ rail 0
     (estado, no código). Estrategia de agrupación como entrada (S18 fija).
- **Trigger:** banco P2A-000 (Q(ΔP) + deadtime vs V) sustituye hipótesis;
  S18 conecta scheduler y monitores/DTC.
- **Refs:** Research v0.1 §3, donantes §3; Foundation §6.5/S13.
