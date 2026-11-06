# D16SIM — Engine Engineering Sandbox

Laboratorio de ingeniería de motores en Rust. Incluye modelos reducidos L1,
cinemática, motor integrado, trazas deterministas, snapshots y herramientas
de diagnóstico y verificación numérica. El motor piloto es el Honda D16Y7
SOHC non-VTEC; los parámetros ilustrativos y proxies están identificados en
la documentación.

**Proyecto de investigación: no está calibrado ni validado experimentalmente
para predecir el comportamiento de un D16Y7 real.** Verificación de software,
calibración y validación experimental son actividades distintas.

## Estado publicado

S00–S30 permanecen congelados. Esta rama parte de la remediación R01–R02-C4
(`8c51b5d9c705745b1a558b00e8bc41d73aad0d7f`), cuya integración de desarrollo
fue aprobada por revisión independiente. La revisión registró 215 tests Rust
aprobados; sus conclusiones científicas se limitan al dominio ensayado.

Persisten limitaciones y defectos pendientes de etapas posteriores. La
publicación no inicia ni completa R03/R04/R05/S31 y no implica precisión OEM,
certificación, vida útil ni aptitud para decisiones de seguridad real.

## Ejecutar

El workspace declara Rust 1.99.0 como versión mínima.

```bash
git clone https://github.com/Franku83/d16sim.git
cd d16sim
cargo test --locked --workspace
cargo run --locked --bin d16sim -- --help
cargo run --locked --bin d16sim -- engine-run --scenario healthy --seconds 2
```

Otros comandos muestran mecanismos y protocolos independientes; su existencia
no demuestra que todos los subsistemas estén integrados en el motor runtime.

## Documentación

- [Foundation](docs/foundation.md): reglas normativas, alcance y evidencia.
- [Matriz de capacidades](docs/capability_matrix.md),
  [supuestos](docs/model_assumptions.md) y [limitaciones](docs/limitations.md).
- [Contratos de snapshots](docs/snapshot-contract.md) y
  [política de evidencia](docs/EVIDENCE_POLICY.md).
- [Reporte de implementación C4](docs/remediation/c4/C4_BATCH_REPORT.md).
- [Verificación independiente C4](https://github.com/Franku83/d16sim/blob/42c7e61fbb9510a4b96ae6e4b22457939ac2ee6a/docs/remediation/review/R01_R02_C4_INDEPENDENT_VERIFICATION.md),
  [gates e integración](https://github.com/Franku83/d16sim/blob/42c7e61fbb9510a4b96ae6e4b22457939ac2ee6a/docs/remediation/review/C4_GATE_DECISION.md) y
  [reproducción](https://github.com/Franku83/d16sim/blob/42c7e61fbb9510a4b96ae6e4b22457939ac2ee6a/docs/remediation/review/C4_REPRODUCTION.md).
- [Auditoría original S30](https://github.com/Franku83/d16sim/blob/25d528e3ff5b31eaeab0c894c4227b3e7d3a574c/docs/audit/INDEPENDENT_AUDIT_REPORT.md).
- Investigación: [D16Y7](D16Y7_RESEARCH_v0.1.md) y
  [motores donantes](D16Y7_RESEARCH_DONANTES_v0.1.md). Las referencias y
  deducciones de estos documentos no sustituyen mediciones independientes.

## Organización e historia

El workspace contiene 30 crates. `crates/engine` integra el motor;
`crates/kinematics`, `thermo`, `combustion`, `flow`, `fuel`, `crank` y otros
módulos contienen mecanismos y herramientas. `crates/cli` expone el runner
headless. Los contratos, ADR y reportes están en `docs/`.

La historia de revisión se conserva en ramas separadas:

| Rama | Contenido |
|---|---|
| `main` | C4 integrado y presentación pública del proyecto |
| `remediation/r01-r05` | Estado de implementación aprobado, sin el commit de presentación |
| `remediation/r01-r02-c1`…`c4` | Revisiones históricas de remediación |
| `verification/r01-r02-2026-10-10` | Pruebas, hallazgos y evidencia independiente |
| `audit/independent-2026-10-10` | Auditoría original del S30 congelado |
| `master` | Baseline histórico S30 |

Los reportes anteriores se conservan como evidencia histórica, incluidos sus
fallos y veredictos. En C4 el harness S30 literal obtuvo **21/32**; la copia
adaptada con autorización obtuvo **22/32**. El caso 14 literal no se declaró
aprobado. Los diez fallos restantes corresponden a R03/R04 y siguen pendientes.
La evidencia independiente reside en su propia rama y no se mezcla con el
código de producción por esta publicación.

## Licencia

El código propio se distribuye bajo [Apache License 2.0](LICENSE), la licencia
ya elegida para el proyecto. Consulta [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)
y [el estado de licencias](docs/LICENSE_STATUS.md) para dependencias y
materiales de terceros. Las referencias, marcas y materiales ajenos conservan
sus condiciones originales; el proyecto no redistribuye manuales OEM ni se
presenta como software oficial de Honda.
