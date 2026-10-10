//! Slow-state persistente S26: un motor = un estado; serializable,
//! determinista desde seed, con replay. Nuevo y gastado nunca comparten
//! estado (estructuras independientes por construcción).

use crate::laws::{LawError, WearLaw};
use serde::{Deserialize, Serialize};

/// Punto operativo horario (lo que el harness declara por tramo).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OperatingPoint {
    /// Régimen medio (rpm).
    pub rpm: f64,
    /// Carga media 0–1 (fracción de p_contacto nominal).
    pub load_01: f64,
    /// T aceite media (°C, alimenta Arrhenius + viscosidad).
    pub oil_temp_c: f64,
    /// T refrigerante media (°C).
    pub coolant_temp_c: f64,
    /// Calidad de película media (acopla S22; 0–1).
    pub film_0_1: f64,
    /// Presión de contacto nominal anillo (Pa, ilustrativa 5 MPa).
    pub ring_pressure_pa: f64,
    /// Presión media cojinete (Pa, ilustrativa 5 MPa).
    pub bearing_pressure_pa: f64,
}

impl OperatingPoint {
    /// Valida dominio.
    ///
    /// # Errors
    ///
    /// `Invalid` si algo viola dominio.
    pub fn validated(&self) -> Result<(), LawError> {
        for v in [
            self.rpm,
            self.load_01,
            self.oil_temp_c,
            self.coolant_temp_c,
            self.film_0_1,
            self.ring_pressure_pa,
            self.bearing_pressure_pa,
        ] {
            if !v.is_finite() {
                return Err(LawError::Invalid {
                    reason: "non-finite op".to_owned(),
                });
            }
        }
        if self.rpm < 0.0
            || !(0.0..=1.0).contains(&self.load_01)
            || !(0.0..=1.0).contains(&self.film_0_1)
            || self.ring_pressure_pa < 0.0
            || self.bearing_pressure_pa < 0.0
        {
            return Err(LawError::Invalid {
                reason: "op domain".to_owned(),
            });
        }
        Ok(())
    }
}

/// Slow-state de UN motor (horas de uso + cotas que crecen).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WearState {
    pub schema_version: u32,
    pub seed: u64,
    pub hours: f64,
    /// Gap de anillo por cilindro (m, crece por Archard).
    pub ring_gap_m: [f64; 4],
    /// Desgaste de camisa por cilindro (m).
    pub liner_wear_m: [f64; 4],
    /// Holgura extra de cojinete 5+4 (m, crece con inanición).
    pub bearing_extra_m: [f64; 9],
    /// Oxidación del aceite 0–1 (Arrhenius).
    pub oil_oxidation_01: f64,
    /// Relajación de junta 0–1 (creep ilustrativo).
    pub gasket_relax_01: f64,
}

impl WearState {
    /// Motor nuevo: gaps stock S23, resto a cero.
    #[must_use]
    pub fn new_seeded(seed: u64) -> Self {
        Self {
            schema_version: 1,
            seed,
            hours: 0.0,
            ring_gap_m: [0.3e-3; 4],
            liner_wear_m: [0.0; 4],
            bearing_extra_m: [0.0; 9],
            oil_oxidation_01: 0.0,
            gasket_relax_01: 0.0,
        }
    }

    /// Avanza `dt_hours` en el punto dado (Euler en horas: la
    /// aceleración de tiempo es exacta por construcción del integrador
    /// lineal en t; el test de subdivisión lo verifica).
    ///
    /// # Errors
    ///
    /// `Invalid` si punto/paso violan dominio o el esquema es ajeno.
    pub fn advance(
        &mut self,
        dt_hours: f64,
        op: &OperatingPoint,
        ring_law: &WearLaw,
        bearing_law: &WearLaw,
        oil_law: &WearLaw,
    ) -> Result<(), LawError> {
        if self.schema_version != 1 {
            return Err(LawError::Invalid {
                reason: "schema".to_owned(),
            });
        }
        op.validated()?;
        if !(dt_hours.is_finite() && dt_hours > 0.0) {
            return Err(LawError::Invalid {
                reason: "dt".to_owned(),
            });
        }
        // Deslizamiento del anillo: 2 carreras × rpm × 60 × dt.
        let sliding_m = 2.0 * 0.0907 * op.rpm * 60.0 * dt_hours;
        let dh = ring_law.archard_step_oiled(
            op.ring_pressure_pa * op.load_01,
            sliding_m,
            2e9,
            op.film_0_1,
        )?;
        for k in 0..4 {
            self.ring_gap_m[k] += dh / 2.0;
            self.liner_wear_m[k] += dh / 2.0;
        }
        // Cojinete: inanición × presión × tiempo (acopla film S22).
        let dt_s = dt_hours * 3600.0;
        for b in self.bearing_extra_m.iter_mut() {
            *b += bearing_law.coeff * (1.0 - op.film_0_1) * op.bearing_pressure_pa * dt_s;
        }
        // Aceite: Arrhenius hacia oxidación total (satura en 1).
        let t_k = op.oil_temp_c + 273.15;
        // A ilustrativa 5e4 (P1): ~0.06/50h a 100 °C, ~0.4 a 130 °C.
        let rate = 5e4 * (-oil_law.coeff / (8.314 * t_k)).exp();
        self.oil_oxidation_01 = (self.oil_oxidation_01 + rate * dt_s).min(1.0);
        // Junta: creep con tasa k/(1+t); el tramo se integra en forma
        // cerrada (ln) para que la subdivisión no altere el resultado
        // (bug real detectado por el test de escala temporal).
        let k = 1e-4 * op.load_01 * (op.coolant_temp_c / 90.0);
        let dg = k * ((1.0 + self.hours + dt_hours).ln() - (1.0 + self.hours).ln());
        self.gasket_relax_01 = (self.gasket_relax_01 + dg).min(1.0);
        self.hours += dt_hours;
        Ok(())
    }

    /// Área de fuga de anillo por cilindro (acopla S23: el desgaste
    /// abre el gap y la fuga crece por la ley, no por bandera).
    #[must_use]
    pub fn ring_leak_area_m2(&self, cyl: usize, depth_m: f64, cd: f64) -> f64 {
        leak::annulus_leak_area(self.ring_gap_m[cyl.min(3)], depth_m).unwrap_or(0.0) * cd
    }

    /// Hash canónico del estado (replay verificable).
    #[must_use]
    pub fn hash(&self) -> String {
        engine_types::sha256_hex(engine_types::canonical_json(self).as_bytes())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    pub(crate) fn op() -> OperatingPoint {
        OperatingPoint {
            rpm: 3000.0,
            load_01: 0.6,
            oil_temp_c: 100.0,
            coolant_temp_c: 88.0,
            film_0_1: 0.8,
            ring_pressure_pa: 5e6,
            bearing_pressure_pa: 5e6,
        }
    }

    #[test]
    fn step_subdivision_is_time_scale_free() {
        // Gate S26: el desgaste no depende de FPS/paso — 50 h en 50×1 h
        // ≡ 5000×0.01 h (tolerancia de redondeo, documentada).
        let laws = (
            WearLaw::archard_ring(),
            WearLaw::bearing_starvation(),
            WearLaw::oil_aging(),
        );
        let mut a = WearState::new_seeded(7);
        for _ in 0..50 {
            a.advance(1.0, &op(), &laws.0, &laws.1, &laws.2).unwrap();
        }
        let mut b = WearState::new_seeded(7);
        for _ in 0..5000 {
            b.advance(0.01, &op(), &laws.0, &laws.1, &laws.2).unwrap();
        }
        // El gate exige independencia de la subdivisión, no igualdad de
        // bits entre redondeos distintos (50 vs 5000 sumas acumulan fp
        // diferente): acuerdo relativo 1e-9 en TODOS los campos.
        assert!((a.hours - b.hours).abs() < 1e-9);
        let rel = |x: f64, y: f64| {
            if x.abs() < 1e-300 && y.abs() < 1e-300 {
                0.0
            } else {
                (x - y).abs() / x.abs().max(1e-300)
            }
        };
        for k in 0..4 {
            assert!(rel(a.ring_gap_m[k], b.ring_gap_m[k]) < 1e-9);
            assert!(rel(a.liner_wear_m[k], b.liner_wear_m[k]) < 1e-9);
        }
        for k in 0..9 {
            assert!(rel(a.bearing_extra_m[k], b.bearing_extra_m[k]) < 1e-9);
        }
        assert!(rel(a.oil_oxidation_01, b.oil_oxidation_01) < 1e-9);
        assert!(rel(a.gasket_relax_01, b.gasket_relax_01) < 1e-9);
    }

    #[test]
    fn replay_and_snapshot_roundtrip() {
        let laws = (
            WearLaw::archard_ring(),
            WearLaw::bearing_starvation(),
            WearLaw::oil_aging(),
        );
        let mut a = WearState::new_seeded(42);
        a.advance(10.0, &op(), &laws.0, &laws.1, &laws.2).unwrap();
        // Replay desde seed: bit-idéntico.
        let mut c = WearState::new_seeded(42);
        c.advance(10.0, &op(), &laws.0, &laws.1, &laws.2).unwrap();
        assert_eq!(a.hash(), c.hash());
        // Snapshot JSON: roundtrip exacto.
        let json = engine_types::canonical_json(&a);
        let back: WearState = serde_json::from_str(&json).unwrap();
        assert_eq!(a, back);
        // Nuevo vs gastado: estados independientes (semilla distinta no
        // contamina; el gastado avanzó y el nuevo no).
        let fresh = WearState::new_seeded(42);
        assert_ne!(a.ring_gap_m, fresh.ring_gap_m);
        assert_eq!(fresh.hours, 0.0);
    }
}
