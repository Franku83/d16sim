# Backlog documental — S00

Los sprints S01–S37 son el backlog. No se crean tickets de física en S00.
Orden secuencial; paralelizar solo trabajos sin interfaces inestables.
No saltar gates para mostrar progreso visual.

| Sprint | Objetivo (resumen) | Dep. | Estado |
|---|---|---|---|
| S00 | Charter, alcance, reglas repo | — | **APROBADO** |
| S01 | Variante + manual OEM + ficha maestra | S00 | **APROBADO** |
| S02 | Toolchain, CI, runner headless | S00 | **APROBADO** |
| S03 | Unidades, constantes, álgebra | S02 | **APROBADO** |
| S04 | Catálogo estructural e instancias | S01–S03 | **APROBADO** |
| S05 | Eventos, snapshots, replay | S02–S04 | **APROBADO** |
| S06 | Cinemática cigüeñal/pistón/biela | S03–S05 | **APROBADO** |
| S07 | Herramientas y contactos | S04,S06 | **APROBADO** |
| S08 | Pernos, precarga, junta | S01,S03–S05,S07 | **APROBADO** |
| S09 | Gas ideal / cilindro sellado | S03,S06 | **APROBADO** |
| S10 | Combustión reducida | S09 | **APROBADO** |
| S11 | Distribución SOHC | S06–S07,S09 | Pendiente |
| S12 | Flujos adm/esc 0D | S09,S11 | Pendiente |
| S13 | Combustible rail/inyector | S03,S10,S12 | Pendiente |
| S14–S22 | Motor operable + electrónica | (ver SPRINTS) | Pendiente |
| S23–S30 | Fallas emergentes + base auditada | (ver SPRINTS) | Pendiente |
| S31–S37 | Taller 3D y expansión (tras base) | — | Pendiente, sin competir con core |

Adquisiciones físicas priorizadas (research §7 v0.1, no bloquean S00):
perfil leva Y7, flow escape, banco inyectores P2A-000, compresión/leak-down
protocolizados, CKP/TDC/CYP en scope + datalog, dyno stock con protocolo,
p(θ). Ver research completos; no fijar como nominal ningún proxy.
