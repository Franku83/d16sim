#!/usr/bin/env python3
"""Generate genuine type/solver snapshots from archived Git producers; never overwrite."""
import argparse,subprocess,json,time
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--worktree',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();h=a.worktree/'verification/c2_review';o=a.output;o.mkdir(parents=True,exist_ok=False);commands=[]
for label in ['v1','s30','r02','c1','c2']:
 manifest=h/('producer_'+label)/'Cargo.toml'
 assert (manifest.parent/'Cargo.lock').exists(),'tracked lockfile required'
 for stage,t in [('cold',0),('pre',.2),('mid',.35955),('post',.38),('hot',.5)]:
  argv=['cargo','run','--locked','--offline','--release','--manifest-path',str(manifest),'--bin','d16sim-c2-history-'+label,'--',str(t)];start=time.monotonic()
  with (o/f'{label}-{stage}.json').open('w')as out,(o/f'{label}-{stage}.log').open('w')as err:r=subprocess.run(argv,cwd=a.worktree,stdout=out,stderr=err)
  commands.append({'producer':label,'stage':stage,'argv':argv,'exit':r.returncode,'elapsed_s':time.monotonic()-start});assert r.returncode==0
(o/'commands.txt').write_text(json.dumps(commands,indent=2)+'\n')
