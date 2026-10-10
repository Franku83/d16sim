//! Starter L1 (S15): par vs ω lineal, corriente explícita, reductora
//! piñón/corona. La eficiencia es DIAGNÓSTICO (≤ 1 exigido en tests), no
//! identidad contable: los modelos τ e I son independientes declarados.

use thiserror::Error;

/// Error de starter.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum StarterError {
    /// Parámetros inválidos (pares, corrientes, relación, ω).
    #[error("invalid starter input: {reason}")]
    InvalidInput { reason: String },
}

/// Motor de arranque (lado motor, antes de la reductora).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StarterMotor {
    /// Par de calado (N·m, lado motor).
    pub stall_torque_n_m: f64,
    /// Corriente de calado (A).
    pub stall_current_a: f64,
    /// Velocidad en vacío (rad/s, lado motor).
    pub free_speed_rad_s: f64,
    /// Corriente en vacío (A).
    pub free_current_a: f64,
    /// ¿Montado? `None` = ausente (fallo estructural, no eléctrico).
    pub present: bool,
}

impl StarterMotor {
    /// Starter ilustrativo (≈1 kW mecánico máx, 12 V): 28 N·m/350 A de
    /// calado, 150 rad/s en vacío. NO medido en el D16Y7.
    #[must_use]
    pub fn illustrative() -> Self {
        Self {
            stall_torque_n_m: 28.0,
            stall_current_a: 350.0,
            free_speed_rad_s: 150.0,
            free_current_a: 60.0,
            present: true,
        }
    }

    /// Valida el motor (magnitudes > 0 finitas y free < stall).
    ///
    /// # Errors
    ///
    /// `InvalidInput` si algún parámetro viola dominio.
    pub fn validated(&self) -> Result<(), StarterError> {
        let bad = |reason: &str| StarterError::InvalidInput {
            reason: reason.to_owned(),
        };
        for (v, name) in [
            (self.stall_torque_n_m, "stall torque"),
            (self.stall_current_a, "stall current"),
            (self.free_speed_rad_s, "free speed"),
            (self.free_current_a, "free current"),
        ] {
            if !(v.is_finite() && v > 0.0) {
                return Err(bad(name));
            }
        }
        if self.free_current_a >= self.stall_current_a {
            return Err(bad("free current below stall current"));
        }
        Ok(())
    }

    /// Par lado motor (N·m) a ω_motor; 0 si ausente o en/por encima de vacío.
    ///
    /// # Errors
    ///
    /// `InvalidInput` si el motor es inconsistente o ω no finita.
    pub fn torque_at(&self, omega_motor_rad_s: f64) -> Result<f64, StarterError> {
        self.validated()?;
        if !omega_motor_rad_s.is_finite() || omega_motor_rad_s < 0.0 {
            return Err(StarterError::InvalidInput {
                reason: "omega must be finite and >= 0".to_owned(),
            });
        }
        if !self.present {
            return Ok(0.0);
        }
        Ok(self.stall_torque_n_m * (1.0 - omega_motor_rad_s / self.free_speed_rad_s).max(0.0))
    }

    /// Corriente (A) a ω_motor; 0 si ausente.
    ///
    /// # Errors
    ///
    /// Igual que `torque_at`.
    pub fn current_at(&self, omega_motor_rad_s: f64) -> Result<f64, StarterError> {
        self.validated()?;
        if !omega_motor_rad_s.is_finite() || omega_motor_rad_s < 0.0 {
            return Err(StarterError::InvalidInput {
                reason: "omega must be finite and >= 0".to_owned(),
            });
        }
        if !self.present {
            return Ok(0.0);
        }
        let x = (omega_motor_rad_s / self.free_speed_rad_s).min(1.0);
        Ok(self.stall_current_a - (self.stall_current_a - self.free_current_a) * x)
    }
}

/// Reductora piñón/corona (relación explícita, ilustrativa 12:1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StarterDrive {
    pub ratio: f64,
}

impl StarterDrive {
    /// # Errors
    ///
    /// `InvalidInput` si la relación no es `> 0` finita.
    pub fn new(ratio: f64) -> Result<Self, StarterError> {
        if !(ratio.is_finite() && ratio > 0.0) {
            return Err(StarterError::InvalidInput {
                reason: "ratio must be finite and > 0".to_owned(),
            });
        }
        Ok(Self { ratio })
    }

    /// Par al cigüeñal (N·m) dada ω del cigüeñal.
    ///
    /// # Errors
    ///
    /// Propaga validación del motor.
    pub fn crank_torque(
        &self,
        motor: &StarterMotor,
        omega_crank_rad_s: f64,
    ) -> Result<f64, StarterError> {
        Ok(motor.torque_at(omega_crank_rad_s * self.ratio)? * self.ratio)
    }

    /// Corriente dada ω del cigüeñal.
    ///
    /// # Errors
    ///
    /// Propaga validación del motor.
    pub fn current(
        &self,
        motor: &StarterMotor,
        omega_crank_rad_s: f64,
    ) -> Result<f64, StarterError> {
        motor.current_at(omega_crank_rad_s * self.ratio)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn torque_and_current_fall_with_speed_and_efficiency_below_one() {
        let m = StarterMotor::illustrative();
        let d = StarterDrive::new(12.0).unwrap();
        // En calado: 28×12 = 336 N·m al cigüeñal, 350 A.
        assert!((d.crank_torque(&m, 0.0).unwrap() - 336.0).abs() < 1e-9);
        assert_eq!(d.current(&m, 0.0).unwrap(), 350.0);
        // A mitad de vacío el par cae a la mitad.
        let half = m.free_speed_rad_s / 2.0 / 12.0;
        assert!((d.crank_torque(&m, half).unwrap() - 168.0).abs() < 1e-9);
        // Eficiencia electromecánica ≤ 1 en todo el rango (12.8 V).
        for i in 0..=20 {
            let w = f64::from(i) * m.free_speed_rad_s / 20.0 / 12.0;
            let t = d.crank_torque(&m, w).unwrap();
            let cur = d.current(&m, w).unwrap();
            let eta = t * w / (12.8 * cur).max(1e-9);
            assert!(eta <= 1.0 + 1e-12 && eta >= 0.0, "eta={eta} at {w}");
        }
        // Ausente ⇒ 0 (fallo estructural).
        let gone = StarterMotor {
            present: false,
            ..m
        };
        assert_eq!(d.crank_torque(&gone, 0.0).unwrap(), 0.0);
        assert_eq!(d.current(&gone, 0.0).unwrap(), 0.0);
    }
}
