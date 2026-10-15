#!/usr/bin/env python3
"""Independent expanded C2 requirements. No changes to any historical oracle."""
import os,json,unittest
from pathlib import Path
O=Path(os.environ['D16_C2_RESULTS'])
A=[json.loads(x) for f in ['new-matrix','new-thermal','new-diagnostic','new-snapshots','focused'] for x in (O/(f+'.jsonl')).read_text().splitlines()]
def rows(k):return [a['data'] for a in A if a['probe']==k]
class C2(unittest.TestCase):
 predicates=0
 def check(self,ok,msg):C2.predicates+=1;self.assertTrue(ok,str(msg))
 def test_01_listed_nonfinite_inputs_rejected_atomically(self):
  a=[r for r in rows('input') if r['value'] in ['nan','inf','minus_inf'] and not r['field'].startswith(('injector','deadtime','cam'))]
  self.check(len(a)==222,'coverage:37 fields x3 values x2 intervals')
  for r in a:self.check(not r['accepted'] and not r['restore_ok'] and not r['run_ok'] and r['unchanged'] and not r['panic'] and not r['run_panic'],r)
 def test_02_public_active_cam_and_injector_parameters(self):
  for r in rows('input'):
   if r['value'] in ['nan','inf','minus_inf'] and r['field'].startswith(('injector','deadtime','cam')):self.check(not r['accepted'] and not r['restore_ok'] and not r['run_ok'] and r['unchanged'],r)
  for r in rows('active'):
   if r['case']!='cold_battery':self.check(not r['accepted'],r)
 def test_03_derived_physical_temperature_finite(self):
  for r in rows('derived_cylinder'):self.check(not r['temperature_finite'] and not r['restore_ok'] and not r['noop_ok'] and r['unchanged'],r)
 def test_04_finite_actual_thermal_energy_not_factory_temperature(self):
  self.check(len(rows('finite_actual_thermal'))==2,'coverage')
  for r in rows('finite_actual_thermal'):self.check(r['params_valid'] and r['actual_energy'] is not None and r['standalone_step_ok'] and r['standalone_stored'] is not None and r['resume_ok'] and r['restore_ok'],r)
 def test_05_thermal_invalid_domains_atomic_and_valid_extremes(self):
  self.check(len(rows('thermal_param'))==98,'coverage')
  for r in rows('thermal_param'):
   self.check(not r['panic'],r)
   if not r['params_valid'] or not r['stored_finite']:self.check(not r['accepted'] and r['unchanged'] and not r['restore_ok'] and not r['run_ok'] and not r['resume_ok'] and r['work_unchanged'],r)
  for r in rows('thermal_extreme_valid'):self.check(r['accepted'] and r['stored'] is not None,r)
 def test_06_diagnostics_domain_and_instrument_absence(self):
  self.check(len(rows('diagnostic'))==30,'coverage')
  for r in rows('diagnostic'):
   invalid=r['value'] in ['nan','inf','negative'] or (r['field']=='threshold' and r['value']=='zero') or (r['field'] in ['pct','crank','exhaust','coolant'] and r['value']=='above')
   if invalid:self.check('Inconclusive' in r['verdict'],r)
  for r in rows('split_tolerance'):self.check(('Inconclusive' in r['verdict'])==(abs(r['delta'])>1e-6),r)
  for r in rows('missing'):self.check(('Inconclusive' in r['verdict'])==(r['mask']!=7),r)
 def test_07_genuine_producers_and_provenance_persistence(self):
  a=rows('snapshot_producer');self.check(len(a)==25,'five genuine types x five cuts')
  for r in a:
   self.check(r['verified'] and r['restore_ok'] and r['roundtrip_verified'] and r['continuation_ok'] and r['provenance_after']==r['reread_provenance'],r)
   p=r['provenance_after'];native=r['fixture'].startswith('c2-')
   self.check(p is None if native else p is not None and p['original_hash']==r['hash'] and p['continuation']==('exact_neutral' if '-cold' in r['fixture'] else 'approximate_hot'),r)
 def test_08_all_unknown_or_incoherent_semantic_identifiers_rejected(self):
  for r in rows('semantic_negative'):
   if r['case'] in ['unknown_model','mismatch_model']:self.check(r['hash_valid'] and not r['accepted'],r)
 def test_09_native_history_cannot_be_silently_reconstructed_exact(self):
  for r in rows('native_missing_history'):self.check(r['hash_valid'] and not r['same_trajectory'] and r['provenance'] is not None and r['provenance']['continuation']=='approximate_hot',r)
 def test_11_relational_inputs(self):
  self.check(len(rows('relational_input'))==4,'coverage')
  for r in rows('relational_input'):self.check(not r['accepted'] and r['unchanged'],r)
 def test_10_errors_atomic_and_no_panics(self):
  for r in rows('input')+rows('state'):
   self.check(not r['panic'],r)
   if not r['accepted']:self.check(r['unchanged'],r)
if __name__=='__main__':
 r=unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(C2));s={'cases':r.testsRun,'passed':r.testsRun-len(r.failures)-len(r.errors),'failures':len(r.failures),'errors':len(r.errors),'predicates_executed':C2.predicates,'rows':len(A),'families':len(set(a['probe']for a in A)),'failure_names':[x.id().split('.')[-1]for x,y in r.failures]};(O/'c2-summary.json').write_text(json.dumps(s,indent=2)+'\n');raise SystemExit(not r.wasSuccessful())
