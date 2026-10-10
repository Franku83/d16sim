#!/usr/bin/env python3
"""Read-only structural inventory, counts do not imply physical coverage."""
import collections
import hashlib
import json
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "docs/audit/results"
meta = json.loads((OUT / "cargo-metadata.json").read_text())
packages = meta["packages"]
local = {p["name"]: p for p in packages}
deps = {name: sorted(d["name"] for d in p["dependencies"] if d["name"] in local) for name, p in local.items()}
visited, active, cycles = set(), [], []


def visit(n):
    if n in active:
        cycles.append(active[active.index(n):]+[n])
    elif n not in visited:
        active.append(n)
        for d in deps[n]:
            visit(d)
        active.pop()
        visited.add(n)


for n in deps:
    visit(n)
log = (OUT / "workspace-test.log").read_text()
adr = sorted(p.name for p in (ROOT / "docs/adr").glob("ADR-*.md"))
declared = re.findall(r"`(ADR-[^`]+\.md)`", (ROOT / "docs/adr/README.md").read_text())
rows = [json.loads(line) for name in ["probes-engine.jsonl", "probes-subsystems.jsonl", "probes-dyno.jsonl"]
        for line in (OUT / name).read_text().splitlines()]
result = {
    "base_commit": json.loads((OUT / "environment.json").read_text())["base_commit"],
    "workspace_crates": sorted(local), "crate_count": len(local),
    "rust_source_files": sorted(str(p.relative_to(ROOT)) for p in (ROOT / "crates").rglob("*.rs")),
    "local_dependency_graph": deps, "dependency_cycles": cycles,
    "adr_count": len(adr), "adr_files": adr,
    "adr_declared_nonexistent": [x for x in declared if x not in adr],
    "adr_missing_from_index": [x for x in adr if x not in declared],
    "sprint_files": sorted(p.name for p in (ROOT / "docs/sprints").glob("S*.md")),
    "historical_handoff_briefs": re.findall(r"^## Brief (S\d+-histórico).*$", (ROOT / "docs/HANDOFF.md").read_text(), re.M),
    "baseline_unit_targets": log.count("Running unittests"),
    "baseline_doc_targets": log.count("Doc-tests"),
    "baseline_passed_cases": sum(map(int, re.findall(r"test result: ok\. (\d+) passed", log))),
    "source_assertion_sites_not_execution_count": sum(len(re.findall(r"\bassert(?:_eq|_ne)?!\s*\(", p.read_text())) for p in (ROOT / "crates").rglob("*.rs")),
    "probe_rows": len(rows), "probe_families": dict(sorted(collections.Counter(r["probe"] for r in rows).items())),
    "production_diff": subprocess.check_output(["git", "diff", "HEAD", "--", "crates", "Cargo.toml", "Cargo.lock", "docs/foundation.md", "docs/SPRINTS.md", "docs/PROGRESS.md", "docs/HANDOFF.md"], cwd=ROOT, text=True),
    "root_release_report_exists": (ROOT / "FOUNDATION_RELEASE_REPORT.md").exists(),
    "actual_release_report": "docs/FOUNDATION_RELEASE_REPORT.md",
    "readme_parent_research_paths_exist": [(ROOT.parent / f).exists() for f in ["D16Y7_RESEARCH_v0.1.md", "D16Y7_RESEARCH_DONANTES_v0.1.md"]],
}
(OUT / "inventory.json").write_text(json.dumps(result, indent=2, ensure_ascii=False)+"\n")
print(json.dumps({k: result[k] for k in ["crate_count", "adr_count", "dependency_cycles", "baseline_unit_targets", "baseline_doc_targets", "baseline_passed_cases", "probe_rows"]}))
