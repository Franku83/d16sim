//! Ciclo cerrado ideal reversible (S09): compresión BDC→TDC y expansión
//! TDC→BDC con `dp/dθ = −γ·p/V·dV/dθ` integrada por RK4 de paso fijo.
//!
//! Referencia analítica: `p(θ) = p_ref·(V_ref/V(θ))^γ`. Trabajo analítico
//! de compresión `W = (p2·V2 − p1·V1)/(1−γ)` (trabajo hecho POR el gas;
//! en compresión es negativo). Red reversible ⇒ `W_neto ≈ 0`.

use crate::mixture::Mixture;
use kinematics::GeometryPack;
use thiserror::Error;

/// Error del ciclo cerrado.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum CycleError {
    /// Parámetros inválidos (paso, ángulos, referencia).
    #[error("invalid closed-cycle input: {reason}")]
    InvalidInput { reason: String },
}

/// Punto de la serie `p–θ`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CyclePoint {
    pub theta_deg: f64,
    pub p_pa: f64,
    pub v_m3: f64,
    pub t_k: f64,
}

/// Auditoría energética del ciclo.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Audit {
    /// Trabajo de compresión numérico (J, ≤ 0).
    pub w_comp_j: f64,
    /// Trabajo de compresión analítico (J).
    pub w_comp_analytic_j: f64,
    /// Error relativo de compresión.
    pub w_comp_rel_err: f64,
    /// Trabajo neto del ciclo reversible (J, ≈ 0).
    pub w_net_j: f64,
    /// Error relativo máximo de presión vs analítica.
    pub max_p_rel_err: f64,
    /// Orden de convergencia estimado.
    pub conv_order: f64,
}

/// Ciclo cerrado: geometría + mezcla + masa atrapada + referencia.
#[derive(Debug, Clone)]
pub struct ClosedCycle {
    geom: GeometryPack,
    mix: Mixture,
    m_kg: f64,
    /// (V_ref, p_ref): la referencia fija la constante adiabática.
    p_ref_pa: f64,
    v_ref_m3: f64,
}

impl ClosedCycle {
    /// Construye el ciclo (referencia en BDC de compresión, θ = 180°).
    ///
    /// # Errors
    ///
    /// `InvalidInput` si masa, presión o temperatura de referencia no son
    /// `> 0` finitas.
    pub fn new(
        geom: GeometryPack,
        mix: Mixture,
        p_bdc_pa: f64,
        t_bdc_k: f64,
    ) -> Result<Self, CycleError> {
        let bad = |reason: &str| CycleError::InvalidInput {
            reason: reason.to_owned(),
        };
        if !(p_bdc_pa.is_finite() && p_bdc_pa > 0.0) {
            return Err(bad("p_bdc must be finite and > 0"));
        }
        if !(t_bdc_k.is_finite() && t_bdc_k > 0.0) {
            return Err(bad("t_bdc must be finite and > 0"));
        }
        let theta_ref = std::f64::consts::PI;
        let v_ref = geom.volume(theta_ref);
        // Masa atrapada consistente con gas ideal en la referencia.
        let m = p_bdc_pa * v_ref / (mix.r() * t_bdc_k);
        Ok(Self {
            geom,
            mix,
            m_kg: m,
            p_ref_pa: p_bdc_pa,
            v_ref_m3: v_ref,
        })
    }

    #[must_use]
    pub fn trapped_mass_kg(&self) -> f64 {
        self.m_kg
    }

    /// Presión analítica a `theta_rad`.
    #[must_use]
    pub fn analytic_p(&self, theta_rad: f64) -> f64 {
        let v = self.geom.volume(theta_rad);
        self.p_ref_pa * (self.v_ref_m3 / v).powf(self.mix.gamma())
    }

    /// Temperatura analítica (gas ideal con masa atrapada).
    #[must_use]
    pub fn analytic_t(&self, theta_rad: f64) -> f64 {
        let v = self.geom.volume(theta_rad);
        self.analytic_p(theta_rad) * v / (self.m_kg * self.mix.r())
    }

    /// Derivada `dp/dθ` del modelo (primera ley, Q = 0).
    fn dpdtheta(&self, theta_rad: f64, p_pa: f64) -> f64 {
        -self.mix.gamma() * p_pa / self.geom.volume(theta_rad) * self.geom.dvdtheta(theta_rad)
    }

    /// Integra de `theta0` a `theta1` con RK4 de paso `dtheta_rad`.
    /// Retorna serie + trabajo ∫p·dV por Simpson sobre la malla (mismo
    /// orden que RK4; trapecios degradarían la auditoría a O(h²)).
    fn integrate(
        &self,
        theta0_rad: f64,
        theta1_rad: f64,
        dtheta_rad: f64,
    ) -> (Vec<CyclePoint>, f64) {
        let mut n = ((theta1_rad - theta0_rad) / dtheta_rad).ceil() as usize;
        if n % 2 == 1 {
            n += 1;
        }
        let h = (theta1_rad - theta0_rad) / n as f64;
        // Condición inicial consistente con la analítica en theta0.
        let mut p = self.analytic_p(theta0_rad);
        let mut th = theta0_rad;
        let mut points = Vec::with_capacity(n + 1);
        let mut integrand = Vec::with_capacity(n + 1);
        points.push(CyclePoint {
            theta_deg: th.to_degrees(),
            p_pa: p,
            v_m3: self.geom.volume(th),
            t_k: p * self.geom.volume(th) / (self.m_kg * self.mix.r()),
        });
        integrand.push(p * self.geom.dvdtheta(th));
        for _ in 0..n {
            let k1 = self.dpdtheta(th, p);
            let k2 = self.dpdtheta(th + h / 2.0, p + h * k1 / 2.0);
            let k3 = self.dpdtheta(th + h / 2.0, p + h * k2 / 2.0);
            let k4 = self.dpdtheta(th + h, p + h * k3);
            p += h * (k1 + 2.0 * k2 + 2.0 * k3 + k4) / 6.0;
            th += h;
            let v1 = self.geom.volume(th);
            points.push(CyclePoint {
                theta_deg: th.to_degrees(),
                p_pa: p,
                v_m3: v1,
                t_k: p * v1 / (self.m_kg * self.mix.r()),
            });
            integrand.push(p * self.geom.dvdtheta(th));
        }
        // Simpson compuesto (n par): W = ∫p·V′dθ.
        let mut work = integrand[0] + integrand[n];
        for (i, f) in integrand.iter().enumerate().take(n + 1).skip(1) {
            work += if i % 2 == 0 { 2.0 } else { 4.0 } * f;
        }
        work *= h / 3.0;
        (points, work)
    }

    /// Ejecuta compresión + expansión con paso `dtheta_deg` y audita.
    #[must_use]
    pub fn run(&self, dtheta_deg: f64) -> (Vec<CyclePoint>, Audit) {
        let h = dtheta_deg.to_radians();
        let pi = std::f64::consts::PI;
        let (mut comp, w_comp) = self.integrate(pi, 2.0 * pi, h);
        let (exp, w_exp) = self.integrate(2.0 * pi, 3.0 * pi, h);
        comp.extend(exp.into_iter().skip(1));
        // Analíticos.
        let v_bdc = self.v_ref_m3;
        let v_tdc = self.geom.volume(2.0 * pi);
        let p_tdc = self.analytic_p(2.0 * pi);
        let w_comp_an = (p_tdc * v_tdc - self.p_ref_pa * v_bdc) / (1.0 - self.mix.gamma());
        let mut max_err = 0.0f64;
        for pt in &comp {
            let pa = self.analytic_p(pt.theta_deg.to_radians());
            max_err = max_err.max((pt.p_pa - pa).abs() / pa);
        }
        let w_net = w_comp + w_exp;
        (
            comp,
            Audit {
                w_comp_j: w_comp,
                w_comp_analytic_j: w_comp_an,
                w_comp_rel_err: ((w_comp - w_comp_an) / w_comp_an).abs(),
                w_net_j: w_net,
                max_p_rel_err: max_err,
                conv_order: f64::NAN, // se rellena en `convergence()`
            },
        )
    }

    /// Estudio de convergencia: error máximo en mallas sucesivas (factor 2).
    /// Retorna `(errores, orden_estimado)` con orden = log2(e_h/e_h/2).
    #[must_use]
    pub fn convergence(&self, steps_deg: &[f64]) -> (Vec<f64>, f64) {
        let errs: Vec<f64> = steps_deg
            .iter()
            .map(|d| self.run(*d).1.max_p_rel_err)
            .collect();
        let order = if errs.len() >= 2 {
            let (a, b) = (errs[errs.len() - 2], errs[errs.len() - 1]);
            if b > 0.0 && a > 0.0 {
                (a / b).log2()
            } else {
                f64::NAN
            }
        } else {
            f64::NAN
        };
        (errs, order)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn cycle() -> ClosedCycle {
        let geom = GeometryPack::illustrative_d16_like().unwrap();
        let mix = Mixture::new(287.0, 1.35).unwrap();
        ClosedCycle::new(geom, mix, 101_325.0, 350.0).unwrap()
    }

    #[test]
    fn matches_ideal_within_justified_tolerance() {
        // Gate S09: error relativo máximo medido a 0.25°; la tolerancia
        // 1e-6 se justifica por esta medición (no al revés).
        let c = cycle();
        let (_, audit) = c.run(0.25);
        assert!(audit.max_p_rel_err < 1e-6, "err={}", audit.max_p_rel_err);
        assert!(audit.w_comp_rel_err < 1e-6);
        // Reversibilidad: trabajo neto ≈ 0 frente a escala de compresión.
        assert!((audit.w_net_j / audit.w_comp_analytic_j).abs() < 1e-6);
        // Positividad en toda la serie.
        let (pts, _) = c.run(1.0);
        assert!(pts
            .iter()
            .all(|p| p.p_pa > 0.0 && p.v_m3 > 0.0 && p.t_k > 0.0));
    }

    #[test]
    fn convergence_order_matches_rk4() {
        let c = cycle();
        let (errs, order) = c.convergence(&[1.0, 0.5, 0.25, 0.125]);
        for w in errs.windows(2) {
            assert!(w[1] < w[0], "error must decrease: {errs:?}");
        }
        assert!(
            order >= 3.0 && order <= 5.0,
            "RK4 order expected ~4, got {order} ({errs:?})"
        );
    }

    #[test]
    fn invalid_inputs_rejected() {
        let geom = GeometryPack::illustrative_d16_like().unwrap();
        let mix = Mixture::new(287.0, 1.35).unwrap();
        assert!(ClosedCycle::new(geom.clone(), mix, -1.0, 350.0).is_err());
        assert!(ClosedCycle::new(geom.clone(), mix, 101_325.0, 0.0).is_err());
        assert!(Mixture::new(287.0, 1.0).is_err());
        assert!(Mixture::new(-1.0, 1.35).is_err());
    }
}
