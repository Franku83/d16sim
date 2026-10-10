#!/usr/bin/env python3
"""Check package completeness, references and production isolation, not physics."""
import collections
import json
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'docs/audit'
BASE = json.loads((OUT / 'results/environment.json').read_text())['base_commit']
required_docs = ['INDEPENDENT_AUDIT_REPORT.md', 'FINDINGS.md', 'REPRODUCTION.md',
                 'DOCUMENTATION_CONSISTENCY.md', 'CAPABILITY_RISK_ASSESSMENT.md',
                 'S25_EVIDENCE_ACQUISITION_PLAN.md']
fields = ['id', 'category', 'severity', 'subsystem', 'location', 'claim', 'reproduction',
          'expected', 'observed', 'cause', 'impact', 'recommendation', 'regression', 'evidence']
items = json.loads((OUT / 'findings.json').read_text())
assert len({f['id'] for f in items}) == len(items)
assert all(all(f.get(k) for k in fields) for f in items)
counts = collections.Counter(f['severity'] for f in items if f['category'] == 'CONFIRMED_DEFECT')
assert dict(counts) == {'P0': 2, 'P1': 12, 'P2': 5}, counts
assert sum(f['category'] == 'DOCUMENTATION_DRIFT' for f in items) == 26
for name in required_docs:
    p = OUT / name
    assert p.is_file(), p
    content = p.read_text()
    for href in re.findall(r'\[[^\]]*\]\(([^)]+)\)', content):
        if href.startswith(('https://', 'http://')):
            continue
        path, _, anchor = href.partition('#')
        target = (p.parent / path).resolve() if path else p
        assert target.is_file(), (name, href)
        if anchor and '<a id=' in target.read_text():
            assert f'id="{anchor}"' in target.read_text(), (name, href)
    for match in re.finditer(r'`((?:crates|docs|\.github)/[^` ;]+\.(?:rs|md|yml)):(\d+)', content):
        target = ROOT / match[1]
        assert target.is_file(), (name, match[1])
        assert int(match[2]) <= len(target.read_text().splitlines()), (name, match[0])
summary = json.loads((OUT / 'results/independent-test-summary.json').read_text())
assert summary['test_cases'] == 32 and summary['failures'] == 23 and summary['errors'] == 0
repeat = json.loads((OUT / 'results/reproduction-comparison.json').read_text())
assert all(x['byte_identical'] for x in repeat['comparison'].values())
changed = subprocess.check_output(['git', 'diff', '--name-only', BASE, '--'], cwd=ROOT, text=True).splitlines()
production = [p for p in changed if not p.startswith(('audit/independent/', 'docs/audit/'))]
assert not production, production
original = ROOT.parent / 'd16sim'
main_head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=original, text=True).strip()
main_status = subprocess.check_output(['git', 'status', '--porcelain'], cwd=original, text=True)
assert main_head == BASE and not main_status
proof = {'base_commit': BASE, 'audit_branch': subprocess.check_output(['git', 'branch', '--show-current'], cwd=ROOT, text=True).strip(),
         'diff_scope': 'every tracked path against base, including staged and working changes',
         'production_changed_paths': production, 'allowed_new_paths': ['audit/independent/', 'docs/audit/'],
         'original_worktree': str(original), 'original_head': main_head, 'original_status': main_status,
         'complete_docs': required_docs, 'confirmed_by_severity': dict(counts), 'documentation_entries': 26,
         'finding_catalog_complete': True, 'meaning': 'package and isolation check; not Engine Core PASS'}
(OUT / 'results/production-isolation.json').write_text(json.dumps(proof, indent=2)+'\n')
print('Package complete; production unchanged; 19 confirmed findings and 26 documentary entries.')
