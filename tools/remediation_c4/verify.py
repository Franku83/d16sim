#!/usr/bin/env python3
"""C4 implementation evidence, not an independent integration gate.
Reconstruct frozen oracles from Git. Keep literal and owner-authorized adaptation separate.
"""
import argparse,subprocess,tarfile,io,hashlib,json,os,time,difflib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
REVIEW='36919b0455e8eaf741a72ff4a74ae85f40e116ba'
p=argparse.ArgumentParser();p.add_argument('--source-ref',default='HEAD');p.add_argument('--output',type=Path,required=True);p.add_argument('--archive',type=Path,required=True);a=p.parse_args();o=a.output.resolve();w=a.archive.resolve()
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT)
assert not o.exists()or not any(o.iterdir()),'never overwrite evidence';assert not w.exists()or not any(w.iterdir()),'archive must be new'
o.mkdir(parents=True,exist_ok=True);w.mkdir(parents=True,exist_ok=True)
source=git('rev-parse',a.source_ref).decode().strip()
for ref,paths in [(source,[]),(REVIEW,['verification/c3_review'])]:tarfile.open(fileobj=io.BytesIO(git('archive',ref,*paths))).extractall(w,filter='data')
h=w/'verification/c3_review';commands=[]
for d in ['literal','adapted','shared','c2','c3','adapter','oracles']:(o/d).mkdir()
def run(argv,name,*,env=None,cwd=w):
 start=time.monotonic();dest=o/name;dest.parent.mkdir(parents=True,exist_ok=True)
 with Path(str(dest)+'.log').open('w')as err,Path(str(dest)+'.jsonl').open('w')as out:r=subprocess.run([str(x)for x in argv],cwd=cwd,stdout=out,stderr=err,env=dict(os.environ,**(env or {})))
 commands.append({'argv':[str(x)for x in argv],'cwd':str(cwd),'exit':r.returncode,'elapsed_s':time.monotonic()-start,'name':name,'env':env or {}});(o/'commands.json').write_text(json.dumps(commands,indent=2)+'\n');print(name,'exit',r.returncode,flush=True);return r.returncode
checks=[]
for file in ['check_contracts.py','check_adversarial.py','check_c1_controls.py','check_c2.py','check_c3.py','src/main.rs','src/adversarial.rs','src/common.rs','src/c1_controls.rs','src/c2_boundaries.rs','src/c3_directed.rs']:
 b=(h/file).read_bytes();assert b==git('show',REVIEW+':verification/c3_review/'+file)
 checks.append({'path':file,'origin':REVIEW+':verification/c3_review/'+file,'sha256':hashlib.sha256(b).hexdigest(),'byte_identical':True})
 if file.endswith('.py'):(o/'oracles'/Path(file).name).write_bytes(b)
(o/'integrity.json').write_text(json.dumps({'source_tree_or_commit':source,'frozen_review':REVIEW,'files':checks},indent=2)+'\n')
# Exactly the approved one-line metadata adaptation, in a separate derived source/bin.
original=(h/'src/main.rs').read_bytes();needle=b'    v1.state.schema_version = 1;\n';assert original.count(needle)==1
derived=original.replace(needle,needle+b'    v1.model = None;\n',1)
(h/'src/v1_fixture_adapter.rs').write_bytes(derived)
(o/'adapter/original_main.rs').write_bytes(original);(o/'adapter/derived_main.rs').write_bytes(derived)
(o/'adapter/fixture.diff').write_text(''.join(difflib.unified_diff(original.decode().splitlines(True),derived.decode().splitlines(True),fromfile='original_main.rs',tofile='derived_main.rs')))
(o/'adapter/transformation.json').write_text(json.dumps({'authorization':'User formal Option A, 2026-10-15; C4_OPTION_A_AUTHORIZATION.md','origin':REVIEW+':verification/c3_review/src/main.rs','original_sha256':hashlib.sha256(original).hexdigest(),'derived_sha256':hashlib.sha256(derived).hexdigest(),'only_inserted_statement':'v1.model = None;','state_hash_parameters_expectations_tolerances_unchanged':True},indent=2)+'\n')
(h/'src/verify_v1_adapter.rs').write_bytes((w/'tools/remediation_c4/verify_v1_adapter.rs').read_bytes())
with(h/'Cargo.toml').open('a')as f:f.write('\n[[bin]]\nname="v1-fixture-adapted"\npath="src/v1_fixture_adapter.rs"\n\n[[bin]]\nname="verify-v1-adapter"\npath="src/verify_v1_adapter.rs"\n')
for cmd,label in [(['cargo','test','--locked','--workspace'],'workspace-test'),(['cargo','fmt','--all','--','--check'],'fmt'),(['cargo','clippy','--locked','--workspace','--all-targets','--','-D','clippy::unwrap_used'],'clippy')]:assert run(cmd,label)==0,label
def cargo(directory,bin_name,argv,label,env=None):return run(['cargo','run','--locked','--offline','--release','--manifest-path',h/directory/'Cargo.toml','--bin',bin_name,*(['--',*argv]if argv else [])],label,env=env)
for ref,name in [('2541266','legacy'),('ff09a0d','s30'),('8a7e868','r02'),('caad1cb','c1'),('a0040c5','c2')]:
 target=w/'target'/('remediation-review-'+name);target.mkdir(parents=True,exist_ok=True);tarfile.open(fileobj=io.BytesIO(git('archive',ref))).extractall(target,filter='data')
# Genuine producers, not reconstructed legacy DTOs.
for directory,bin_name,args,label in [('legacy','d16sim-historical-v1-producer',[],'historical-v1-hot'),('legacy','historical-cold-or-hot',['cold'],'historical-v1-cold'),('legacy_v2','d16sim-historical-v2-producer',['cold'],'historical-v2-cold'),('legacy_v2','d16sim-historical-v2-producer',[],'historical-v2-hot'),('r02_producer','d16sim-r02-native-producer',[],'historical-r02-native'),('r02_producer','r02-mid-burn',[],'historical-r02-mid-burn')]:
 assert cargo(directory,bin_name,args,'shared/'+label)==0;(o/'shared'/(label+'.json')).write_bytes((o/'shared'/(label+'.jsonl')).read_bytes())
assert (h/'fixtures/genuine-v1-cold.json').read_bytes()==(o/'shared/historical-v1-cold.json').read_bytes()
for category,bin_name in [('literal','d16sim-independent-probes'),('adapted','v1-fixture-adapted')]:
 for mode in ['engine','subsystems','dyno']:assert cargo('.',bin_name,[mode],category+'/probes-'+mode)==0
 assert run(['python3',h/'check_contracts.py'],category+'/contract-check',env={'D16_AUDIT_RESULTS':str(o/category)})==1
lit=(o/'literal/probes-engine.jsonl').read_text().splitlines();ad=(o/'adapted/probes-engine.jsonl').read_text().splitlines();assert len(lit)==len(ad)
deltas=[(json.loads(x),json.loads(y))for x,y in zip(lit,ad)if x!=y];assert len(deltas)==1 and deltas[0][0]['probe']=='v1_hash_migration'
x,y=deltas[0];assert not x['data']['accepted'] and y['data']['accepted'];x['data']['accepted']=True;assert x==y
for mode in ['subsystems','dyno']:assert(o/f'literal/probes-{mode}.jsonl').read_bytes()==(o/f'adapted/probes-{mode}.jsonl').read_bytes()
(o/'adapter/result_equivalence.json').write_text(json.dumps({'only_observation_changed':'v1_hash_migration.accepted: false -> true','state_bytes_and_hash_identical':True,'all_other_observations_byte_identical':True},indent=2)+'\n')
for mode in ['engine','leak','thermal','snapshot','boundary']:assert cargo('.','remediation-adversarial',[mode],'shared/adversarial-'+mode,{'D16_REVIEW_RESULTS':str(o/'shared')})==0
assert run(['python3',h/'check_adversarial.py'],'shared/adversarial-check',env={'D16_REVIEW_RESULTS':str(o/'shared')})==0
for mode in ['boundary','chemistry','analytic','diagnostic','migration','thermal','electronics','fit','dyno']:assert cargo('.','c1-controls',[mode],'shared/controls-'+mode,{'D16_REVIEW_RESULTS':str(o/'shared')})==0
assert run(['python3',h/'check_c1_controls.py'],'shared/c1-check',env={'D16_REVIEW_RESULTS':str(o/'shared')})==0
assert run(['python3',h/'run_history.py','--worktree',w,'--output',o/'c2/producer-matrix'],'c2/producer-driver')==0
for mode,label in [('matrix','new-matrix'),('thermal','new-thermal'),('diagnostics','new-diagnostic'),('snapshots','new-snapshots'),('focused','focused')]:assert cargo('.','c2-boundaries',[mode],'c2/'+label,{'D16_C2_FIXTURES':str(o/'c2/producer-matrix')})==0
assert run(['python3',h/'check_c2.py'],'c2/check',env={'D16_C2_RESULTS':str(o/'c2')})==0
for mode in ['fields','thermo','thermal','battery','replay','semantics']:assert cargo('.','c3-directed',[mode],'c3/'+mode)==0
assert run(['python3',h/'check_c3.py'],'c3/check',env={'D16_C3_RESULTS':str(o/'c3')})==0
assert cargo('.','verify-v1-adapter',[],'adapter/behavior')==0
for scenario in ['healthy_baseline','missing_head_gasket','underclamped_head','ring_leak_cyl3']:assert run(['cargo','run','--locked','--offline','--release','--bin','d16sim','--','mech-demo','--scenario',scenario],'smoke-'+scenario)==0
(o/'completion.json').write_text(json.dumps({'status':'IMPLEMENTED_PENDING_INDEPENDENT_REVIEW','source_tree_or_commit':source,'literal_original_case14':'FAIL_EXPECTED_INCOHERENT_MODEL_REJECTED','adapter_authorized_by_owner':True,'no_integration_gate_granted':True},indent=2)+'\n')
