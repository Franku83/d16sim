#!/usr/bin/env python3
"""Reconstruct all reference sources from Git, execute, preserve red contracts."""
import argparse, hashlib, io, json, os, subprocess, tarfile, time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
HERE=Path(__file__).resolve().parent
REMEDIATION='8c51b5d9c705745b1a558b00e8bc41d73aad0d7f'
S30='ff09a0dda82188e6bf27a80d17c902f8972a39d2'
AUDIT='25d528e3ff5b31eaeab0c894c4227b3e7d3a574c'
p=argparse.ArgumentParser();p.add_argument('--output',type=Path,default=ROOT/'docs/remediation/review/c4-results/reproduced');p.add_argument('--skip-baseline',action='store_true',help='Skip already verified original reference; does not certify baseline.')
a=p.parse_args(); OUT=a.output.resolve();OUT.mkdir(parents=True,exist_ok=True)
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT)
# New branch commits may add verification artifacts, but reviewed production must be exact.
assert not git('diff',REMEDIATION,'--','crates','Cargo.toml','Cargo.lock','docs/foundation.md','docs/snapshot-contract.md'), 'production differs from reviewed commit'
assert (HERE/'check_contracts.py').read_bytes()==git('show',AUDIT+':audit/independent/check_contracts.py'), 'original oracle changed'
original=git('show',AUDIT+':audit/independent/src/main.rs')
assert (HERE/'reference/original_main.rs').read_bytes()==original
adapted=original.replace(b'        thermal: i.thermal.map',b'        plen_fuel_kg: 0.0,\n        exh_fuel_kg: 0.0,\n        fuel_emitted_kg: 0.0,\n        pmax_pa: None,\n        thermal: i.thermal.map',1)
assert (HERE/'src/main.rs').read_bytes()==adapted, 'adapter differs beyond neutral fields'
cli=git('show',REMEDIATION+':crates/cli/src/main.rs').decode();start=cli.index('fn observable_chamber_split(');end=cli.index('\n/// Comando S24:',start)
assert (HERE/'src/observed_cli.rs').read_text()==cli[start:end]+'\n','production function extraction changed'
base_old=git('show',S30+':crates/engine/src/run.rs').decode();base_new=git('show',REMEDIATION+':crates/engine/src/run.rs').decode()
def nominal(s):return s[s.index('fn base_inputs('):s.index('/// Estado de trabajo mutable')]
assert nominal(base_old)==nominal(base_new),'nominal initializer changed'
checks={'oracle_sha256':hashlib.sha256((HERE/'check_contracts.py').read_bytes()).hexdigest(),'neutral_adapter_exact':True,'production_function_extraction_exact':True,'nominal_initializer_byte_identical':True,'reviewed_production':REMEDIATION}
(OUT/'integrity.json').write_text(json.dumps(checks,indent=2)+'\n')
commands=[]
def run(args,log,stdout=None,env=None,cwd=ROOT):
    t=time.monotonic();merged=dict(os.environ,**(env or {}))
    with (OUT/log).open('w') as err:
        if stdout:
            with (OUT/stdout).open('w') as out:r=subprocess.run(args,cwd=cwd,env=merged,stdout=out,stderr=err)
        else:r=subprocess.run(args,cwd=cwd,env=merged,stdout=err,stderr=subprocess.STDOUT)
    commands.append({'argv':[str(x) for x in args],'cwd':str(cwd),'exit':r.returncode,'elapsed_s':time.monotonic()-t,'stdout':stdout,'log':log,'env':env or {}})
    (OUT/'commands.json').write_text(json.dumps(commands,indent=2)+'\n')
    print(log, 'exit',r.returncode,flush=True);return r.returncode
for cmd,log in [(['cargo','test','--locked','--workspace'],'workspace-test.log'),(['cargo','fmt','--all','--','--check'],'format.log'),(['cargo','clippy','--locked','--workspace','--all-targets','--','-D','clippy::unwrap_used'],'clippy.log')]:
    assert run(cmd,log)==0,log
# Paths are generated build products, never required pre-existing ignored sources.
for commit,name in [('2541266','remediation-review-legacy'),(S30,'remediation-review-s30')]:
    if a.skip_baseline and commit==S30:continue
    target=ROOT/'target'/name;target.mkdir(parents=True,exist_ok=True)
    tarfile.open(fileobj=io.BytesIO(git('archive',commit))).extractall(target,filter='data')
assert run(['cargo','run','--locked','--offline','--release','--manifest-path',str(HERE/'legacy/Cargo.toml'),'--bin','d16sim-historical-v1-producer'],'historical-v1-build.log','historical-v1-hot.json')==0
for mode in ['engine','subsystems','dyno']:
    assert run(['cargo','run','--locked','--offline','--release','--manifest-path',str(HERE/'Cargo.toml'),'--bin','d16sim-independent-probes','--',mode],mode+'-build.log','probes-'+mode+'.jsonl')==0
original_exit=run(['python3',str(HERE/'check_contracts.py')],'contract-check.log',env={'D16_AUDIT_RESULTS':str(OUT)})
for mode in ['engine','leak','thermal','snapshot','boundary']:
    assert run(['cargo','run','--locked','--offline','--release','--manifest-path',str(HERE/'Cargo.toml'),'--bin','remediation-adversarial','--',mode],'adversarial-'+mode+'.log','adversarial-'+mode+'.jsonl',env={'D16_REVIEW_RESULTS':str(OUT)})==0
# Repetition tests determinism; no oracle or production equation changes.
for mode in ['engine','leak','thermal','snapshot','boundary']:
    assert run(['cargo','run','--locked','--offline','--release','--manifest-path',str(HERE/'Cargo.toml'),'--bin','remediation-adversarial','--',mode],'repeat-'+mode+'.log','repeat-'+mode+'.jsonl',env={'D16_REVIEW_RESULTS':str(OUT)})==0
    assert (OUT/('repeat-'+mode+'.jsonl')).read_bytes()==(OUT/('adversarial-'+mode+'.jsonl')).read_bytes(),mode+' nondeterministic'
new_exit=run(['python3',str(HERE/'check_adversarial.py')],'adversarial-check.log',env={'D16_REVIEW_RESULTS':str(OUT)})
for scenario in ['healthy_baseline','missing_head_gasket','underclamped_head','ring_leak_cyl3']:
    assert run(['cargo','run','--locked','--offline','--release','--bin','d16sim','--','mech-demo','--scenario',scenario],'cli-'+scenario+'.log')==0
if not a.skip_baseline:
    assert run(['cargo','test','--locked','--offline','--workspace'],'workspace-test-original.log',cwd=ROOT/'target/remediation-review-s30')==0
    B=OUT/'original';B.mkdir(exist_ok=True)
    for mode in ['engine','subsystems','dyno']:
        assert run(['cargo','run','--locked','--offline','--release','--manifest-path',str(HERE/'baseline/Cargo.toml'),'--',mode],'original-'+mode+'-build.log','original/probes-'+mode+'.jsonl')==0
    run(['python3',str(HERE/'check_contracts.py')],'original/contract-check.log',env={'D16_AUDIT_RESULTS':str(B)})
(OUT/'execution-complete.json').write_text(json.dumps({'reviewed_commit':REMEDIATION,'deterministic_modes':5,'baseline_executed':not a.skip_baseline,'original_contract_exit':original_exit,'new_contract_exit':new_exit,'gate':'NOT_DECIDED_BY_HARNESS'},indent=2)+'\n')
raise SystemExit(1 if original_exit or new_exit else 0)
