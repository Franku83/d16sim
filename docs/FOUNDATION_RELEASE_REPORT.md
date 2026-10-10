# Foundation Release Report — `foundation-core-0.x` (S30)

- **Fecha:** 2026-10-10 · **Base:** S00–S30 APROBADOS · **Estado:
  RESEARCH FOUNDATION, no modelo validado.**
- **Paquete:** headless reproducible (`cargo test --locked --workspace`,
  `cargo fmt --check`, Clippy 0 errores). Sin 3D obligatorio.

## 1. Cadenas causales (S30-T3, `release-check`)

| Cadena | Resultado medido |
|---|---|
| torque→sellado | 100 kN ⇒ 4.000e-4 m²; 200 kN ⇒ 0 (monótono exacto) |
| leva→compresión | calado +6° ⇒ ralentí 4455 → 4568 rpm (emergente) |
| CKP→chispa | borde saltado ⇒ confianza 1.000 → 0.875, rpm estimada 749.8 |
| anillo→blow-by | 50 h ⇒ gap 300.0 → 302.4 µm, área 2.700e-8 → 2.722e-8 m² |

## 2. Perfiles numéricos (S30-T2, 1 máquina × 3 pasos)

dt 25/50/100 µs ⇒ 4470/4454/4445 rpm (spread 0.6 %); masa ~1e-11;
energía 2.2e-5/4.5e-5/9.1e-5 (∝ dt, splitting primer orden S16).
**Segunda máquina: PENDING** (una disponible).

## 3. Benchmark (S30-T5)

Núcleo motor 2 s simulados en ~125 ms ⇒ **16× realtime** en esta
máquina (no certificado en hardware objetivo; dyno/sweeps más lentos
por diseño, no por FPS).

## 4. Licencias/fuentes/tests

Apache-2.0 código propio; terceros con sus términos; research intactos
(verificado); 30 crates, 37 ADRs (001–037 con Fase 0), H-S00–H-S30,
suites verdes por sprint + workspace completo.

## 5. Incertidumbre y validación pendiente (S30-T4)

Cero comparaciones reales (P1: 8 vacíos S25 intactos). Clasificación:
verificado-numéricamente / no-calibrado / parcial / pendiente-externo /
no-soportado en `docs/capability_matrix.md`. Revisión experta
independiente PENDING (requisito previo a cualquier etiqueta L2+).

## 6. Limitaciones auditadas

Ver `docs/limitations.md` + §Dominio de cada `docs/physics/*.md`:
proxy A6, trigger genérico, mapas REF-*, leyes ilustrativas con
sensibilidad declarada, blow-by motor parcial, inestabilidad carga-8
candidata, K efectivo a film 0.8, corrección dyno L1.

## 7. Versionado (S30-T6)

CHANGELOG con S00–S30; snapshots engine v2 / wear v1 con hash canónico
(`docs/snapshot-contract.md`); trazas `trace-bundle-v1`
(`docs/export-contract.md`).

## Veredicto S30

**PASS como research foundation incompleta pero honesta** (gate
Foundation: permitido). Sin etiqueta «predictivo» (sin evidencia).
Siguiente: S31 solo con autorización; reapertura de validación solo
con C1–C6+M2.
