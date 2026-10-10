# Contrato de exportación de trazas (solo lectura) — Visual CAD Spike

Versión del contrato: `trace-bundle-v1`. Propósito: que herramientas
externas (spike visual/CAD) lean telemetría SIN tocar el Engine Core
(nunca Blender/UI dentro del core).

## Reglas

1. Solo lectura: el exportador es función pura de entradas+artefactos;
   jamás muta estado del motor ni añade física.
2. Versionado: todo bundle lleva `schema_version`; los consumidores lo
   exigen (`== 1`), sin adivinación.
3. Unidades SI explícitas por campo (`unit` junto al valor o sufijo
   `_si`). Tiempos en s, ángulos donde aplique en grados con sufijo.
4. Determinista: mismos inputs ⇒ mismos bytes (JSON canónico, claves
   ordenadas).
5. Sin datos OEM nuevos: solo reempaqueta lo ya calculado/etiquetado.

## Esquema `trace-bundle-v1`

```json
{
  "schema_version": 1,
  "producer": "d16sim-cli export-trace",
  "source": {"sprint": "S16", "scenario": "healthy", "inputs_hash": "<sha256>"},
  "series": {
    "t_s": {"unit": "s", "values": [...]},
    "rpm": {"unit": "rev/min", "values": [...]},
    "p_plen_kpa": {"unit": "kPa", "values": [...]},
    "p_exh_kpa": {"unit": "kPa", "values": [...]},
    "fuel_rate_g_s": {"unit": "g/s", "values": [...]}
  },
  "audit": {"mass_closure": 0.0, "energy_closure": 0.0, "...": "solo lectura"},
  "provenance": {"model_pack": "physics-pack-v0.x", "labels": ["ILLUSTRATIVE", "..."]}
}
```

Implementación: `d16sim export-trace --scenario healthy --seconds 6 --out
bundle.json` (decimado ×10 para el spike). El core no cambia.
