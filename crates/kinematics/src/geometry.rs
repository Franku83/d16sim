//! Geometría biela-manivela ideal sin offset de bulón.
//!
//! Ecuaciones (libros de texto de cinemática de mecanismos; contexto de
//! motor en Heywood, *Internal Combustion Engine Fundamentals* — aquí solo
//! se usa la geometría ideal, sin coeficientes empíricos):
//!
//! - `r = S/2`, `λ = r/l`, `A = π·B²/4`
//! - `x(θ) = r·(1−cosθ) + l − √(l² − r²·sin²θ)`
//! - `dx/dθ = r·sinθ + r²·sinθ·cosθ / D`, `D = √(l² − r²·sin²θ)`
//! - `d²x/dθ² = r·cosθ + (u′·D − u·D′)/D²`, `u = r²·sinθ·cosθ`,
//!   `u′ = r²·cos2θ`, `D′ = −r²·sinθ·cosθ/D`
//! - `V(θ) = Vc + A·x(θ)`, `dV/dθ = A·dx/dθ`, `Vc = Vd/(CR−1)`, `Vd = A·S`
//!
//! Unidades: SI vía `units` (m, rad). Los parámetros geométricos del preset
//! ilustrativo son candidatos PENDING (S01), no nominales OEM.

use thiserror::Error;
use units::mechanics::Length;

/// Error de construcción/evaluación cinemática.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum KinematicsError {
    /// Parámetro geométrico inválido (no positivo, no finito o λ ≥ 1).
    #[error("invalid geometry: {reason}")]
    InvalidGeometry { reason: String },
}

/// Procedencia del parámetro geométrico: siempre explícita.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IllustrativeSource {
    /// Candidato S01 pendiente de auditoría OEM (no usar como nominal).
    PendingOemCandidate { provenance: &'static str },
}

/// Paquete geométrico declarado: un motor = una instancia.
#[derive(Debug, Clone)]
pub struct GeometryPack {
    bore: Length,
    stroke: Length,
    rod_length: Length,
    clearance_volume_m3: f64,
    compression_ratio: f64,
    crank_radius_m: f64,
    lambda_rod: f64,
    piston_area_m2: f64,
    displaced_m3: f64,
    source_note: &'static str,
}

/// Estado cinemático instantáneo de un pistón.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SliderState {
    /// Desplazamiento desde PMS (m).
    pub x_m: f64,
    /// Velocidad del pistón (m/s) a ω dado.
    pub v_m_s: f64,
    /// Aceleración del pistón (m/s²) a ω dado, régimen estacionario.
    pub a_m_s2: f64,
    /// Volumen de cámara (m³).
    pub v_m3: f64,
    /// Derivada del volumen respecto al ángulo (m³/rad).
    pub dvdtheta_m3_per_rad: f64,
}

impl GeometryPack {
    /// Construye el paquete validando positividad y `λ = r/l < 1`.
    ///
    /// # Errors
    ///
    /// `InvalidGeometry` si algún parámetro no es `> 0` finito, si
    /// `compression_ratio <= 1` o si `λ >= 1` (mecanismo imposible).
    pub fn new(
        bore: Length,
        stroke: Length,
        rod_length: Length,
        compression_ratio: f64,
        source_note: &'static str,
    ) -> Result<Self, KinematicsError> {
        let reject = |reason: &str| KinematicsError::InvalidGeometry {
            reason: reason.to_owned(),
        };
        if !(bore.as_m() > 0.0 && stroke.as_m() > 0.0 && rod_length.as_m() > 0.0) {
            return Err(reject("bore, stroke and rod length must be > 0"));
        }
        if !(compression_ratio.is_finite() && compression_ratio > 1.0) {
            return Err(reject("compression ratio must be finite and > 1"));
        }
        let r = stroke.as_m() / 2.0;
        let lambda = r / rod_length.as_m();
        if !(lambda.is_finite() && lambda < 1.0) {
            return Err(reject("r/l must be < 1 (otherwise D is imaginary)"));
        }
        let area = std::f64::consts::PI * bore.as_m() * bore.as_m() / 4.0;
        let vd = area * stroke.as_m();
        let vc = vd / (compression_ratio - 1.0);
        Ok(Self {
            bore,
            stroke,
            rod_length,
            clearance_volume_m3: vc,
            compression_ratio,
            crank_radius_m: r,
            lambda_rod: lambda,
            piston_area_m2: area,
            displaced_m3: vd,
            source_note,
        })
    }

    /// Preset ilustrativo "tipo D16": B=75 mm (FORUM-CLAIM pendiente),
    /// S=90 mm (FORUM-CLAIM pendiente), biela 137 mm (MFR-TECH Arias),
    /// CR 9.4 (FORUM-CLAIM pendiente). **No es nominal OEM**: solo sirve
    /// para verificar la matemática contra soluciones analíticas.
    ///
    /// # Errors
    ///
    /// Propaga `InvalidGeometry` si los constructores de `units` (cubiertos
    /// por sus propios tests) rechazaran estas constantes.
    pub fn illustrative_d16_like() -> Result<Self, KinematicsError> {
        Ok(Self::new(
            Length::from_m(0.075).map_err(|e| KinematicsError::InvalidGeometry {
                reason: e.to_string(),
            })?,
            Length::from_m(0.090).map_err(|e| KinematicsError::InvalidGeometry {
                reason: e.to_string(),
            })?,
            Length::from_m(0.137).map_err(|e| KinematicsError::InvalidGeometry {
                reason: e.to_string(),
            })?,
            9.4,
            "ILLUSTRATIVE preset: bore/stroke/CR are PENDING S01 candidates (FORUM-CLAIM), rod is MFR-TECH; not OEM nominal",
        )?)
    }

    #[must_use]
    pub fn bore(&self) -> Length {
        self.bore
    }

    #[must_use]
    pub fn stroke(&self) -> Length {
        self.stroke
    }

    #[must_use]
    pub fn rod_length(&self) -> Length {
        self.rod_length
    }

    #[must_use]
    pub fn compression_ratio(&self) -> f64 {
        self.compression_ratio
    }

    #[must_use]
    pub fn lambda_rod(&self) -> f64 {
        self.lambda_rod
    }

    #[must_use]
    pub fn displaced_m3(&self) -> f64 {
        self.displaced_m3
    }

    #[must_use]
    pub fn clearance_volume_m3(&self) -> f64 {
        self.clearance_volume_m3
    }

    #[must_use]
    pub fn source_note(&self) -> &'static str {
        self.source_note
    }

    /// Desplazamiento desde PMS a ángulo `theta_rad` (0 = PMS).
    #[must_use]
    pub fn displacement(&self, theta_rad: f64) -> f64 {
        let (s, c) = theta_rad.sin_cos();
        let r = self.crank_radius_m;
        let l = self.rod_length.as_m();
        let d = (l * l - r * r * s * s).sqrt();
        r * (1.0 - c) + l - d
    }

    /// Primera derivada analítica `dx/dθ` (m/rad).
    #[must_use]
    pub fn displacement_dtheta(&self, theta_rad: f64) -> f64 {
        let (s, c) = theta_rad.sin_cos();
        let r = self.crank_radius_m;
        let l = self.rod_length.as_m();
        let d = (l * l - r * r * s * s).sqrt();
        r * s + r * r * s * c / d
    }

    /// Segunda derivada analítica `d²x/dθ²` (m/rad²).
    #[must_use]
    pub fn displacement_d2theta(&self, theta_rad: f64) -> f64 {
        let (s, c) = theta_rad.sin_cos();
        let r = self.crank_radius_m;
        let l = self.rod_length.as_m();
        let d = (l * l - r * r * s * s).sqrt();
        let u = r * r * s * c;
        let u_prime = r * r * (2.0 * theta_rad).cos();
        let d_prime = -r * r * s * c / d;
        r * c + (u_prime * d - u * d_prime) / (d * d)
    }

    /// Volumen de cámara a `theta_rad`.
    #[must_use]
    pub fn volume(&self, theta_rad: f64) -> f64 {
        self.clearance_volume_m3 + self.piston_area_m2 * self.displacement(theta_rad)
    }

    /// `dV/dθ` analítica (m³/rad).
    #[must_use]
    pub fn dvdtheta(&self, theta_rad: f64) -> f64 {
        self.piston_area_m2 * self.displacement_dtheta(theta_rad)
    }

    /// Estado completo a ángulo y régimen dados (régimen estacionario:
    /// `dω/dt = 0`; el término transitorio llega con la dinámica S14).
    #[must_use]
    pub fn state(&self, theta_rad: f64, omega_rad_s: f64) -> SliderState {
        let x = self.displacement(theta_rad);
        let dx = self.displacement_dtheta(theta_rad);
        let d2x = self.displacement_d2theta(theta_rad);
        SliderState {
            x_m: x,
            v_m_s: dx * omega_rad_s,
            a_m_s2: d2x * omega_rad_s * omega_rad_s,
            v_m3: self.clearance_volume_m3 + self.piston_area_m2 * x,
            dvdtheta_m3_per_rad: self.piston_area_m2 * dx,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn pack() -> GeometryPack {
        GeometryPack::illustrative_d16_like().unwrap()
    }

    /// Diferencia finita centrada de 4º orden para contraste.
    fn fd4(f: &dyn Fn(f64) -> f64, x: f64, h: f64) -> f64 {
        (-f(x + 2.0 * h) + 8.0 * f(x + h) - 8.0 * f(x - h) + f(x - 2.0 * h)) / (12.0 * h)
    }

    #[test]
    fn tdc_bdc_positions_and_volumes() {
        let g = pack();
        // PMS: x = 0; PMI: x = S.
        assert!(g.displacement(0.0).abs() < 1e-15);
        assert!((g.displacement(std::f64::consts::PI) - 0.090).abs() < 1e-12);
        // Vmin = Vc, Vmax = Vc + Vd.
        assert!((g.volume(0.0) - g.clearance_volume_m3()).abs() < 1e-18);
        assert!(
            (g.volume(std::f64::consts::PI) - (g.clearance_volume_m3() + g.displaced_m3())).abs()
                < 1e-15
        );
        // dx/dθ = 0 en PMS y PMI (puntos muertos).
        assert!(g.displacement_dtheta(0.0).abs() < 1e-15);
        assert!(g.displacement_dtheta(std::f64::consts::PI).abs() < 1e-12);
    }

    #[test]
    fn illustrative_displacement_matches_expected_value() {
        // 4 × π/4 × 75² × 90 mm³ = 1590.43 cm³ (chequeo aritmético del
        // preset ilustrativo, no medición OEM).
        assert!((pack().displaced_m3() * 4.0 * 1e6 - 1590.43).abs() < 0.01);
    }

    #[test]
    fn symmetry_and_analytic_vs_finite_differences() {
        let g = pack();
        // Simetría x(θ) = x(−θ).
        for deg in [10.0f64, 45.0, 90.0, 135.0, 200.0, 350.0] {
            let t = deg.to_radians();
            assert!((g.displacement(t) - g.displacement(-t)).abs() < 1e-15);
        }
        // Derivadas analíticas contra FD4 en 25 ángulos.
        for i in 0..25 {
            let t = (i as f64) * 2.0 * std::f64::consts::PI / 25.0 + 0.07;
            let f = |x: f64| g.displacement(x);
            let num1 = fd4(&f, t, 1e-5);
            assert!(
                (g.displacement_dtheta(t) - num1).abs() < 1e-9,
                "dx/dθ mismatch at {t}"
            );
            let f1 = |x: f64| g.displacement_dtheta(x);
            let num2 = fd4(&f1, t, 1e-5);
            assert!(
                (g.displacement_d2theta(t) - num2).abs() < 1e-6,
                "d2x/dθ² mismatch at {t}"
            );
        }
    }

    #[test]
    fn impossible_mechanism_rejected() {
        // λ = r/l ≥ 1: biela más corta que el radio de manivela.
        let err = GeometryPack::new(
            Length::from_m(0.075).unwrap(),
            Length::from_m(0.090).unwrap(),
            Length::from_m(0.040).unwrap(),
            9.4,
            "test",
        );
        assert!(matches!(err, Err(KinematicsError::InvalidGeometry { .. })));
        assert!(GeometryPack::new(
            Length::from_m(0.075).unwrap(),
            Length::from_m(0.090).unwrap(),
            Length::from_m(0.137).unwrap(),
            1.0,
            "test",
        )
        .is_err());
    }

    #[test]
    fn velocity_acceleration_scale_with_omega() {
        let g = pack();
        let w = 750.0 * 2.0 * std::f64::consts::PI / 60.0;
        let s = g.state(1.0, w);
        // v y a escalan con ω y ω²; x, V, dV/dθ no dependen de ω.
        let s2 = g.state(1.0, 2.0 * w);
        assert!((s2.v_m_s - 2.0 * s.v_m_s).abs() < 1e-12);
        assert!((s2.a_m_s2 - 4.0 * s.a_m_s2).abs() < 1e-9);
        assert_eq!(s2.x_m, s.x_m);
        assert_eq!(s2.v_m3, s.v_m3);
    }
}
