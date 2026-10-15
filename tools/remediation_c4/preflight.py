#!/usr/bin/env python3
"""Reproduce unchanged C3 and its fixture conflict; no production fix or policy decision."""
import argparse,subprocess,tarfile,io,hashlib,json,os,time,platform
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
BASE='3aef5f6ccdc076dad6e8be627a5eb836b26ee998'
REVIEW='36919b0455e8eaf741a72ff4a74ae85f40e116ba'
p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);p.add_argument('--archive',type=Path,required=True);a=p.parse_args();o=a.output.resolve();w=a.archive.resolve()
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT)
assert not git('diff',BASE,'--','crates','Cargo.toml','Cargo.lock'),'preflight requires unchanged C3 production'
assert not o.exists() or not any(o.iterdir()),'never overwrite logs';assert not w.exists()or not any(w.iterdir()),'archive must be new'
o.mkdir(parents=True,exist_ok=True);w.mkdir(parents=True,exist_ok=True)
for ref,paths in [(BASE,[]),(REVIEW,['verification/c3_review'])]:
 tarfile.open(fileobj=io.BytesIO(git('archive',ref,*paths))).extractall(w,filter='data')
h=w/'verification/c3_review';checks=[]
for file in ['check_contracts.py','check_adversarial.py','check_c1_controls.py','check_c2.py','check_c3.py','src/main.rs','src/c3_directed.rs','src/common.rs']:
 b=(h/file).read_bytes();ref=git('show',REVIEW+':verification/c3_review/'+file);assert b==ref
 checks.append({'path':file,'sha256':hashlib.sha256(b).hexdigest(),'byte_identical':True})
(o/'integrity.json').write_text(json.dumps({'base':BASE,'review':REVIEW,'files':checks},indent=2)+'\n')
(o/'environment.json').write_text(json.dumps({'date':'2026-10-15','platform':platform.platform(),'rustc':subprocess.check_output(['rustc','--version'],text=True).strip(),'cargo':subprocess.check_output(['cargo','--version'],text=True).strip()},indent=2)+'\n')
# New bin only in the generated audit manifest; Core and original probes remain untouched.
(h/'src/c4_schema_conflict.rs').write_bytes((ROOT/'tools/remediation_c4/schema_conflict.rs').read_bytes())
with(h/'Cargo.toml').open('a')as f:f.write('\n[[bin]]\nname="c4-schema-conflict"\npath="src/c4_schema_conflict.rs"\n')
commands=[]
def run(argv,name,env=None):
 start=time.monotonic()
 with(o/(name+'.jsonl')).open('w')as out,(o/(name+'.log')).open('w')as err:r=subprocess.run(argv,cwd=w,stdout=out,stderr=err,env=dict(os.environ,**(env or {})))
 commands.append({'argv':argv,'cwd':str(w),'exit':r.returncode,'elapsed_s':time.monotonic()-start,'name':name});(o/'commands.json').write_text(json.dumps(commands,indent=2)+'\n');return r.returncode
for mode in ['fields','thermo','thermal','battery','replay','semantics']:
 assert run(['cargo','run','--locked','--offline','--release','--manifest-path',str(h/'Cargo.toml'),'--bin','c3-directed','--',mode],mode)==0
check=run(['python3',str(h/'check_c3.py')],'c3-check',{'D16_C3_RESULTS':str(o)});assert check==1,'the unchanged baseline must reproduce four failures'
s=json.loads((o/'c3-summary.json').read_text());assert(s['cases'],s['passed'],s['failures'],s['errors'])==(11,7,4,0)
assert run(['cargo','run','--locked','--offline','--release','--manifest-path',str(h/'Cargo.toml'),'--bin','c4-schema-conflict'],'schema-conflict')==0
facts=[json.loads(line) for line in(o/'schema-conflict.jsonl').read_text().splitlines()]
by_case={row['case']:row for row in facts}
assert by_case['physical_equivalence']['genuine_and_retagged_state_equal']
assert by_case['physical_equivalence']['genuine_and_retagged_hash_equal']
assert by_case['physical_equivalence']['difference_in_wrapper_model_only']
assert not by_case['retagged_original_fixture']['strict_pair_consistent']
assert by_case['genuine_v1']['strict_pair_consistent']
assert by_case['retagged_original_fixture']['c3_restore_accepted'] and by_case['genuine_v1']['c3_restore_accepted']
(o/'completion.json').write_text(json.dumps({'baseline_reproduced':True,'production_fixes_applied':False,'policy_decision':'PENDING_OWNER_DECISION'},indent=2)+'\n')
