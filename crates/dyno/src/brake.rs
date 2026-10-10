//! Freno + ambiente S28: carga declarada y corrección explícita.

use thiserror::Error;

/// Error de dyno.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum DynoError {
    /// Configuración o punto inválidos (fail-closed).
    #[error("invalid dyno input: {reason}")]
    Invalid { reason: String },
}

/// Condiciones ambiente del ensayo.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ambient {
    pub p_kpa: f64,
    pub t_c: f64,
}

impl Ambient {
    /// Estándar de referencia (corrección exactamente 1).
    #[must_use]
    pub fn standard() -> Self {
        Self {
            p_kpa: 99.0,
            t_c: 25.0,
        }
    }

    /// Valida dominio.
    ///
    /// # Errors
    ///
    /// `Invalid` si p/T no finitas o p ≤ 0.
    pub fn validated(&self) -> Result<(), DynoError> {
        if !(self.p_kpa.is_finite() && self.p_kpa > 0.0 && self.t_c.is_finite()) {
            return Err(DynoError::Invalid {
                reason: "ambient".to_owned(),
            });
        }
        Ok(())
    }
}

/// Corrección ambiente explícita estilo inherited (forma documentada
/// L1, NO certificación SAE): `cf = (99/p)·sqrt(T/298)`. Se reporta el
/// par OBSERVADO y el CORREGIDO; cf = 1 exacto en estándar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Correction {
    pub factor: f64,
    pub applied: bool,
}

impl Correction {
    /// Calcula; `apply=false` ⇒ factor 1 sin tocar nada.
    ///
    /// # Errors
    ///
    /// `Invalid` si el ambiente viola dominio.
    pub fn of(amb: &Ambient, apply: bool) -> Result<Self, DynoError> {
        amb.validated()?;
        if !apply {
            return Ok(Self {
                factor: 1.0,
                applied: false,
            });
        }
        Ok(Self {
            factor: (99.0 / amb.p_kpa) * ((amb.t_c + 273.15) / 298.15).sqrt(),
            applied: true,
        })
    }
}

/// Configuración del freno virtual (carga constante por punto + auxiliar).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DynoConfig {
    /// Par auxiliar (bomba agua/alternador, N·m, declarado).
    pub aux_tau_n_m: f64,
    pub ambient: Ambient,
    pub apply_correction: bool,
}

impl DynoConfig {
    /// Valida dominio.
    ///
    /// # Errors
    ///
    /// `Invalid` si el auxiliar es negativo/no finito.
    pub fn validated(&self) -> Result<(), DynoError> {
        self.ambient.validated()?;
        if !(self.aux_tau_n_m.is_finite() && self.aux_tau_n_m >= 0.0) {
            return Err(DynoError::Invalid {
                reason: "aux torque".to_owned(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn correction_is_one_at_standard_and_labeled() {
        let std = Correction::of(&Ambient::standard(), true).unwrap();
        assert_eq!(std.factor, 1.0);
        assert!(std.applied);
        let off = Correction::of(
            &Ambient {
                p_kpa: 90.0,
                t_c: 35.0,
            },
            true,
        )
        .unwrap();
        assert!(off.factor > 1.0);
        // Sin aplicar: 1.0 aunque el ambiente difiera (etiquetado).
        let plain = Correction::of(
            &Ambient {
                p_kpa: 90.0,
                t_c: 35.0,
            },
            false,
        )
        .unwrap();
        assert_eq!(plain.factor, 1.0);
        assert!(!plain.applied);
        assert!(Correction::of(
            &Ambient {
                p_kpa: -1.0,
                t_c: 0.0
            },
            true
        )
        .is_err());
    }
}
