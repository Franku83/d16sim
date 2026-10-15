#!/usr/bin/env python3
"""Regression specifications against recorded numerical observations.

These tests deliberately assert the required contracts, not the buggy outputs.
Failures remain visible; a zero probe-process exit is never an audit PASS.
"""
import json
import math
import os
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
RESULTS = Path(os.environ.get("D16_AUDIT_RESULTS", ROOT / "docs/audit/results"))
ROWS = []
for name in ["probes-engine.jsonl", "probes-subsystems.jsonl", "probes-dyno.jsonl"]:
    path = RESULTS / name
    ROWS.extend(json.loads(line) for line in path.read_text().splitlines())


def rows(name):
    return [row["data"] for row in ROWS if row["probe"] == name]


def one(name):
    result = rows(name)
    if len(result) != 1:
        raise RuntimeError(f"{name}: expected one row, got {len(result)}")
    return result[0]


class Contracts(unittest.TestCase):
    checks = 0

    def check(self, predicate, message):
        Contracts.checks += 1
        self.assertTrue(predicate, message)

    def test_01_closed_cylinder_independent_invariant(self):
        errors = [r["invariant_relative_error"] for r in rows("analytic_cylinder")]
        for e in errors:
            self.check(e < 1e-6, f"pV^gamma error {e}")
        for a, b in zip(errors, errors[1:]):
            self.check(14 < a / b < 18, f"RK4 error ratio {a/b}")

    def test_02_flow_scalings_and_bernoulli(self):
        r = one("flow_metamorphic")
        self.check(r["pressure_area_scaling_ratio"] == 4, str(r))
        self.check(r["temperature_scaling_ratio"] == 0.5, str(r))
        self.check(r["bernoulli_relative_error"] < 1e-4, str(r))

    def test_03_integrated_mass_balance_in_3_to_30_seconds(self):
        for r in rows("engine_refinement"):
            self.check(r["result"]["ok"] and abs(r["result"]["mass"]) < 1e-9, str(r))

    def test_04_causal_cam_changes_both_signs(self):
        r = {r["shift_deg"]: r for r in rows("cam_integrated")}
        for shift in [-6, 6]:
            self.check(r[shift]["lift_at_70_mm"] != r[0]["lift_at_70_mm"], str(r[shift]))
            self.check(abs(r[shift]["rpm"]-r[0]["rpm"]) > 1, str(r[shift]))

    def test_05_negative_start_controls(self):
        for r in rows("engine_negative"):
            if r["scenario"] in ["no_fuel", "no_spark"]:
                self.check(r["result"]["rpm"] == 0, str(r))
        self.check(one("fuel_cut")["rpm_after"] == 0, str(one("fuel_cut")))

    def test_06_wear_film_temperature_and_subdivision(self):
        rr = rows("wear_grid")
        for r in rr:
            self.check(abs(r["gap_subdivision_diff"]) < 1e-12, str(r))
            if r["film"] == 1:
                self.check(r["gap_m"] == 0 and r["bearing_m"] == 0, str(r))
        self.check(rr[2]["oxidation"] > rr[1]["oxidation"] > rr[0]["oxidation"], str(rr[:3]))

    def test_07_damage_stays_within_declared_support(self):
        self.check([r["state"] for r in rows("damage_bolt")] == ["Elastic", "UncertainRange", "PlasticOnsetGeneric"], str(rows("damage_bolt")))
        self.check(rows("damage_geometry")[0]["state"].startswith("Clear"), str(rows("damage_geometry")))
        self.check("NOT inferred" in rows("damage_geometry")[1]["state"], str(rows("damage_geometry")))
        self.check(one("damage_support")["thread"] == "Unsupported", str(one("damage_support")))

    def test_08_scheduler_inhibits_and_reports_latency(self):
        r = one("scheduler")
        self.check(r["healthy_ignitions"] == 4 and r["lost_ignitions"] == 0, str(r))
        self.check(abs(r["requested"]-r["effective"]-4.5) < 1e-10, str(r))

    def test_09_hash_tamper_and_unknown_schema_reject(self):
        for key, value in one("restore_negative_controls").items():
            self.check(value, key)

    def test_10_fuel_species_outflow_and_inventory(self):
        r = one("fuel_outflow")
        self.check(r["mass_after"] < r["mass_before"], "fixture must have outflow")
        self.check(abs(r["fuel_after"]-r["expected_homogeneous_fuel_after"]) < 1e-15, str(r))
        self.check(one("fuel_inventory")["max_ratio_over_4s"] <= 1, str(one("fuel_inventory")))

    def test_11_replay_includes_pressure_history(self):
        r = [r for r in rows("replay_split") if r["c2"] == 0.05][0]
        self.check(r["hash_match"], str(r))

    def test_12_engine_json_roundtrip_preserves_hash(self):
        self.check(all(r["same_hash"] for r in rows("snapshot_json")), "engine JSON reload does not verify")

    def test_13_wear_json_roundtrip_preserves_hash(self):
        self.check(one("wear_roundtrip")["hash_match"], str(one("wear_roundtrip")))

    def test_14_actual_v1_hash_can_migrate(self):
        self.check(one("v1_hash_migration")["accepted"], "valid historical v1 bytes rejected")

    def test_15_leak_species_conserve(self):
        r = one("leak_species")
        self.check(abs(r["receiver_fresh"]-r["expected_receiver_fresh"]) < 1e-12, str(r))

    def test_16_liquid_volume_converts_to_mass(self):
        r = one("leak_liquid_units")
        self.check(abs(r["reported_mass_kg"]/r["expected_mass_kg_at_declared_870kg_m3"]-1) < 1e-9, str(r))

    def test_17_dyno_sparse_tail_matches_resolved_quality(self):
        r = [r for r in rows("dyno_sampling") if r["load"] == 8]
        self.check(r[-1]["tail_torque_resid"] < 0.1, "resolved tail should meet unchanged 10% criterion")
        self.check(r[0]["tail_torque_resid"] < 0.1, str(r[0]))

    def test_18_dyno_power_channels_share_window(self):
        r = [r for r in rows("dyno_public") if r["load"] == 4][0]["result"]
        residue = r["p_ind_kw"]-r["p_fric_kw"]-r["p_aux_kw"]-r["p_brake_kw_observed"]-r["p_accel_kw"]
        self.check(abs(residue)/r["p_ind_kw"] < 0.01, f"reported channels residual={residue}kW")

    def test_19_unsupported_wear_must_not_produce_prediction(self):
        self.check(not one("wear_unsupported")["accepted"], str(one("wear_unsupported")))

    def test_20_no_spontaneous_reversal_of_relaxation(self):
        r = one("wear_irreversible")
        self.check(not r["accepted"] or r["after"] >= r["before"], str(r))

    def test_21_decoder_reanchors_after_cyp(self):
        r = rows("decoder_recovery")
        self.check(r[0]["phase"] == r[1]["phase"], str(r))

    def test_22_decoder_origin_is_not_physical_phase(self):
        r = rows("decoder_origin")
        self.check(r[0]["phase"] == r[1]["phase"], str(r))

    def test_23_wire_declared_seed_controls_realization(self):
        r = rows("wire_seed")
        self.check(r[0]["sequence"] != r[1]["sequence"], "both seeds use hardcoded 7193")

    def test_24_engine_invalid_config_fails_closed(self):
        r = {r["case"]: r for r in rows("engine_invalid")}
        self.check(r["zero_record"]["returned_error"] and not r["zero_record"]["panic"], str(r))

    def test_25_thermal_positive_capacity(self):
        self.check(not one("thermal_invalid")["zero_capacity_accepted"], str(one("thermal_invalid")))

    def test_26_dyno_absolute_temperature(self):
        self.check(not one("dyno_invalid")["below_absolute_zero_accepted"], str(one("dyno_invalid")))

    def test_27_dataset_finite_observations_and_uncertainty(self):
        self.check(not one("dataset_invalid")["invalid_observations_accepted"], str(one("dataset_invalid")))

    def test_28_phase_periodicity_of_complete_engine(self):
        r = {r["shift_deg"]: r for r in rows("cam_integrated")}
        self.check(abs(r[720]["rpm"]-r[0]["rpm"]) < 1e-6, str([r[0], r[720]]))

    def test_29_snapshot_restore_validates_physical_state(self):
        self.check(not one("restore_invalid")["negative_mass_accepted"], str(one("restore_invalid")))

    def test_30_decoder_rejects_nonfinite_and_future_edges(self):
        r = one("decoder_invalid")
        self.check(not r["nan_accepted"] and not r["future_accepted"], str(r))

    def test_31_covariance_is_in_reported_parameter_units(self):
        for r in rows("fit_analytic"):
            if r["start"] != 2:
                self.check(r["ok"], str(r))
                self.check(abs(r["std_reported"]/r["std_independent"]-1) < 1e-6, str(r))

    def test_32_exact_initial_optimum_succeeds(self):
        r = [r for r in rows("fit_analytic") if r["start"] == 2][0]
        self.check(r["ok"], str(r))


if __name__ == "__main__":
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(Contracts)
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    summary = {"test_targets": 1, "test_cases": result.testsRun,
               "assertion_predicates_executed": Contracts.checks,
               "failures": len(result.failures), "errors": len(result.errors),
               "probe_rows": len(ROWS), "purpose": "required contracts; expected failures remain red"}
    (RESULTS / "independent-test-summary.json").write_text(json.dumps(summary, indent=2)+"\n")
    raise SystemExit(0 if result.wasSuccessful() else 1)
