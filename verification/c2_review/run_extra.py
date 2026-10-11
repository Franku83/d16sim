#!/usr/bin/env python3
"""Reconstruct extra C2 probes/producers from a tracked Git reference."""
import argparse,io,tarfile,subprocess,json,time,os
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
p=argparse.ArgumentParser();p.add_argument('--worktree',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--source-ref',default='HEAD');a=p.parse_args();w=a.worktree.resolve();o=a.output.resolve();h=w/'verification/c2_review'
def git(*args,cwd=ROOT):return subprocess.check_output(['git',*args],cwd=cwd)
assert git('rev-parse','HEAD',cwd=w).decode().strip()=='a0040c58174a6cf6a2ac84e38605eafd2f584159'
assert not git('diff','HEAD','--','crates','Cargo.toml','Cargo.lock',cwd=w)
source=git('rev-parse',a.source_ref).decode().strip();tarfile.open(fileobj=io.BytesIO(git('archive',source,'verification/c2_review'))).extractall(w,filter='data')
assert not o.exists() or not any(o.iterdir()),'never overwrite';o.mkdir(parents=True,exist_ok=True)
for ref,name in [('2541266','legacy'),('ff09a0d','s30'),('8a7e868','r02'),('caad1cb','c1'),('a0040c5','c2')]:
 d=w/'target'/('remediation-review-'+name);d.mkdir(parents=True,exist_ok=True);tarfile.open(fileobj=io.BytesIO(git('archive',ref))).extractall(d,filter='data')
commands=[]
def run(argv,label,env=None):
 start=time.monotonic()
 with (o/(label+'.jsonl')).open('w')as out,(o/(label+'.log')).open('w')as err:r=subprocess.run(argv,cwd=w,stdout=out,stderr=err,env=dict(os.environ,**(env or {})))
 commands.append({'argv':argv,'exit':r.returncode,'elapsed_s':time.monotonic()-start,'label':label,'env':env});(o/'commands.json').write_text(json.dumps(commands,indent=2)+'\n');return r.returncode
assert run(['python3',str(h/'run_history.py'),'--worktree',str(w),'--output',str(o/'producer-matrix')],'producer-driver')==0
assert run(['cargo','run','--locked','--offline','--release','--manifest-path',str(h/'producer_c1/Cargo.toml'),'--bin','previous-control'],'previous-c1-controls')==0
for mode,label in [('matrix','new-matrix'),('thermal','new-thermal'),('diagnostics','new-diagnostic'),('snapshots','new-snapshots'),('focused','focused')]:
 assert run(['cargo','run','--locked','--offline','--release','--manifest-path',str(h/'Cargo.toml'),'--bin','c2-boundaries','--',mode],label,{'D16_C2_FIXTURES':str(o/'producer-matrix')})==0
(o/'source.json').write_text(json.dumps({'source_tree_or_commit':source,'target':git('rev-parse','HEAD',cwd=w).decode().strip()},indent=2)+'\n')
raise SystemExit(run(['python3',str(h/'check_c2.py')],'new-contract-check',{'D16_C2_RESULTS':str(o)}))
