//! Torque indicado desde trazas p(θ) (S14): `τ = Σ p_i·dV_i/dθ` con los
//! cuatro cilindros en fase (compañeras S06). La traza monocilíndrica S12
//! se replica en los 4 (cilindros idénticos; S16 diferenciará).
//!
//! Convención: la traza de ciclo COMPLETO incluye bombeo; quien acople
//! solo circuito cerrado debe sumar pumping explícito (ver ADR-021).

use kinematics::{phases::CylinderId, GeometryPack};
use math::{OutOfRange, Table1};
use thiserror::Error;

/// Error de torque indicado.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum TorqueError {
    /// Traza inválida (vacía, no finita, o ángulo fuera con rechazo).
    #[error("invalid indicated-torque input: {reason}")]
    InvalidInput { reason: String },
}

/// Torque indicado interpolado de la traza p(θ) monocilíndrica.
#[derive(Debug, Clone)]
pub struct IndicatedTorque {
    table: Table1,
    geom: GeometryPack,
}

impl IndicatedTorque {
    /// Construye desde serie (θ_grados 0–720, p_Pa) de un cilindro.
    ///
    /// # Errors
    ///
    /// `InvalidInput` si hay < 2 puntos, no finitos o θ no creciente.
    pub fn from_trace(
        theta_deg: &[f64],
        p_pa: &[f64],
        geom: GeometryPack,
    ) -> Result<Self, TorqueError> {
        if theta_deg.len() < 2 || theta_deg.len() != p_pa.len() {
            return Err(TorqueError::InvalidInput {
                reason: "need >= 2 matching points".to_owned(),
            });
        }
        let table =
            Table1::new("p_theta", theta_deg, p_pa).map_err(|_| TorqueError::InvalidInput {
                reason: "trace must be finite with increasing angles".to_owned(),
            })?;
        Ok(Self { table, geom })
    }

    /// Presión del cilindro `cyl` a ángulo global (interpola la traza
    /// monocilíndrica en su local; fuera de [0,720] satura al borde).
    pub fn pressure_at(&self, global_deg: f64, cyl: CylinderId) -> f64 {
        let local = kinematics::cylinder_local_angle(global_deg, cyl);
        self.table.eval(local, OutOfRange::Clamp).unwrap_or(0.0)
    }

    /// Torque indicado total (N·m) a ángulo global: Σ p·dV/dθ.
    pub fn torque_at(&self, global_deg: f64) -> f64 {
        let mut tau = 0.0;
        for cyl in [
            CylinderId::Cyl1,
            CylinderId::Cyl2,
            CylinderId::Cyl3,
            CylinderId::Cyl4,
        ] {
            let local = kinematics::cylinder_local_angle(global_deg, cyl);
            let crank = kinematics::phases::crank_angle_for_piston(local);
            let p = self.pressure_at(global_deg, cyl);
            tau += p * self.geom.dvdtheta(crank.to_radians());
        }
        tau
    }

    /// Trabajo indicado por ciclo (J) integrando τ en 720° (Simpson).
    #[must_use]
    pub fn cycle_work(&self) -> f64 {
        let n = 1440usize;
        let h = 4.0 * std::f64::consts::PI / n as f64;
        let mut s = self.torque_at(0.0) + self.torque_at(720.0);
        for i in 1..n {
            let t = f64::from(i as u32) * 720.0 / n as f64;
            s += if i % 2 == 0 { 2.0 } else { 4.0 } * self.torque_at(t);
        }
        s * h / 3.0
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn flat_trace(p: f64) -> (Vec<f64>, Vec<f64>) {
        (vec![0.0, 360.0, 720.0], vec![p, p, p])
    }

    #[test]
    fn constant_pressure_gives_zero_net_work() {
        // p constante ⇒ ∮p·dV = 0 por cilindro (conservación geométrica).
        let geom = GeometryPack::illustrative_d16_like().unwrap();
        let (th, pp) = flat_trace(200_000.0);
        let t = IndicatedTorque::from_trace(&th, &pp, geom).unwrap();
        assert!(t.cycle_work().abs() < 1e-6);
        // Pero el torque instantáneo NO es cero (cruces por fase).
        assert!(t.torque_at(100.0).abs() > 0.0);
    }
}
