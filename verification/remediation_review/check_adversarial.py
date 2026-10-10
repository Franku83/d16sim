#!/usr/bin/env python3
"""Independent required contracts. Failures stay red; observations are not tests."""
import json, math, os, unittest
from pathlib import Path
RESULTS=Path(os.environ.get('D16_REVIEW_RESULTS',Path(__file__).resolve().parents[2]/'docs/remediation/review/results'))
ALL=[json.loads(line) for mode in ['engine','leak','thermal','snapshot','boundary'] for line in (RESULTS/f'adversarial-{mode}.jsonl').read_text().splitlines()]
def rows(name):return [r['data'] for r in ALL if r['probe']==name]
class Contracts(unittest.TestCase):
    predicates=0
    def check(self,p,msg):
        Contracts.predicates+=1
        self.assertTrue(p,msg)
    def test_01_fuel_transport_budget_and_bounds(self):
        for r in rows('engine_species'):
            self.check(r['error'] is None and r['closure_kg']<1e-12 and r['max_fuel_fraction']<=1+1e-12,str(r))
        for r in rows('reverse_fuel'):
            self.check(r['ok'] and abs(r['fuel_closure_kg'])<1e-12 and abs(r['mass_closure_kg'])<1e-12 and all(0<=f<=1 for f in r['fractions']),str(r))
    def test_02_chemical_energy_matches_consumed_fuel_during_burn(self):
        # Zero burned fuel permits zero reaction heat; 1 J is an intentionally generous roundoff allowance.
        for r in rows('engine_species'):
            if r['spark']:
                self.check(r['max_chemical_excess_j']<1.,str(r))
    def test_03_physical_exhaust_is_reported(self):
        for r in rows('diagnostic_physical_exhaust'):
            # expected is [crank,coolant,exhaust], reported [crank,exhaust,coolant].
            e=r['expected'];self.check(r['exhaust_flag'] and all(abs(a-b)<1e-12 for a,b in zip(r['reported'],[e[0],e[2],e[1]])),str(r))
    def test_04_missing_split_cannot_discriminate(self):
        for r in rows('diagnostic_channels'):
            if sum(r['split'])==0:self.check('Inconclusive' in r['verdict'],str(r))
    def test_05_fresh_transport_closed_invariant(self):
        for r in rows('fresh_transport'):
            self.check(abs(r['closure_kg'])<1e-12 and abs(r['receiver']-r['expected'])<1e-12 and abs(r['ledger'])<1e-12,str(r))
        for r in rows('fresh_regulator'):
            self.check(abs(r['ledger'])<1e-12 and abs(r['mass'])<1e-12 and all(0<=x<=1 for x in r['fractions']),str(r))
    def test_06_liquid_mass_and_volume_two_densities(self):
        for r in rows('liquid_units'):
            self.check(abs(r['mass']/r['expected_mass']-1)<1e-12 and abs(r['volume']/r['expected_volume']-1)<1e-12,str(r))
    def test_07_thermal_local_and_global_energy(self):
        for r in rows('thermal_zero_boundary'):
            self.check(abs(r['closure_j'])<1e-7 and r['ambient_q_j']==0.,str(r))
        for r in rows('thermal_ledger'):
            self.check(abs(r['closure_j'])<1e-7 and r['max_local_error_j']<1e-7,str(r))
    def test_08_thermal_analytic_refinement(self):
        a=sorted(rows('thermal_analytic'),key=lambda x:x['dt'],reverse=True)
        for coarse,fine in zip(a,a[1:]):
            self.check(fine['error_c']<0.6*coarse['error_c'],str([coarse,fine]))
    def test_09_positive_float_roundtrip(self):
        for r in rows('snapshot_bits'):self.check(r['bits_equal'] and r['hash_equal'] and r['verified'] and r['state_equal'],str(r))
        for r in rows('snapshot_full_physical'):self.check(r['all_bits_equal'] and r['hash_equal'] and r['restore_accepted'],str(r))
        for r in rows('wear_roundtrip'):self.check(r['equal'] and r['hours_bits'] and r['gap_bits'],str(r))
    def test_10_signed_zero_preserves_promised_bits(self):
        for r in rows('snapshot_signed_zero'):self.check(r['before_bits']==r['after_bits'],str(r))
    def test_11_authentic_v1_integrity_and_continuation(self):
        a=rows('authentic_v1');self.check(len(a)==1,'missing historical fixture')
        for r in a:self.check(r['hash_verified'] and r['accepted'] and r['migrated_schema']==2,str(r))
        for r in rows('authentic_v1_continuation'):self.check(r['post_verifies'],str(r))
        for r in rows('legacy_negative'):self.check(r['rejected'],str(r))
    def test_12_native_replay_full_state_and_every_trace_sample(self):
        for r in rows('native_replay')+rows('native_multiple_cuts'):self.check(r['hash_equal'] and r['state_equal'] and r['trace_equal'],str(r))
    def test_13_engine_invalid_inputs_reject_atomically(self):
        for r in rows('engine_boundary_atomic'):self.check(not r['panicked'] and not r['accepted'] and r['unchanged'],str(r))
    def test_14_restore_rejects_invalid_physics_and_configuration(self):
        for r in rows('restore_boundary'):self.check(not r['accepted'],str(r))
    def test_15_thermal_rejects_invalid_state_atomically(self):
        for r in rows('thermal_invalid_state'):self.check(not r['accepted'] and r['unchanged'],str(r))
        for r in rows('thermal_invalid_ambient'):self.check(not r['accepted'],str(r))
        for r in rows('thermal_overflow_atomic'):self.check(r['rejected'] and r['unchanged'],str(r))
    def test_16_dyno_never_returns_nonfinite_factor(self):
        for r in rows('dyno_overflow'):self.check(not r['accepted'] or r['finite'],str(r))
    def test_17_decoder_and_ecu_invalid_observations_inhibit(self):
        for r in rows('decoder_boundary'):self.check(not r['nonfinite_estimate_time'] and r['finite_phase'],str(r))
        for r in rows('ecu_boundary'):self.check(r['finite_commands'] and r['injection_count']==0 and r['ignition_count']==0,str(r))
    def test_18_calibration_malformed_model_returns_error(self):
        for r in rows('calibration_dimension'):self.check(not r['panicked'] and r['rejected'],str(r))
    def test_19_manifest_rejects_invalid_observations_metadata(self):
        for r in rows('manifest_boundary'):self.check(r['rejected'],str(r))
if __name__=='__main__':
    result=unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(Contracts))
    summary={'test_targets':1,'test_cases':result.testsRun,'assertion_predicates_executed':Contracts.predicates,'passed':result.testsRun-len(result.failures)-len(result.errors),'failures':len(result.failures),'errors':len(result.errors),'observation_rows':len(ALL),'families':len({r['probe'] for r in ALL}),'failure_names':[t.id().split('.')[-1] for t,_ in result.failures]}
    (RESULTS/'adversarial-test-summary.json').write_text(json.dumps(summary,indent=2)+'\n')
    raise SystemExit(0 if result.wasSuccessful() else 1)
