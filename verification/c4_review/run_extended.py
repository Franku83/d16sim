#!/usr/bin/env python3
"""C4-only independent checks and authorized A; does not decide or execute integration."""
import argparse,subprocess,tarfile,io,json,os,time,hashlib,difflib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];p=argparse.ArgumentParser();p.add_argument('--worktree',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--literal-results',type=Path,required=True);p.add_argument('--source-ref',default='HEAD');a=p.parse_args();w=a.worktree.resolve();o=a.output.resolve();literal=a.literal_results.resolve()
def git(*args,cwd=ROOT):return subprocess.check_output(['git',*args],cwd=cwd)
assert git('rev-parse','HEAD',cwd=w).decode().strip()=='8c51b5d9c705745b1a558b00e8bc41d73aad0d7f'
assert not git('diff','HEAD','--','crates','Cargo.toml','Cargo.lock',cwd=w)
source=git('rev-parse',a.source_ref).decode().strip();tarfile.open(fileobj=io.BytesIO(git('archive',source,'verification/c4_review'))).extractall(w,filter='data');h=w/'verification/c4_review'
assert not o.exists() or not any(o.iterdir()),'no overwrite';o.mkdir(parents=True,exist_ok=True);(o/'adapted').mkdir();(o/'adapter').mkdir()
original=git('show','36919b0:verification/c3_review/src/main.rs');assert original==(h/'src/main.rs').read_bytes();needle=b'    v1.state.schema_version = 1;\n';assert original.count(needle)==1;derived=original.replace(needle,needle+b'    v1.model = None;\n',1);assert derived==(h/'src/main_option_a.rs').read_bytes()
for name,b in [('literal_main.rs',original),('derived_main.rs',derived)]:(o/'adapter'/name).write_bytes(b)
(o/'adapter/fixture.diff').write_text(''.join(difflib.unified_diff(original.decode().splitlines(True),derived.decode().splitlines(True),fromfile='literal_main.rs',tofile='derived_main.rs')))
(o/'adapter/sha256.json').write_text(json.dumps({'literal':hashlib.sha256(original).hexdigest(),'adapted':hashlib.sha256(derived).hexdigest(),'only_change':'v1.model = None;'},indent=2)+'\n')
commands=[]
def run(argv,label,env=None,output=None):
 start=time.monotonic();path=o/(output or(label+'.jsonl'));path.parent.mkdir(parents=True,exist_ok=True)
 with path.open('w')as out,(o/(label.replace('/','-')+'.log')).open('w')as err:r=subprocess.run(argv,cwd=w,stdout=out,stderr=err,env=dict(os.environ,**(env or {})))
 commands.append({'argv':argv,'cwd':str(w),'label':label,'exit':r.returncode,'elapsed_s':time.monotonic()-start,'env':env});(o/'commands.json').write_text(json.dumps(commands,indent=2)+'\n');return r.returncode
for mode in ['engine','subsystems','dyno']:assert run(['cargo','run','--locked','--offline','--release','--manifest-path',str(h/'Cargo.toml'),'--bin','s30-option-a','--',mode],'adapted-'+mode,output='adapted/probes-'+mode+'.jsonl')==0
assert run(['python3',str(h/'check_contracts.py')],'adapted-check',{'D16_AUDIT_RESULTS':str(o/'adapted')})==1
assert json.loads((o/'adapted/independent-test-summary.json').read_text())['failures']==10
assert json.loads((literal/'independent-test-summary.json').read_text())['failures']==11
changes=[]
for mode in ['engine','subsystems','dyno']:
 x=(literal/('probes-'+mode+'.jsonl')).read_text().splitlines();y=(o/'adapted'/('probes-'+mode+'.jsonl')).read_text().splitlines();assert len(x)==len(y)
 for p,q in zip(x,y):
  if p!=q:
   p,q=json.loads(p),json.loads(q);assert p['probe']==q['probe']=='v1_hash_migration' and not p['data']['accepted'] and q['data']['accepted'];old=dict(p['data']);p['data']['accepted']=True;assert p==q;changes.append({'mode':mode,'probe':q['probe'],'literal':old,'adapted':q['data']})
assert len(changes)==1;(o/'adapter/result_equivalence.json').write_text(json.dumps({'only_changed_observation':changes,'all_other_observations_byte_identical':True},indent=2)+'\n')
for mode in ['provenance','adapter','battery','nodes']:assert run(['cargo','run','--locked','--offline','--release','--manifest-path',str(h/'Cargo.toml'),'--bin','c4-directed','--',mode],mode)==0
(o/'source.json').write_text(json.dumps({'source_tree_or_commit':source,'target':git('rev-parse','HEAD',cwd=w).decode().strip(),'literal_results':str(literal)},indent=2)+'\n')
raise SystemExit(run(['python3',str(h/'check_c4.py')],'c4-check',{'D16_C4_RESULTS':str(o)}))
