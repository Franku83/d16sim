#!/usr/bin/env python3
"""Render the audit catalog; does not touch production documents."""
import json
from pathlib import Path
root = Path(__file__).resolve().parents[2]
items = json.loads((root / 'docs/audit/findings.json').read_text())
out = ['# Hallazgos independientes', '', 'Base auditada: `ff09a0dda82188e6bf27a80d17c902f8972a39d2`. Fecha: 2026-10-10.', '',
       'Estas categorías son etiquetas de auditoría, no estados científicos nuevos. La severidad representa el impacto en los contratos y en la inferencia, no el esfuerzo de reparación. P0 compromete una cadena científica central; P1 afecta resultados, persistencia o garantías; P2 afecta escenarios acotados o controles operativos; P3 afecta claridad o referencias.', '',
       'Los fixtures numéricos son `SYNTHETIC_SELFTEST`: sirven para falsar contratos y verificar software; ninguno es evidencia experimental del D16Y7. Los errores esperados permanecen rojos en la batería. Reproducción común: [REPRODUCTION.md](REPRODUCTION.md).', '',
       '| ID | Categoría | Severidad | Hallazgo |', '|---|---|---|---|']
for f in items:
    out.append(f"| [{f['id']}](#{f['id'].lower()}) | {f['category']} | {f['severity']} | {f['title']} |")
for f in items:
    out += ['', f"<a id=\"{f['id'].lower()}\"></a>", '', f"## {f['id']} — {f['title']}", '']
    for key, label in [('category','Categoría'),('severity','Severidad'),('subsystem','Subsistema'),('location','Archivo, símbolo y líneas'),('claim','Afirmación o gate comprometido'),('reproduction','Método de reproducción'),('expected','Resultado esperado'),('observed','Resultado observado'),('cause','Causa raíz o hipótesis causal'),('impact','Alcance del impacto'),('recommendation','Recomendación'),('regression','Regresión propuesta'),('evidence','Estado de evidencia')]:
        out += [f"**{label}:** {f[key]}", '']
(root / 'docs/audit/FINDINGS.md').write_text('\n'.join(out).rstrip()+'\n')
