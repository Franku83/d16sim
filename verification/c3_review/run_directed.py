#!/usr/bin/env python3
"""Execute C3-only root-cause experiments from tracked Git sources."""
import argparse,subprocess,tarfile,io,time,os,json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];p=argparse.ArgumentParser();p.add_argument('--worktree',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--source-ref',default='HEAD');a=p.parse_args();w=a.worktree.resolve();o=a.output.resolve()
def git(*args,cwd=ROOT):return subprocess.check_output(['git',*args],cwd=cwd)
assert git('rev-parse','HEAD',cwd=w).decode().strip()=='3aef5f6ccdc076dad6e8be627a5eb836b26ee998'
assert not git('diff','HEAD','--','crates','Cargo.toml','Cargo.lock',cwd=w)
d=w/'target/remediation-review-c2';d.mkdir(parents=True,exist_ok=True);tarfile.open(fileobj=io.BytesIO(git('archive','a0040c58174a6cf6a2ac84e38605eafd2f584159'))).extractall(d,filter='data')
source=git('rev-parse',a.source_ref).decode().strip();tarfile.open(fileobj=io.BytesIO(git('archive',source,'verification/c3_review'))).extractall(w,filter='data');h=w/'verification/c3_review'
assert not o.exists() or not any(o.iterdir()),'refuse overwrite';o.mkdir(parents=True,exist_ok=True);commands=[]
for mode in ['fields','thermo','thermal','battery','replay','semantics']:
 argv=['cargo','run','--locked','--offline','--release','--manifest-path',str(h/'Cargo.toml'),'--bin','c3-directed','--',mode];start=time.monotonic()
 with (o/(mode+'.jsonl')).open('w')as out,(o/(mode+'.log')).open('w')as err:r=subprocess.run(argv,cwd=w,stdout=out,stderr=err)
 commands.append({'argv':argv,'cwd':str(w),'exit':r.returncode,'elapsed_s':time.monotonic()-start,'mode':mode});(o/'commands.json').write_text(json.dumps(commands,indent=2)+'\n');assert r.returncode==0
# Counterfactuals use the exact same probe source against archived C2 crates.
for mode in ['thermo','battery','semantics','replay']:
 argv=['cargo','run','--locked','--offline','--release','--manifest-path',str(h/'producer_c2/Cargo.toml'),'--bin','c2-directed-control','--',mode]
 start=time.monotonic()
 with (o/('c2-control-'+mode+'.jsonl')).open('w')as out,(o/('c2-control-'+mode+'.log')).open('w')as err:r=subprocess.run(argv,cwd=w,stdout=out,stderr=err)
 commands.append({'argv':argv,'cwd':str(w),'exit':r.returncode,'elapsed_s':time.monotonic()-start,'mode':'c2-control-'+mode});(o/'commands.json').write_text(json.dumps(commands,indent=2)+'\n');assert r.returncode==0
(o/'source.json').write_text(json.dumps({'verification_source':source,'target':git('rev-parse','HEAD',cwd=w).decode().strip()},indent=2)+'\n')
with (o/'check.log').open('w')as log:r=subprocess.run(['python3',str(h/'check_c3.py')],env=dict(os.environ,D16_C3_RESULTS=str(o)),cwd=w,stdout=log,stderr=subprocess.STDOUT)
raise SystemExit(r.returncode)
