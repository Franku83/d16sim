#!/usr/bin/env python3
"""Additional C4 mechanism controls, independent of all five preserved checkers."""
import json,os,math,unittest,sys
from decimal import Decimal,localcontext
from pathlib import Path
O=Path(os.environ['D16_C4_RESULTS'])
A=[json.loads(x)for mode in ['provenance','adapter','battery','nodes']for x in(O/(mode+'.jsonl')).read_text().splitlines()]
def rows(k):return [x['data']for x in A if x['probe']==k]
class C4(unittest.TestCase):
 predicates=0
 def check(self,x,msg):C4.predicates+=1;self.assertTrue(x,str(msg))
 def test_01_node_pressure_density_errors_atomic(self):
  self.check(len(rows('node_boundary'))==24,'two nodes x6 states x2 intervals')
  for r in rows('node_boundary'):
   self.check(r['hash_valid'] and r['restore_ok']==r['expected_valid_state'] and r['unchanged_on_error'],r)
   if not r['expected_valid_state']:self.check(not r['resume_ok'],r)
   elif r['end']==0.:self.check(r['resume_ok'],r)
 def test_02_factory_run_audits_finite_or_error(self):
  for r in rows('node_factory'):self.check(not r['accepted'] or r['audit_finite'],r)
 def test_03_composition_preserves_order_origin_and_scope(self):
  self.check(len(rows('provenance_transition'))==8,'eight distinct transitions')
  for r in rows('provenance_transition'):self.check(r['incoming_verified'] and r['origin_hash_unchanged'] and r['previous_record_preserved'] and r['chain_hashes']==r['expected_hashes'] and r['chain_depth']==r['step']+2 and r['scope']=='approximate_hot',r)
 def test_04_idempotence_json_recapture_no_growth(self):
  for r in rows('provenance_transition'):self.check(r['reread20_equal'] and r['recapture20_equal'],r)
 def test_05_adapter_reproduces_genuine_state_not_inconsistent_metadata(self):
  self.check(len(rows('adapter_identity'))==1,'coverage')
  for r in rows('adapter_identity'):self.check(all(r[k]for k in ['literal_hash_valid','adapted_hash_valid','genuine_hash_valid','state_all_equal','hash_all_equal','provenance_all_equal','adapted_accepted','genuine_accepted'])and not r['literal_accepted'],r)
 def test_06_all_present_schema_pairs_strict(self):
  for r in rows('schema_pair'):self.check(r['hash_valid'] and r['accepted']==r['expected_valid'],r)
 def test_07_derived_resistance_decimal_reference_and_api_consistency(self):
  self.check(len(rows('battery_boundary'))==126,'7 temperatures x3SOC x6resistancecases')
  for r in rows('battery_boundary'):
   with localcontext()as c:
    c.prec=2000;mx=Decimal.from_float(sys.float_info.max);name=r['case'];r0,g={'ideal':(Decimal(0),Decimal(0)),'subnormal':(Decimal.from_float(float.fromhex('0x0.0000000000001p-1022')),Decimal(0)),'nominal':(Decimal.from_float(.008),Decimal.from_float(.002)),'large_finite':(mx/16,Decimal(0)),'product_overflow':(mx,Decimal(0)),'sum_overflow':(mx,mx)}[name]
    t=Decimal(str(r['temp']));soc=Decimal(str(r['soc']));cold=1+Decimal('.02')*max(Decimal(0),20-t);expected=t>Decimal('-273.15') and r0*(1+Decimal('1.5')*(1-soc))*cold+g<=mx
   self.check(r['validated']==expected and r['resistance_ok']==expected and r['terminal_ok']==expected and r['resume_ok']==expected and r['restore_ok']==expected and r['run_ok']==expected and r['unchanged_on_error'],r)
   if expected:self.check(r['resistance_finite_nonnegative'] and r['terminal_finite'] and (name!='ideal' or r['resistance']==0),r)
 def test_08_hash_does_not_authenticate_metadata(self):
  for r in rows('metadata_trust'):self.check(r['state_hash_still_valid_after_metadata_change'],r)
if __name__=='__main__':
 r=unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(C4));s={'cases':r.testsRun,'passed':r.testsRun-len(r.failures)-len(r.errors),'failures':len(r.failures),'errors':len(r.errors),'predicates_executed':C4.predicates,'rows':len(A),'families':len(set(x['probe']for x in A)),'failure_names':[x.id().split('.')[-1]for x,y in r.failures]};(O/'c4-summary.json').write_text(json.dumps(s,indent=2)+'\n');raise SystemExit(not r.wasSuccessful())
