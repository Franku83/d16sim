# Bibliografía versionada — S01

Congelada 2026-10-10. URLs completas en `docs/foundation.md` §21/§23.14.
Ninguna entrada sustituye la auditoría de edición/página/variante en S01.

## Manuales / catálogos (candidatos OEM-TEXT/EPC)

- R01 Honda Civic 1996–2000 Service Manual (ManualsLib / Archive).
  Estado: REQUIRES_VERIFICATION edición/sección/página/variante.
- R02 PGM-FI CKP/TDC/CYP+CKF (ManualsLib pp.374/442–457).
  Estado: REQUIRES_VERIFICATION año/transmisión.
- CF01 compresión 250rpm WOT · CF02/CF03 aceite p.236 · CF04 bancada ·
  CF05 biela · CF06 lash · CF07 rail · CF08 cam-holder · CF15–CF20 EPC
  1997 DX 5MT. Estado: candidatos OEM-TEXT/EPC, edición por confirmar.

## Software / métodos

- R03/R03a Engine-Sim MIT (no científico) · R04/R04a Speeduino GPL-2.0
  (referencia, no copiar) · R05 Ardu-Stim · R06/R06a Cantera BSD-3 (ejemplo
  diesel ilustrativo) · R07 OpenWAM CMT-UPV (licencia por auditar; no
  confundir con homónimos IA) · R08 Haltech trigger (aftermarket) ·
  R09 ASME VVUQ · R10 NASA-STD-7009B · R11 ARP · R12 Fel-Pro · R13 Tauri ·
  R14/R16 NASA Glenn · R15 Garrett (turbo futuro) · R17 SAE J1349 (texto
  bajo licencia) · R18 Livengood-Wu · R19 Ng et al. 2020 · R20 NASA SWE-070.

## Research del repo (fuente primaria de esta síntesis)

- `D16Y7_RESEARCH_v0.1.md` (2026-10-09): flow admisión Bad Guys, tren
  válvulas, P2E/inyectores/encendido, priors Chen-Flynn/Woschni/Wiebe/
  Livengood-Wu, dyno 91.1 whp, compresión n≈1.13, fixtures §8.
- `D16Y7_RESEARCH_DONANTES_v0.1.md` (2026-10-09): leva A6 DIRECTA,
  escape P2A-06 I/E 0.77–0.82, Q∝√ΔP ±2%, deadtime analógico CBR600RR,
  BMEP familiar, IMEP cordura 11.5–13.0 bar.

## Campo / fabricantes (índice CF09–CF62)

- Casos M01–M13 (montaje/metrología), E01–E12 (encendido/ECU),
  T01–T08 (turbo/dyno, post-stock) + Motorservice/King/Fel-Pro/ARP.
  Clases FORUM-*/MFR-TECH según EVIDENCE_POLICY. Detalle y URLs en
  foundation §23.14 y `data/provenance_registry.yaml`.
