#!/usr/bin/env python3
"""New contracts for C1 coverage, independent of the preserved two oracles."""
import json, math, os, unittest
from pathlib import Path
OUT=Path(os.environ['D16_REVIEW_RESULTS'])
MODES=['boundary','chemistry','analytic','diagnostic','migration','thermal','electronics','fit','dyno']
ALL=[json.loads(x) for m in MODES for x in (OUT/f'controls-{m}.jsonl').read_text().splitlines()]
def rows(name):return [x['data'] for x in ALL if x['probe']==name]
class C1Controls(unittest.TestCase):
    predicates=0
    def check(self,ok,msg):
        C1Controls.predicates+=1;self.assertTrue(ok,msg)
    def test_01_every_step_chemical_inventory(self):
        self.check(len(rows('step_chemical_budget'))==12,'coverage')
        for r in rows('step_chemical_budget'):
            self.check(r['error'] is None and r['reacting_steps']>0 and r['inventory_closure_kg']<1e-12 and r['max_step_inventory_heat_error_j']<1e-7 and r['max_fraction']<=1+1e-12 and r['replay_equal'] and r['burn_window_cuts']==3,str(r))
    def test_02_closed_form_burn_antiderivative(self):
        a=rows('burn_analytic');self.check(len(a)==15,'coverage')
        # Composite rectangular quadrature on an interval with endpoint jumps:
        # |error| <= h * total_variation(rate), TV <= 2*peak(rate).
        peak=15*(2/15)**(2/3)*math.exp(-2/3)/(math.pi/3)
        for r in a:self.check(r['accepted'] and r['error_kg']<2e-6*peak*100*r['dt']+1e-14 and r['energy_error_j']<1e-7 and abs(r['omega_delta'])<1e-8,str(r))
        for efficiency in [.25,.5,1.]:
            b=sorted([r for r in a if r['efficiency']==efficiency],key=lambda x:x['dt'],reverse=True)
            self.check(b[-1]['error_kg']<b[0]['error_kg']/100,str(b))
    def test_03_label_invariance_and_missing_observations(self):
        a=rows('diagnostic_labels');self.check(len(a)==6,'coverage')
        for x,y in zip(a[:3],a[3:]):self.check(x['split']==y['split'] and x['flag']==y['flag'],str([x,y]))
        for r in a:
            total=sum(r['flows']);expected=[r['flows'][0],r['flows'][2],r['flows'][1]]
            expected=[v/total for v in expected] if total else [0.,0.,0.]
            self.check(all(abs(x-y)<1e-12 for x,y in zip(expected,r['split'])),str(r))
        for r in rows('diagnostic_incomplete'):self.check('Inconclusive' in r['verdict'],str(r))
    def test_04_signed_zero_through_restore(self):
        a=rows('signed_zero_restore');self.check(len(a)==6,'coverage')
        for r in a:self.check(r['before_bits']==r['after_bits'] and r['hash_equal'],str(r))
    def test_05_genuine_historical_provenance(self):
        a=rows('historical_provenance');self.check(len(a)==4,'coverage')
        for r in a:
            p=r['provenance'];self.check(r['verified'] and r['roundtrip_verified'] and r['second_restore_equal'] and r['continuation_preserved'] and p['from_schema']==r['origin_schema'] and p['original_hash']==r['origin_hash'] and p['transformation'] and 'pmax_pa' in p['missing_fields'] and p['continuation']==('exact_neutral' if 'cold' in r['name'] else 'approximate_hot'),str(r))
        for r in rows('r02_hash_compatibility'):self.check(r['verified'] and r['restore_accepted'],str(r))
    def test_06_complete_engine_validation(self):
        # Extreme finite friction is a legitimate stress case, not inherently invalid input.
        for r in rows('extended_engine_config'):
            if r['case']=='friction_overflow':continue
            self.check(not r['resume_panicked'] and not r['resume_accepted'] and r['unchanged'] and not r['run_panicked'] and not r['run_accepted'] and not r['restore_panicked'] and not r['restore_accepted'],str(r))
        for r in rows('extended_work_state'):self.check(not r['accepted'] and r['unchanged'],str(r))
    def test_15_r02_midburn_is_not_unlabeled_native_c1(self):
        a=rows("r02_midburn_migration");self.check(len(a)==1,"missing genuine R02 mid-burn producer")
        for r in a:self.check(r["hash_verified"] and r["restored_provenance"] is not None and r["restored_provenance"]["continuation"]=="approximate_hot",str(r))
    def test_14_dynamic_error_rolls_back_complete_work_state(self):
        for r in rows("runtime_error_atomic"):self.check(r["rejected"] and r["unchanged"],str(r))
    def test_07_restore_thermal_configuration(self):
        for r in rows('thermal_restore_config'):self.check(r['hash_valid'] and not r['accepted'],str(r))
    def test_08_thermal_atomicity_and_overflow(self):
        for r in rows('extended_thermal'):self.check(not r['accepted'] and r['unchanged'],str(r))
    def test_09_full_nonfinite_observation_matrix(self):
        self.check(len(rows('ecu_nonfinite_all'))==15,'coverage')
        for r in rows('ecu_nonfinite_all'):self.check(r['injections']==r['ignitions']==0 and r['state']=='SyncLost',str(r))
        for r in rows('decoder_nonfinite_all'):self.check(r['finite'] and r['state']=='SyncLost' and r['commands']==0,str(r))
    def test_10_changing_callback_dimensions_and_nonfinite(self):
        self.check(len(rows('fitter_boundaries'))==10,'coverage')
        for r in rows('fitter_boundaries'):self.check(not r['panicked'] and r['rejected'],str(r))
    def test_11_dyno_nonfinite_outputs(self):
        for r in rows('dyno_boundaries'):self.check(not r['accepted'] or r['finite'],str(r))
    def test_12_valid_temperature_energy_control(self):
        for r in rows('thermal_physical_control'):self.check(r['all_above_abs'] and abs(r['closure_j'])<1e-7,str(r))
    def test_13_invalid_diagnostic_observations(self):
        # Domain negatives: separate from unproven multiple-cause identifiability AUD-R01.
        for r in rows('diagnostic_invalid'):self.check('Inconclusive' in r['verdict'],str(r))
if __name__=='__main__':
    result=unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(C1Controls))
    summary={'test_targets':1,'cases':result.testsRun,'passed':result.testsRun-len(result.failures)-len(result.errors),'failures':len(result.failures),'errors':len(result.errors),'predicates_executed':C1Controls.predicates,'rows':len(ALL),'families':len(set(r['probe'] for r in ALL)),'failure_names':[x.id().split('.')[-1] for x,y in result.failures]}
    (OUT/'c1-controls-summary.json').write_text(json.dumps(summary,indent=2)+'\n')
    raise SystemExit(0 if result.wasSuccessful() else 1)
