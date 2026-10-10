//! Perfil de leva y eventos (S11): ley de alzada genérica + parámetros de
//! duración del proxy, con procedencia explícita.
//!
//! - El Y7 stock (duración/alzada/LSA/centerline, tabla lift-vs-CA) es
//!   MISSING-DATA: no existe `CamSpec::y7_stock()`.
//! - `CamSpec::proxy_d16a6()` usa lobe lift medido (.225/.193" —
//!   FORUM-MEASURED), duraciones comunitarias y LSA reportado, con rocker
//!   documentado 1.60/1.80 y lash candidato. Etiqueta: ADAPTED_PROXY,
//!   nunca leva Y7 OEM ni tabla lift-vs-angle.
//! - La ley de forma `sin^flank` es ilustrativa y simétrica; subestima la
//!   duración a 0.050" frente al proxy (limitación declarada, no ajustada).

use thiserror::Error;

/// Error de distribución.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum CamError {
    /// Parámetros inválidos (duraciones, alzadas, LSA, exponente).
    #[error("invalid cam spec: {reason}")]
    InvalidSpec { reason: String },
}

/// Procedencia del perfil (siempre explícita).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CamProvenance {
    /// Proxy D16A6: transferencia documentada, no dato Y7.
    ProxyD16A6,
}

/// Especificación de leva por válvula (admisión o escape).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CamSpec {
    /// Duración advertised (grados de cigüeñal).
    pub adv_duration_crank_deg: f64,
    /// Duración a 0.050" (grados de cigüeñal, estándar USA).
    pub at050_duration_crank_deg: f64,
    /// Alzada de válvula tras rocker y lash (mm).
    pub valve_lift_mm: f64,
    /// Centerline del lóbulo (° ATDC del PMS propio; asumido si se indica).
    pub centerline_atdc_deg: f64,
    /// LSA (°).
    pub lsa_deg: f64,
    /// Exponente de forma `sin^flank` (2 = sen² simétrico ilustrativo).
    pub flank: f64,
    pub provenance: CamProvenance,
}

/// Eventos de válvula en grados de cigüeñal del ciclo local propio.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValveEvents {
    pub open_deg: f64,
    pub close_deg: f64,
}

impl CamSpec {
    /// Proxy D16A6 (research donantes §1): adv 222/224°, @.050 194/196°,
    /// lobe lift medido .225/.193" (Mista Bone 2001, FORUM-MEASURED),
    /// valve lift = lobe × rocker (1.60/1.80) − lash medio ilustrativo
    /// (0.20/0.25 mm), LSA 110°, ICL 110° asumido y declarado.
    ///
    /// # Errors
    ///
    /// Nunca falla con estas constantes (validadas abajo); el `Result`
    /// existe para futuros perfiles medidos.
    pub fn proxy_d16a6_intake() -> Result<Self, CamError> {
        Self::checked(
            222.0,
            194.0,
            0.225 * 25.4 * 1.60 - 0.20,
            110.0,
            110.0,
            2.0,
            CamProvenance::ProxyD16A6,
        )
    }

    /// Proxy D16A6 escape: adv 224°, @.050 196°, lobe .193", rocker 1.80,
    /// lash 0.25 mm, ECL 110° BTDC del PMS propio (expresado como centro).
    ///
    /// # Errors
    ///
    /// Igual que admisión.
    pub fn proxy_d16a6_exhaust() -> Result<Self, CamError> {
        Self::checked(
            224.0,
            196.0,
            0.193 * 25.4 * 1.80 - 0.25,
            110.0,
            110.0,
            2.0,
            CamProvenance::ProxyD16A6,
        )
    }

    fn checked(
        adv: f64,
        at050: f64,
        lift_mm: f64,
        centerline: f64,
        lsa: f64,
        flank: f64,
        provenance: CamProvenance,
    ) -> Result<Self, CamError> {
        let bad = |reason: &str| CamError::InvalidSpec {
            reason: reason.to_owned(),
        };
        if !(adv.is_finite() && adv > 0.0 && adv < 360.0) {
            return Err(bad("advertised duration must be in (0, 360)"));
        }
        if !(at050.is_finite() && at050 > 0.0 && at050 <= adv) {
            return Err(bad("at050 duration must be in (0, adv]"));
        }
        if !(lift_mm.is_finite() && lift_mm > 0.0) {
            return Err(bad("valve lift must be finite and > 0"));
        }
        if !(flank.is_finite() && flank > 0.0) {
            return Err(bad("flank exponent must be finite and > 0"));
        }
        Ok(Self {
            adv_duration_crank_deg: adv,
            at050_duration_crank_deg: at050,
            valve_lift_mm: lift_mm,
            centerline_atdc_deg: centerline,
            lsa_deg: lsa,
            flank: flank,
            provenance,
        })
    }

    /// Eventos advertised RELATIVOS al centro: apertura = −adv/2,
    /// cierre = +adv/2 (el llamante suma el centerline una sola vez).
    /// (Aproximación de catálogo; el cruce real a lash medido es S11+.)
    #[must_use]
    pub fn advertised_events(&self) -> ValveEvents {
        ValveEvents {
            open_deg: -self.adv_duration_crank_deg / 2.0,
            close_deg: self.adv_duration_crank_deg / 2.0,
        }
    }

    /// Alzada (mm) a ángulo de cigüeñal relativo al centro del lóbulo
    /// (`relativ_deg = 0` en el centro). Ley ilustrativa simétrica. El
    /// relativo se envuelve a (−360°, 360°]: la leva abre una vez por ciclo
    /// aunque la ventana cruce el 0°/720° (sin envoltura se perdería la
    /// apertura a caballo del origen).
    #[must_use]
    pub fn lift_at(&self, relative_crank_deg: f64) -> f64 {
        let half = self.adv_duration_crank_deg / 2.0;
        // Ventana en grados de LEVA (la mitad): la forma vive en el árbol.
        let rel = (relative_crank_deg + 360.0).rem_euclid(720.0) - 360.0;
        let u = (rel / half + 1.0) / 2.0;
        if u <= 0.0 || u >= 1.0 {
            0.0
        } else {
            self.valve_lift_mm * (std::f64::consts::PI * u).sin().powf(self.flank)
        }
    }

    /// Cruces a altura `height_mm` (bisección determinista, 60 iteraciones).
    /// Retorna `(apertura, cierre)` relativos al centro, o `None` si la
    /// altura supera el máximo (sin falsos eventos).
    #[must_use]
    pub fn crossings_at(&self, height_mm: f64) -> Option<ValveEvents> {
        if !(height_mm.is_finite() && height_mm > 0.0) || height_mm >= self.valve_lift_mm {
            return None;
        }
        let half = self.adv_duration_crank_deg / 2.0;
        let f = |x: f64| self.lift_at(x) - height_mm;
        let mut lo = -half;
        let mut hi = 0.0;
        for _ in 0..60 {
            let mid = (lo + hi) / 2.0;
            if f(mid) > 0.0 {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        let open = (lo + hi) / 2.0;
        let mut lo2 = 0.0;
        let mut hi2 = half;
        for _ in 0..60 {
            let mid = (lo2 + hi2) / 2.0;
            if f(mid) > 0.0 {
                lo2 = mid;
            } else {
                hi2 = mid;
            }
        }
        let close = (lo2 + hi2) / 2.0;
        Some(ValveEvents {
            open_deg: open,
            close_deg: close,
        })
    }
}

/// Overlap advertised (°): `(adv_i + adv_e)/2 − 2·LSA`.
#[must_use]
pub fn overlap_advertised_deg(intake: &CamSpec, exhaust: &CamSpec) -> f64 {
    (intake.adv_duration_crank_deg + exhaust.adv_duration_crank_deg) / 2.0 - 2.0 * intake.lsa_deg
}

/// Overlap a 0.050" (°).
#[must_use]
pub fn overlap_at050_deg(intake: &CamSpec, exhaust: &CamSpec) -> f64 {
    (intake.at050_duration_crank_deg + exhaust.at050_duration_crank_deg) / 2.0
        - 2.0 * intake.lsa_deg
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn proxy_matches_donor_arithmetic() {
        // Aritmética del research donantes §1.5: overlap advertised ≈ 3°,
        // overlap @.050 = −25° (sin overlap, leva mansa).
        let i = CamSpec::proxy_d16a6_intake().unwrap();
        let e = CamSpec::proxy_d16a6_exhaust().unwrap();
        assert!((overlap_advertised_deg(&i, &e) - 3.0).abs() < 1e-12);
        assert!((overlap_at050_deg(&i, &e) + 25.0).abs() < 1e-12);
        // Valve lift derivado: .225"×1.60−0.20 mm ≈ 8.94 mm.
        assert!((i.valve_lift_mm - 8.944).abs() < 0.01);
        assert!((e.valve_lift_mm - 8.572).abs() < 0.01);
        assert_eq!(i.provenance, CamProvenance::ProxyD16A6);
    }

    #[test]
    fn lift_shape_is_self_consistent_not_tuned() {
        // La forma sen² es ilustrativa: sus cruces a 0.050" se verifican
        // contra la inversión analítica de LA FORMA (autoconsistencia),
        // no contra el proxy 194° (limitación declarada: la forma simétrica
        // subestima la duración real a media alzada).
        let i = CamSpec::proxy_d16a6_intake().unwrap();
        let h = 0.050 * 25.4;
        let cross = i.crossings_at(h).unwrap();
        let frac = (h / i.valve_lift_mm).powf(1.0 / i.flank);
        let u1 = frac.asin() / std::f64::consts::PI;
        let half = i.adv_duration_crank_deg / 2.0;
        let expected_open = (2.0 * u1 - 1.0) * half;
        assert!((cross.open_deg - expected_open).abs() < 1e-9);
        assert!((cross.close_deg + expected_open).abs() < 1e-9);
        // Cero fuera de ventana; None sobre el máximo.
        assert_eq!(i.lift_at(-200.0), 0.0);
        assert_eq!(i.lift_at(200.0), 0.0);
        assert!(i.crossings_at(99.0).is_none());
        // Distancia honesta al proxy (no es test de igualdad).
        let dur050 = cross.close_deg - cross.open_deg;
        assert!((dur050 - 167.0).abs() < 2.0, "shape gives {dur050}");
    }

    #[test]
    fn invalid_specs_rejected() {
        assert!(CamSpec::proxy_d16a6_intake().is_ok());
        // Duración @.050 mayor que advertised: imposible.
        assert!(CamSpec::checked(
            222.0,
            300.0,
            8.0,
            110.0,
            110.0,
            2.0,
            CamProvenance::ProxyD16A6
        )
        .is_err());
        assert!(CamSpec::checked(
            222.0,
            194.0,
            -1.0,
            110.0,
            110.0,
            2.0,
            CamProvenance::ProxyD16A6
        )
        .is_err());
    }
}
