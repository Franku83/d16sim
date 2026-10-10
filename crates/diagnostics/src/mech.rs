//! Instrumentos mecánicos (S24): compresímetro, equipo de leak-down,
//! galga de interiores y micrómetro. Sin panel secreto: funciones puras de
//! lecturas/conductancias declaradas; el harness monta los escenarios.
//!
//! Modelos L1 EXPLÍCITOS y NO calibrados (constantes ilustrativas con
//! sensibilidad declarada, S25 los medirá contra banco):
//! - Cranking: `pico = p_ideal·fill·V/(V+V_dead)/(1+C·A·t_ciclo·n)`.
//! - Leak-down: `fuga% = 100·A_leak/(A_leak+A_orificio)`.
//! Dirección exacta por construcción; magnitud pendiente.

use thiserror::Error;

/// Error mecánico.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum MechError {
    /// Parámetro o lectura inválida (fail-closed).
    #[error("invalid mechanical input: {reason}")]
    Invalid { reason: String },
}

/// Procedencia de observación (S24-T6: instrumento, calibración,
/// operador, timestamp y condiciones en TODA lectura).
#[derive(Debug, Clone, PartialEq)]
pub struct Provenance {
    pub instrument: String,
    pub calibration_id: String,
    pub operator: String,
    pub timestamp: String,
    pub conditions: String,
}

impl Provenance {
    /// Constructor validado (todo no vacío).
    ///
    /// # Errors
    ///
    /// `Invalid` si algún campo está vacío.
    pub fn new(
        instrument: &str,
        calibration_id: &str,
        operator: &str,
        timestamp: &str,
        conditions: &str,
    ) -> Result<Self, MechError> {
        for (v, n) in [
            (instrument, "instrument"),
            (calibration_id, "calibration"),
            (operator, "operator"),
            (timestamp, "timestamp"),
            (conditions, "conditions"),
        ] {
            if v.trim().is_empty() {
                return Err(MechError::Invalid {
                    reason: n.to_owned(),
                });
            }
        }
        Ok(Self {
            instrument: instrument.to_owned(),
            calibration_id: calibration_id.to_owned(),
            operator: operator.to_owned(),
            timestamp: timestamp.to_owned(),
            conditions: conditions.to_owned(),
        })
    }
}

/// Lectura con procedencia (toda observación S24 la lleva).
#[derive(Debug, Clone, PartialEq)]
pub struct Reading {
    pub value: f64,
    pub unit: String,
    pub provenance: Provenance,
}

/// Compresímetro: volumen muerto + resolución + error de cero + T.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CompressionGauge {
    /// Volumen muerto (m³, adaptador + manguera).
    pub dead_vol_m3: f64,
    /// Resolución (kPa).
    pub resolution_kpa: f64,
    /// Error de cero (kPa, sesgo del manómetro).
    pub zero_error_kpa: f64,
}

impl CompressionGauge {
    /// Valida dominio.
    ///
    /// # Errors
    ///
    /// `Invalid` si volumen/resolución violan dominio.
    pub fn validated(&self) -> Result<(), MechError> {
        if !(self.dead_vol_m3.is_finite() && self.dead_vol_m3 >= 0.0) {
            return Err(MechError::Invalid {
                reason: "dead volume".to_owned(),
            });
        }
        if !(self.resolution_kpa.is_finite() && self.resolution_kpa > 0.0) {
            return Err(MechError::Invalid {
                reason: "resolution".to_owned(),
            });
        }
        if !self.zero_error_kpa.is_finite() {
            return Err(MechError::Invalid {
                reason: "zero error".to_owned(),
            });
        }
        Ok(())
    }

    /// Aplica instrumento a un pico verdadero: sesgo + cuantización.
    ///
    /// # Errors
    ///
    /// `Invalid` si el manómetro o el pico violan dominio.
    pub fn read(&self, true_peak_kpa: f64, provenance: Provenance) -> Result<Reading, MechError> {
        self.validated()?;
        if !(true_peak_kpa.is_finite() && true_peak_kpa >= 0.0) {
            return Err(MechError::Invalid {
                reason: "peak".to_owned(),
            });
        }
        let biased = (true_peak_kpa + self.zero_error_kpa).max(0.0);
        let q = (biased / self.resolution_kpa).round() * self.resolution_kpa;
        Ok(Reading {
            value: q,
            unit: "kPa".to_owned(),
            provenance,
        })
    }
}

/// Constante ilustrativa del modelo de cranking (1/(m²·s), NO calibrada).
pub const C_CRANK_ILLUSTRATIVE: f64 = 8.0e5;

/// Pico de cranking L1: `p_ideal·fill·V/(V+V_dead)/(1+C·A·t·n)`.
/// Más fuga ⇒ menos pico; más lento ⇒ menos pico; más muerto ⇒ menos
/// pico; WOT (fill≈1) ⇒ más pico que mariposa cerrada (mismo motor).
///
/// # Errors
///
/// `Invalid` si algún parámetro viola dominio.
pub fn cranking_peak_kpa(
    p_ideal_trapped_kpa: f64,
    throttle_fill_01: f64,
    cyl_vol_m3: f64,
    dead_vol_m3: f64,
    leak_area_m2: f64,
    cranking_rpm: f64,
    cycles_observed: u32,
    c_leak: f64,
) -> Result<f64, MechError> {
    let bad = MechError::Invalid {
        reason: "cranking model".to_owned(),
    };
    for v in [
        p_ideal_trapped_kpa,
        throttle_fill_01,
        cyl_vol_m3,
        leak_area_m2,
        cranking_rpm,
        c_leak,
    ] {
        if !v.is_finite() {
            return Err(bad);
        }
    }
    if p_ideal_trapped_kpa < 0.0
        || !(0.0..=1.0).contains(&throttle_fill_01)
        || cyl_vol_m3 <= 0.0
        || dead_vol_m3 < 0.0
        || leak_area_m2 < 0.0
        || cranking_rpm <= 0.0
        || cycles_observed == 0
        || c_leak < 0.0
    {
        return Err(bad);
    }
    let t_cycle_s = 120.0 / cranking_rpm;
    Ok(p_ideal_trapped_kpa * throttle_fill_01 * cyl_vol_m3
        / (cyl_vol_m3 + dead_vol_m3)
        / (1.0 + c_leak * leak_area_m2 * t_cycle_s * f64::from(cycles_observed)))
}

/// Equipo de leak-down: regulador + orificio interno + posición declarada.
#[derive(Debug, Clone, PartialEq)]
pub struct LeakDownTester {
    /// Presión de suministro regulada (kPa).
    pub supply_kpa: f64,
    /// Orificio interno (m², el que fija la escala del %).
    pub orifice_m2: f64,
    /// Posición mecánica declarada (p. ej. "TDC compresión cil 3").
    pub position: String,
}

impl LeakDownTester {
    /// % de fuga L1: `100·A/(A+A_orificio)` + lectura de dos manómetros.
    ///
    /// # Errors
    ///
    /// `Invalid` si el equipo o el área violan dominio.
    pub fn read(
        &self,
        leak_area_m2: f64,
        provenance: Provenance,
    ) -> Result<(Reading, Reading), MechError> {
        let bad = MechError::Invalid {
            reason: "leak-down".to_owned(),
        };
        if !(self.supply_kpa.is_finite() && self.supply_kpa > 0.0) {
            return Err(bad);
        }
        if !(self.orifice_m2.is_finite() && self.orifice_m2 > 0.0) {
            return Err(bad);
        }
        if self.position.trim().is_empty() {
            return Err(bad);
        }
        if !(leak_area_m2.is_finite() && leak_area_m2 >= 0.0) {
            return Err(bad);
        }
        let pct = 100.0 * leak_area_m2 / (leak_area_m2 + self.orifice_m2);
        let held_kpa = self.supply_kpa * (1.0 - pct / 100.0);
        Ok((
            Reading {
                value: self.supply_kpa,
                unit: "kPa supply".to_owned(),
                provenance: provenance.clone(),
            },
            Reading {
                value: pct,
                unit: "% leak".to_owned(),
                provenance,
            },
        ))
        .map(|(a, b)| {
            let _ = held_kpa;
            (a, b)
        })
    }
}

/// Galga de interiores / micrómetro: resolución + cero + T + orientación.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoreGauge {
    /// Resolución (mm).
    pub resolution_mm: f64,
    /// Error de cero (mm).
    pub zero_error_mm: f64,
}

impl BoreGauge {
    /// Mide un diámetro verdadero: sesgo + cuantización.
    ///
    /// # Errors
    ///
    /// `Invalid` si instrumento o valor violan dominio.
    pub fn read(&self, true_mm: f64, provenance: Provenance) -> Result<Reading, MechError> {
        if !(self.resolution_mm.is_finite() && self.resolution_mm > 0.0) {
            return Err(MechError::Invalid {
                reason: "resolution".to_owned(),
            });
        }
        if !(true_mm.is_finite() && true_mm > 0.0) {
            return Err(MechError::Invalid {
                reason: "diameter".to_owned(),
            });
        }
        let q = ((true_mm + self.zero_error_mm) / self.resolution_mm).round() * self.resolution_mm;
        Ok(Reading {
            value: q,
            unit: "mm".to_owned(),
            provenance,
        })
    }
}

/// Dirección del reparto de fuga escuchado/observado en banco.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LeakSplit {
    /// Fracción a cárter (varilla/tapón de aceite).
    pub to_crankcase_01: f64,
    /// Fracción a escape/admisión (cola del escape).
    pub to_exhaust_01: f64,
    /// Fracción a refrigerante (burbujas en vaso).
    pub to_coolant_01: f64,
}

/// Causa discriminada por el protocolo (nunca leída de un flag).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MechCause {
    Rings,
    Valve,
    HeadGasket,
}

/// Veredicto del protocolo `diagnose_low_compression`.
#[derive(Debug, Clone, PartialEq)]
pub enum MechVerdict {
    /// Faltan mediciones independientes (el gate exige ≥ 2).
    Inconclusive { have_n: usize, need_n: usize },
    Discriminated {
        cause: MechCause,
        by_measurements: String,
    },
}

/// Protocolo S24: compresión BAJA + leak-down con reparto (+ geometría
/// opcional). Con UNA sola medición siempre es `Inconclusive`; con DOS
/// independientes discrimina por ruta física:
///
/// - compresión baja + fuga a cárter ⇒ anillos;
/// - compresión baja + fuga a escape ⇒ válvula;
/// - compresión baja + fuga a refrigerante (o múltiple) ⇒ junta.
#[must_use]
pub fn diagnose_low_compression(
    compression_kpa: Option<f64>,
    low_threshold_kpa: f64,
    leak_pct: Option<f64>,
    split: Option<LeakSplit>,
) -> MechVerdict {
    let mut n = 0;
    if compression_kpa.is_some() {
        n += 1;
    }
    if leak_pct.is_some() && split.is_some() {
        n += 1;
    }
    if n < 2 {
        return MechVerdict::Inconclusive {
            have_n: n,
            need_n: 2,
        };
    }
    let comp = compression_kpa.unwrap_or(f64::MAX);
    let pct = leak_pct.unwrap_or(0.0);
    let s = split.unwrap_or(LeakSplit {
        to_crankcase_01: 0.0,
        to_exhaust_01: 0.0,
        to_coolant_01: 0.0,
    });
    if comp >= low_threshold_kpa || pct < 10.0 {
        return MechVerdict::Inconclusive {
            have_n: n,
            need_n: 2,
        };
    }
    if s.to_coolant_01 >= 0.15 || (s.to_crankcase_01 > 0.2 && s.to_exhaust_01 > 0.2) {
        MechVerdict::Discriminated {
            cause: MechCause::HeadGasket,
            by_measurements: "compression + leak-down to coolant/multiple".to_owned(),
        }
    } else if s.to_crankcase_01 >= s.to_exhaust_01 {
        MechVerdict::Discriminated {
            cause: MechCause::Rings,
            by_measurements: "compression + leak-down to crankcase".to_owned(),
        }
    } else {
        MechVerdict::Discriminated {
            cause: MechCause::Valve,
            by_measurements: "compression + leak-down to exhaust".to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn prov(tag: &str) -> Provenance {
        Provenance::new("gauge", "CAL-01", "op", "2026-10-10", tag).unwrap()
    }

    #[test]
    fn throttle_procedure_changes_reading_same_engine() {
        // FIELD-07: mismo estado mecánico, WOT vs cerrada difieren por la
        // masa admitida (el procedimiento, no el motor).
        let wot = cranking_peak_kpa(
            1500.0,
            1.0,
            400e-6,
            8e-6,
            1e-6,
            200.0,
            4,
            C_CRANK_ILLUSTRATIVE,
        )
        .unwrap();
        let closed = cranking_peak_kpa(
            1500.0,
            0.35,
            400e-6,
            8e-6,
            1e-6,
            200.0,
            4,
            C_CRANK_ILLUSTRATIVE,
        )
        .unwrap();
        assert!(wot > closed && closed > 0.0, "wot={wot} closed={closed}");
    }

    #[test]
    fn gauge_dead_volume_changes_reading_same_engine() {
        // FIELD-08: dos compresímetros distintos leen distinto el MISMO
        // motor (volumen muerto / resolución), sin cambiar el estado.
        let a = CompressionGauge {
            dead_vol_m3: 4e-6,
            resolution_kpa: 5.0,
            zero_error_kpa: 0.0,
        };
        let b = CompressionGauge {
            dead_vol_m3: 12e-6,
            resolution_kpa: 25.0,
            zero_error_kpa: 10.0,
        };
        // Pico no múltiplo común (evita coincidencia de cuantización).
        let peak = 1113.0;
        let ra = a.read(peak, prov("A")).unwrap();
        let rb = b.read(peak, prov("B")).unwrap();
        assert!(ra.value != rb.value);
        assert!(ra.provenance.conditions == "A");
    }

    #[test]
    fn single_measurement_is_inconclusive_for_three_causes() {
        // Anillos/válvula/junta pueden dar la MISMA compresión baja: con
        // una sola medición el protocolo no discrimina (gate S24).
        for split in [
            LeakSplit {
                to_crankcase_01: 0.8,
                to_exhaust_01: 0.1,
                to_coolant_01: 0.0,
            },
            LeakSplit {
                to_crankcase_01: 0.1,
                to_exhaust_01: 0.8,
                to_coolant_01: 0.0,
            },
            LeakSplit {
                to_crankcase_01: 0.3,
                to_exhaust_01: 0.3,
                to_coolant_01: 0.3,
            },
        ] {
            let v = diagnose_low_compression(Some(700.0), 900.0, None, Some(split));
            assert_eq!(
                v,
                MechVerdict::Inconclusive {
                    have_n: 1,
                    need_n: 2
                }
            );
        }
    }

    #[test]
    fn two_measurements_separate_rings_valve_gasket() {
        // La SEGUNDA medición independiente (reparto del leak-down)
        // separa las tres causas con la misma compresión baja.
        let r = diagnose_low_compression(
            Some(700.0),
            900.0,
            Some(35.0),
            Some(LeakSplit {
                to_crankcase_01: 0.8,
                to_exhaust_01: 0.1,
                to_coolant_01: 0.0,
            }),
        );
        let v = diagnose_low_compression(
            Some(700.0),
            900.0,
            Some(35.0),
            Some(LeakSplit {
                to_crankcase_01: 0.1,
                to_exhaust_01: 0.8,
                to_coolant_01: 0.0,
            }),
        );
        let g = diagnose_low_compression(
            Some(700.0),
            900.0,
            Some(55.0),
            Some(LeakSplit {
                to_crankcase_01: 0.3,
                to_exhaust_01: 0.3,
                to_coolant_01: 0.3,
            }),
        );
        assert_eq!(
            r,
            MechVerdict::Discriminated {
                cause: MechCause::Rings,
                by_measurements: "compression + leak-down to crankcase".to_owned(),
            }
        );
        assert_eq!(
            v,
            MechVerdict::Discriminated {
                cause: MechCause::Valve,
                by_measurements: "compression + leak-down to exhaust".to_owned(),
            }
        );
        assert_eq!(
            g,
            MechVerdict::Discriminated {
                cause: MechCause::HeadGasket,
                by_measurements: "compression + leak-down to coolant/multiple".to_owned(),
            }
        );
    }
}
