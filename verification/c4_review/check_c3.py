#!/usr/bin/env python3
"""Independent C3 contracts; separate from four preserved historical oracles."""
import json,os,unittest
from pathlib import Path
O=Path(os.environ['D16_C3_RESULTS']);MODES=['fields','thermo','thermal','battery','replay','semantics']
A=[json.loads(x)for m in MODES for x in(O/(m+'.jsonl')).read_text().splitlines()]
def rows(k):return[a['data']for a in A if a['probe']==k]
class C3(unittest.TestCase):
 predicates=0
 def check(self,ok,msg):C3.predicates+=1;self.assertTrue(ok,str(msg))
 def test_01_complete_cam_and_injector_fields(self):
  self.check(len(rows('parameter'))==240,'20 fields x6 values x2 intervals')
  for r in rows('parameter'):
   v=r['expected_valid'];self.check(not r['panic'] and r['resume_ok']==v and r['restore_ok']==v and r['run_ok']==v and (v or r['unchanged']),r)
  for r in rows('cam_relation'):self.check(r['resume_rejected'] and r['restore_rejected'] and r['unchanged'],r)
 def test_02_node_derived_pressure_and_density(self):
  self.check(len(rows('derived_node'))==20,'two nodes xfive stresses xtwo endpoints')
  for r in rows('derived_node'):
   valid=r['pressure_finite_positive'] and r['density_finite_positive'];self.check(not r['panic'] and r['hash_valid'],r)
   if not valid:self.check(not r['restore_ok'] and not r['resume_ok'] and r['unchanged'],r)
   elif r['end']==0.:self.check(r['restore_ok'] and r['resume_ok'],r)
 def test_03_cylinder_derived_values(self):
  self.check(len(rows('derived_cylinder_new'))==3,'coverage')
  for r in rows('derived_cylinder_new'):self.check(r['hash_valid'] and not r['restore_ok'] and not r['resume_ok'] and r['unchanged'],r)
 def test_04_contextual_thermal_energy(self):
  self.check(len(rows('thermal_context'))==24,'coverage')
  for r in rows('thermal_context'):self.check(r['resume_ok']==r['expected_valid'] and r['restore_ok']==r['expected_valid'] and r['unchanged_on_error'],r)
 def test_05_battery_temperature_domain(self):
  self.check(len(rows('battery'))==11,'coverage')
  for r in rows('battery'):
   expected=r['expected_valid'];self.check(r['validated']==expected and r['resistance_ok']==expected and r['terminal_voltage_ok']==expected and r['run_ok']==expected and r['resume_ok']==expected and r['unchanged_on_error'],r)
  a={r['label']:r for r in rows('battery')};self.check(a['-20.0']['resistance']>a['20.0']['resistance']>0,'cold vs warm')
 def test_06_battery_derived_result_finite(self):
  for r in rows('battery_extreme'):self.check(not r['resistance_ok'] or r['resistance_finite'],r)
 def test_07_pressure_history_is_conditionally_required(self):
  a=rows('history_control');self.check(len(a)==12,'3 frictions x4cuts')
  for r in a:
   if r['c2']==0.:self.check(r['provenance'] is None and r['trace_equal'] and r['remaining_state_equal'],r)
   else:self.check(r['provenance'] is not None and r['provenance']['continuation']=='approximate_hot' and 'pmax_pa' in r['provenance']['missing_fields'],r)
  self.check(any(not r['trace_equal'] for r in a if r['c2']!=0.),'pressure loss sometimes changes trajectory')
 def test_11_inherited_neutral_provenance_does_not_mask_new_history_loss(self):
  self.check(len(rows('inherited_provenance_loss'))==1,'genuine v1 cold migration')
  for r in rows('inherited_provenance_loss'):self.check(r['origin_hash_valid'] and r['missing_hash_valid'] and not r['trace_equal'] and r['returned_provenance'] is not None and r['returned_provenance']['continuation']=='approximate_hot',r)
 def test_08_other_memory_persisted(self):
  self.check(len(rows('history_persisted'))==4,'coverage')
  for r in rows('history_persisted'):self.check(r['state_equal'] and r['json_has_field'] and r['missing_field_rejected'],r)
 def test_09_known_coherent_and_unknown_semantics(self):
  for r in rows('semantic_cross'):
   m=r['model']
   if m is None:self.check(r['accepted'] and r['provenance'] is not None,r)
   elif m['schema_version']==999 or m['combustion_accounting']==999:self.check(not r['accepted'] and r['hash_valid'],r)
   elif m['schema_version']==r['schema']:self.check(r['accepted'],r)
 def test_10_incoherent_known_schema_contract(self):
  for r in rows('semantic_cross'):
   m=r['model']
   if m is not None and m['schema_version'] in [1,2] and m['schema_version']!=r['schema']:self.check(not r['accepted'],r)
if __name__=='__main__':
 r=unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(C3));s={'cases':r.testsRun,'passed':r.testsRun-len(r.failures)-len(r.errors),'failures':len(r.failures),'errors':len(r.errors),'predicates_executed':C3.predicates,'rows':len(A),'families':len(set(a['probe']for a in A)),'failure_names':[x.id().split('.')[-1]for x,y in r.failures]};(O/'c3-summary.json').write_text(json.dumps(s,indent=2)+'\n');raise SystemExit(not r.wasSuccessful())
