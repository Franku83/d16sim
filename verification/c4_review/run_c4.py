#!/usr/bin/env python3
"""Materialize a detached exact-C4 worktree and tracked verification sources.
No ignored fixture is required. All worktrees and red evidence are retained.
"""
import argparse, hashlib, io, json, os, subprocess, tarfile, time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
C1='8c51b5d9c705745b1a558b00e8bc41d73aad0d7f'
R02='8a7e8686e4e8831fb30d1a71b815e67e7d7139ef'
S30='ff09a0dda82188e6bf27a80d17c902f8972a39d2'
p=argparse.ArgumentParser();p.add_argument('--worktree',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--source-ref',default='HEAD');p.add_argument('--skip-baseline',action='store_true');p.add_argument('--prepare-only',action='store_true');a=p.parse_args()
def git(*argv,cwd=ROOT):return subprocess.check_output(['git',*argv],cwd=cwd)
W=a.worktree.resolve();OUT=a.output.resolve()
if W.exists():
    assert git('rev-parse','HEAD',cwd=W).decode().strip()==C1,'wrong target'
    assert not git('diff','HEAD',cwd=W),'tracked target modifications'
else:subprocess.run(['git','worktree','add','--detach',str(W),C1],cwd=ROOT,check=True)
# This archive contains only Git-tracked audit sources, never caches or ignored files.
source=git('rev-parse',a.source_ref).decode().strip()
archive=git('archive',source,'verification/c4_review')
assert archive, 'verification commit does not include harness'
tarfile.open(fileobj=io.BytesIO(archive)).extractall(W,filter='data')
H=W/'verification/c4_review'
PREVIOUS='1d614378dd8481acb2a513b4c6dc6572345e5c66'
AUDIT='25d528e3ff5b31eaeab0c894c4227b3e7d3a574c'
for filename,reference,path in [('check_contracts.py',AUDIT,'audit/independent/check_contracts.py'),('check_adversarial.py',PREVIOUS,'verification/remediation_review/check_adversarial.py'),('src/adversarial.rs',PREVIOUS,'verification/remediation_review/src/adversarial.rs'),('src/main.rs',PREVIOUS,'verification/remediation_review/src/main.rs'),('src/common.rs',PREVIOUS,'verification/remediation_review/src/common.rs')]:
    assert (H/filename).read_bytes()==git('show',reference+':'+path), 'oracle/fixture modified: '+filename
assert (H/'check_c1_controls.py').read_bytes()==git('show','a0088ff:verification/c1_review/check_c1_controls.py'),'C1 oracle changed'
assert not git('diff',C1,'--','crates','Cargo.toml','Cargo.lock',cwd=W)
assert OUT!=ROOT/'docs/remediation/review/results','historical output must be preserved'
assert not OUT.exists() or not any(OUT.iterdir()),'refuse to overwrite prior execution'
OUT.mkdir(parents=True,exist_ok=True)
(OUT/'source.json').write_text(json.dumps({'verification_source':source,'implementation':C1,'worktree':str(W)},indent=2)+'\n')
if a.prepare_only:raise SystemExit(0)
# Historical test result remains red because its ten R03/R04 cases are out of scope.
args=['python3',str(H/'reproduce.py'),'--output',str(OUT)]
if a.skip_baseline:args+=['--skip-baseline']
with (OUT/'reproduction-driver.log').open('w')as log:r=subprocess.run(args,cwd=W,stdout=log,stderr=subprocess.STDOUT)
assert r.returncode in (0,1) and (OUT/'execution-complete.json').exists(),'incomplete original reproduction'
for sha,name in [(S30,'remediation-review-s30'),(R02,'remediation-review-r02')]:
    target=W/'target'/name;target.mkdir(parents=True,exist_ok=True)
    tarfile.open(fileobj=io.BytesIO(git('archive',sha))).extractall(target,filter='data')
commands=[]
def run(argv,label,env=None):
    start=time.monotonic()
    with (OUT/(label+'.jsonl')).open('w')as stdout,(OUT/(label+'.log')).open('w')as stderr:
        q=subprocess.run(argv,cwd=W,env=dict(os.environ,**(env or {})),stdout=stdout,stderr=stderr)
    commands.append({'argv':argv,'cwd':str(W),'exit':q.returncode,'elapsed_s':time.monotonic()-start,'label':label});(OUT/'additional-commands.json').write_text(json.dumps(commands,indent=2)+'\n');return q.returncode
for directory,bin_name,arg,label in [('legacy','historical-cold-or-hot','cold','historical-v1-cold'),('legacy_v2','d16sim-historical-v2-producer','cold','historical-v2-cold'),('legacy_v2','d16sim-historical-v2-producer',None,'historical-v2-hot'),('r02_producer','d16sim-r02-native-producer',None,'historical-r02-native'),('r02_producer','r02-mid-burn',None,'historical-r02-mid-burn')]:
    argv=['cargo','run','--locked','--offline','--release','--manifest-path',str(H/directory/'Cargo.toml'),'--bin',bin_name]
    if arg:argv+=['--',arg]
    assert run(argv,label)==0,label
    (OUT/(label+'.json')).write_bytes((OUT/(label+'.jsonl')).read_bytes())
assert run(['cargo','run','--locked','--offline','--release','--manifest-path',str(H/'r02_producer/Cargo.toml'),'--bin','r02-boundary-control'],'controls-boundary-r02')==0
for mode in ['boundary','chemistry','analytic','diagnostic','migration','thermal','electronics','fit','dyno']:
    assert run(['cargo','run','--locked','--offline','--release','--manifest-path',str(H/'Cargo.toml'),'--bin','c1-controls','--',mode],'controls-'+mode,{'D16_REVIEW_RESULTS':str(OUT)})==0,mode
check=run(['python3',str(H/'check_c1_controls.py')],'c1-controls-check',{'D16_REVIEW_RESULTS':str(OUT)})
summary=json.loads((OUT/'adversarial-test-summary.json').read_text()); extra=json.loads((OUT/'c1-controls-summary.json').read_text())
print(json.dumps({'implementation':C1,'historical_adversarial':summary,'additional_controls':extra,'gate':'REQUIRES_INDEPENDENT_DECISION'},indent=2))
raise SystemExit(1 if check or summary['failures'] or summary['errors'] else 0)
