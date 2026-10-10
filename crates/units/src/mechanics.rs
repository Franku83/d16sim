//! Mecánica: longitud, ángulo, fuerza, torque, masa, energía, caudales,
//! régimen de giro y potencia (banco vs rueda separados).
//!
//! La ausencia deliberada de `impl` cruzados (p. ej. `Torque + Kelvin`,
//! `Rpm` donde se pide `AngularVelocity`) es el mecanismo del gate S03:
//! esos programas **no compilan**.

use crate::float::{require_finite, require_non_negative, UnitsError};

/// Longitud en m (`>= 0`).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Length(f64);

/// Ángulo en rad (finito, con signo).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct AngleRad(f64);

/// Fuerza en N (finita, con signo).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Force(f64);

/// Torque en N·m (finito, con signo).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Torque(f64);

/// Masa en kg (`>= 0`).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct MassKg(f64);

/// Energía en J (finita, con signo).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct EnergyJoule(f64);

/// Caudal másico en kg/s (`>= 0`).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct MassFlowKgPerS(f64);

/// Régimen en revoluciones por minuto (`>= 0`). Nunca rad/s implícitos.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Rpm(f64);

/// Velocidad angular en rad/s (finita, con signo: admite reversa).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct AngularVelocity(f64);

/// Potencia al cigüeñal (freno) en W (`>= 0`).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct CrankWatts(f64);

/// Potencia a las ruedas en W (`>= 0`). Tipo distinto: la pérdida de
/// transmisión solo se aplica con rendimiento explícito.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct WheelWatts(f64);

impl Length {
    /// # Errors
    ///
    /// `NonFinite` / `OutOfRange` si no es `>= 0` finito.
    pub fn from_m(m: f64) -> Result<Self, UnitsError> {
        Ok(Self(require_non_negative(m, "length_m")?))
    }

    /// # Errors
    ///
    /// Igual que [`Self::from_m`].
    pub fn from_mm(mm: f64) -> Result<Self, UnitsError> {
        Self::from_m(mm / 1_000.0)
    }

    #[must_use]
    pub fn as_m(self) -> f64 {
        self.0
    }

    #[must_use]
    pub fn as_mm(self) -> f64 {
        self.0 * 1_000.0
    }
}

impl AngleRad {
    /// # Errors
    ///
    /// `NonFinite` si no es finito.
    pub fn from_rad(rad: f64) -> Result<Self, UnitsError> {
        Ok(Self(require_finite(rad, "angle_rad")?))
    }

    /// # Errors
    ///
    /// Igual que [`Self::from_rad`].
    pub fn from_deg(deg: f64) -> Result<Self, UnitsError> {
        Self::from_rad(deg * std::f64::consts::PI / 180.0)
    }

    #[must_use]
    pub fn as_rad(self) -> f64 {
        self.0
    }

    #[must_use]
    pub fn as_deg(self) -> f64 {
        self.0 * 180.0 / std::f64::consts::PI
    }
}

impl Force {
    /// # Errors
    ///
    /// `NonFinite` si no es finito.
    pub fn from_newton(n: f64) -> Result<Self, UnitsError> {
        Ok(Self(require_finite(n, "force_N")?))
    }

    #[must_use]
    pub fn as_newton(self) -> f64 {
        self.0
    }
}

impl Torque {
    /// # Errors
    ///
    /// `NonFinite` si no es finito.
    pub fn from_newton_metre(nm: f64) -> Result<Self, UnitsError> {
        Ok(Self(require_finite(nm, "torque_Nm")?))
    }

    #[must_use]
    pub fn as_newton_metre(self) -> f64 {
        self.0
    }
}

impl MassKg {
    /// # Errors
    ///
    /// `NonFinite` / `OutOfRange` si no es `>= 0` finito.
    pub fn from_kg(kg: f64) -> Result<Self, UnitsError> {
        Ok(Self(require_non_negative(kg, "mass_kg")?))
    }

    #[must_use]
    pub fn as_kg(self) -> f64 {
        self.0
    }

    /// Cantidad de sustancia (mol) desde masa y masa molar explícita.
    /// Codifica "moles vs kg" (§S03.3): sin masa molar no hay moles.
    ///
    /// # Errors
    ///
    /// `NonFinite` / `OutOfRange` si la masa molar no es `> 0` finita.
    pub fn as_moles(self, molar_mass_kg_per_mol: f64) -> Result<f64, UnitsError> {
        let mm = require_finite(molar_mass_kg_per_mol, "molar_mass")?;
        if mm > 0.0 {
            Ok(self.0 / mm)
        } else {
            Err(UnitsError::OutOfRange {
                what: "molar_mass",
                value: mm,
            })
        }
    }
}

impl EnergyJoule {
    /// # Errors
    ///
    /// `NonFinite` si no es finito.
    pub fn from_joule(j: f64) -> Result<Self, UnitsError> {
        Ok(Self(require_finite(j, "energy_J")?))
    }

    #[must_use]
    pub fn as_joule(self) -> f64 {
        self.0
    }
}

impl MassFlowKgPerS {
    /// # Errors
    ///
    /// `NonFinite` / `OutOfRange` si no es `>= 0` finito.
    pub fn from_kg_per_s(v: f64) -> Result<Self, UnitsError> {
        Ok(Self(require_non_negative(v, "mass_flow")?))
    }

    #[must_use]
    pub fn as_kg_per_s(self) -> f64 {
        self.0
    }
}

impl Rpm {
    /// # Errors
    ///
    /// `NonFinite` / `OutOfRange` si no es `>= 0` finito.
    pub fn from_rpm(rpm: f64) -> Result<Self, UnitsError> {
        Ok(Self(require_non_negative(rpm, "rpm")?))
    }

    #[must_use]
    pub fn as_rpm(self) -> f64 {
        self.0
    }

    /// Conversión explícita RPM → rad/s. No existe `From<Rpm>` a propósito:
    /// la llamada debe ser visible en el código.
    #[must_use]
    pub fn to_rad_per_s(self) -> AngularVelocity {
        AngularVelocity(self.0 * 2.0 * std::f64::consts::PI / 60.0)
    }
}

impl AngularVelocity {
    /// # Errors
    ///
    /// `NonFinite` si no es finito.
    pub fn from_rad_per_s(w: f64) -> Result<Self, UnitsError> {
        Ok(Self(require_finite(w, "omega_rad_s")?))
    }

    #[must_use]
    pub fn as_rad_per_s(self) -> f64 {
        self.0
    }

    /// Conversión explícita rad/s → RPM.
    #[must_use]
    pub fn to_rpm(self) -> Rpm {
        // `Rpm` valida `>= 0`; una velocidad negativa (reversa) satura a 0
        // aquí en vez de construir un RPM imposible: la reversa se modela
        // con `AngularVelocity`, no con `Rpm`.
        Rpm(self.0.max(0.0) * 60.0 / (2.0 * std::f64::consts::PI))
    }
}

impl CrankWatts {
    /// # Errors
    ///
    /// `NonFinite` / `OutOfRange` si no es `>= 0` finito.
    pub fn from_watts(w: f64) -> Result<Self, UnitsError> {
        Ok(Self(require_non_negative(w, "crank_power_W")?))
    }

    #[must_use]
    pub fn as_watts(self) -> f64 {
        self.0
    }

    /// Potencia a ruedas con rendimiento de transmisión **explícito**.
    /// Prohíbe la "pérdida %" fija circular del dyno (§23.10 FIELD-20).
    ///
    /// # Errors
    ///
    /// `OutOfRange` si `efficiency` no está en `(0, 1]`.
    pub fn to_wheel(self, drivetrain_efficiency: f64) -> Result<WheelWatts, UnitsError> {
        let e = require_finite(drivetrain_efficiency, "drivetrain_efficiency")?;
        if e > 0.0 && e <= 1.0 {
            Ok(WheelWatts(self.0 * e))
        } else {
            Err(UnitsError::OutOfRange {
                what: "drivetrain_efficiency",
                value: e,
            })
        }
    }
}

impl WheelWatts {
    #[must_use]
    pub fn as_watts(self) -> f64 {
        self.0
    }
}

// Aritmética solo intra-tipo: la dimensionalidad la impone el compilador.
impl std::ops::Add for Length {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Add for Torque {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Add for EnergyJoule {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn lcg(state: &mut u64) -> f64 {
        *state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((*state >> 11) as f64) / ((1u64 << 53) as f64)
    }

    #[test]
    fn rpm_rad_s_roundtrip() {
        // 750 rpm ralentí candidato + barrido hasta 8000 rpm.
        assert!(
            (Rpm::from_rpm(750.0).unwrap().to_rad_per_s().as_rad_per_s() - 78.539_816_339_744_83)
                .abs()
                < 1e-9
        );
        let mut s = 42;
        for _ in 0..2000 {
            let rpm = lcg(&mut s) * 8000.0;
            let back = Rpm::from_rpm(rpm).unwrap().to_rad_per_s().to_rpm();
            assert!((back.as_rpm() - rpm).abs() <= 1e-9 * rpm.max(1.0));
        }
    }

    #[test]
    fn crank_wheel_needs_explicit_efficiency() {
        let crank = CrankWatts::from_watts(79_000.0).unwrap();
        assert!(crank.to_wheel(0.0).is_err());
        assert!(crank.to_wheel(1.2).is_err());
        let wheel = crank.to_wheel(0.85).unwrap();
        assert!((wheel.as_watts() - 67_150.0).abs() < 1e-6);
    }

    #[test]
    fn moles_need_molar_mass() {
        let m = MassKg::from_kg(1.0).unwrap();
        assert!(m.as_moles(0.0).is_err());
        assert!((m.as_moles(0.029).unwrap() - 34.482_758_620_689_66).abs() < 1e-9);
    }
}
