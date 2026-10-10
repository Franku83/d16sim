//! Dataset S25-T3: manifiesto con fuente obligatoria y split sin leakage.
//! `Real` exige procedencia de banco; `SyntheticSelfTest` solo verifica la
//! maquinaria y NUNCA cuenta como validación (el gate lo prohíbe).

use thiserror::Error;

/// Error de dataset.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum DatasetError {
    /// Manifiesto inválido (leakage, procedencia ausente, IDs duplicados).
    #[error("invalid dataset: {reason}")]
    Invalid { reason: String },
}

/// Fuente del dato (rotulada siempre; los gráficos la distinguen).
#[derive(Debug, Clone, PartialEq)]
pub enum DataSource {
    /// Dato real de banco con procedencia (ensayo/motor/operador/fecha).
    Real { provenance: String },
    /// Sintético de self-consistency (verifica el fitter, NO valida).
    SyntheticSelfTest { generator: String },
}

/// Una observación con incertidumbre (k=1 declarada).
#[derive(Debug, Clone, PartialEq)]
pub struct Observation {
    pub quantity: String,
    pub value: f64,
    pub uncertainty_k1: f64,
    pub unit: String,
}

/// Un ensayo en un motor y campaña dados.
#[derive(Debug, Clone, PartialEq)]
pub struct Trial {
    pub id: String,
    pub motor_id: String,
    pub campaign: String,
    pub source: DataSource,
    pub observations: Vec<Observation>,
}

/// Manifiesto: train + holdout con regla anti-leakage.
#[derive(Debug, Clone, PartialEq)]
pub struct Manifest {
    pub train: Vec<Trial>,
    pub holdout: Vec<Trial>,
}

impl Manifest {
    /// Valida: IDs únicos, holdout en motor DISTINTO (o campaña distinta
    /// con justificación), reales con procedencia, y NINGÚN sintético en
    /// holdout de validación (los sintéticos no validan).
    ///
    /// # Errors
    ///
    /// `Invalid` si hay leakage, duplicados o procedencia ausente.
    pub fn validated(&self) -> Result<(), DatasetError> {
        let bad = |r: &str| DatasetError::Invalid {
            reason: r.to_owned(),
        };
        if self.train.is_empty() {
            return Err(bad("empty train"));
        }
        if self.holdout.is_empty() {
            return Err(bad("empty holdout"));
        }
        let mut ids = std::collections::HashSet::new();
        for t in self.train.iter().chain(self.holdout.iter()) {
            if t.id.trim().is_empty() || !ids.insert(t.id.clone()) {
                return Err(bad("duplicate/empty trial id"));
            }
            if t.observations.is_empty() {
                return Err(bad("trial without observations"));
            }
            // Metadatos/procedencia completos (R02-F15): sin motor/campaña no
            // es un ensayo trazable.
            if t.motor_id.trim().is_empty() || t.campaign.trim().is_empty() {
                return Err(bad("trial without motor/campaign metadata"));
            }
            for o in &t.observations {
                if o.quantity.trim().is_empty() || o.unit.trim().is_empty() {
                    return Err(bad("observation without quantity/unit"));
                }
                if !o.value.is_finite() {
                    return Err(bad("non-finite observation"));
                }
                if !(o.uncertainty_k1.is_finite() && o.uncertainty_k1 >= 0.0) {
                    return Err(bad("invalid observation uncertainty"));
                }
            }
            match &t.source {
                DataSource::Real { provenance } => {
                    if provenance.trim().is_empty() {
                        return Err(bad("real without provenance"));
                    }
                }
                DataSource::SyntheticSelfTest { .. } => {}
            }
        }
        // Anti-leakage: ningún motor del holdout en train.
        let train_motors: std::collections::HashSet<&str> =
            self.train.iter().map(|t| t.motor_id.as_str()).collect();
        for h in &self.holdout {
            if train_motors.contains(h.motor_id.as_str()) {
                return Err(bad("holdout motor present in train (leakage)"));
            }
            // Los sintéticos no pueden validar: holdout solo real.
            if matches!(h.source, DataSource::SyntheticSelfTest { .. }) {
                return Err(bad("synthetic in holdout (cannot validate)"));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn trial(id: &str, motor: &str, src: DataSource) -> Trial {
        Trial {
            id: id.to_owned(),
            motor_id: motor.to_owned(),
            campaign: "leakdown".to_owned(),
            source: src,
            observations: vec![Observation {
                quantity: "leak_g_min".to_owned(),
                value: 4.0,
                uncertainty_k1: 0.5,
                unit: "g/min".to_owned(),
            }],
        }
    }

    fn real(p: &str) -> DataSource {
        DataSource::Real {
            provenance: p.to_owned(),
        }
    }

    #[test]
    fn split_rules_hold() {
        // Split sano: motores distintos, reales con procedencia.
        let ok = Manifest {
            train: vec![trial("T1", "M1", real("banco A, op, 2026"))],
            holdout: vec![trial("H1", "M2", real("banco B, op, 2026"))],
        };
        ok.validated().unwrap();
        // Leakage: mismo motor en holdout.
        let leak = Manifest {
            train: vec![trial("T1", "M1", real("banco A"))],
            holdout: vec![trial("H1", "M1", real("banco A"))],
        };
        assert!(leak.validated().is_err());
        // Sintético en holdout: prohibido (no valida).
        let synth_hold = Manifest {
            train: vec![trial(
                "T1",
                "M1",
                DataSource::SyntheticSelfTest {
                    generator: "selftest".to_owned(),
                },
            )],
            holdout: vec![trial(
                "H1",
                "M2",
                DataSource::SyntheticSelfTest {
                    generator: "selftest".to_owned(),
                },
            )],
        };
        assert!(synth_hold.validated().is_err());
        // Real sin procedencia: rechazado.
        let noprove = Manifest {
            train: vec![trial("T1", "M1", real(""))],
            holdout: vec![trial("H1", "M2", real("banco B"))],
        };
        assert!(noprove.validated().is_err());
    }

    #[test]
    fn non_finite_observation_and_bad_uncertainty_rejected() {
        // R02-F15: valores no finitos o incertidumbre inválida no son datos.
        let mut nan = trial("T1", "M1", real("banco A"));
        nan.observations[0].value = f64::NAN;
        let m = Manifest {
            train: vec![nan],
            holdout: vec![trial("H1", "M2", real("banco B"))],
        };
        assert!(m.validated().is_err());

        let mut bad_sigma = trial("T1", "M1", real("banco A"));
        bad_sigma.observations[0].uncertainty_k1 = -1.0;
        let m2 = Manifest {
            train: vec![bad_sigma],
            holdout: vec![trial("H1", "M2", real("banco B"))],
        };
        assert!(m2.validated().is_err());

        let mut no_unit = trial("T1", "M1", real("banco A"));
        no_unit.observations[0].unit = String::new();
        let m3 = Manifest {
            train: vec![no_unit],
            holdout: vec![trial("H1", "M2", real("banco B"))],
        };
        assert!(m3.validated().is_err());
    }
}
