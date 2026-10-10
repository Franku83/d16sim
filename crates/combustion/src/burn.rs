//! Ciclo con liberación de calor (S10): zona única,
//! `dp/dθ = −γ·p/V·dV/dθ + (γ−1)/V·dQ/dθ`,
//! `dQ/dθ = m_fuel·LHV·eff·dx/dθ` (contada una sola vez), `Q_pared = 0`.
//!
//! De la presión integrada derivan torque indicado `p·dV/dθ`, trabajo neto
//! e IMEP. Sin gas exchange, sin fricción, sin dyno al freno.

use crate::wiebe::{Wiebe, WiebeError};
use kinematics::GeometryPack;
use thermo::Mixture;
use thiserror::Error;

/// Error del ciclo con combustión.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum BurnError {
    /// Parámetros inválidos (combustible, referencia, Wiebe).
    #[error("invalid burn input: {reason}")]
    InvalidInput { reason: String },
}

/// Calor a paredes: S10 solo adiabático; la variante L1 es costura futura.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallHeat {
    Adiabatic,
}

/// Parámetros de combustión (todo explícito, todo ilustrativo si no medido).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BurnParams {
    /// Masa de combustible por ciclo (kg, ≥ 0; 0 = motored).
    pub fuel_mass_kg: f64,
    /// Poder calorífico inferior (J/kg, > 0).
    pub lower_heating_value_j_per_kg: f64,
    /// Eficiencia de combustión (0, 1].
    pub combustion_efficiency: f64,
    /// Pared (solo adiabático en S10).
    pub wall: WallHeat,
}

/// Auditoría del ciclo con combustión.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BurnAudit {
    /// Trabajo neto indicado (J).
    pub w_net_j: f64,
    /// IMEP neto (Pa) = W/Vd.
    pub imep_pa: f64,
    /// Presión máxima (Pa) y su ángulo (°).
    pub pmax_pa: f64,
    pub pmax_theta_deg: f64,
    /// Calor liberado objetivo `m·LHV·eff` (J).
    pub q_target_j: f64,
    /// Calor integrado numéricamente (J).
    pub q_integrated_j: f64,
    /// Error relativo del calor.
    pub q_rel_err: f64,
    /// Trabajo del mismo ciclo sin combustible (J, ≈ 0 reversible).
    pub w_motored_j: f64,
}

/// Ciclo con Wiebe: geometría + mezcla + referencia + combustión.
#[derive(Debug, Clone)]
pub struct BurnCycle {
    geom: GeometryPack,
    gamma: f64,
    m_air_kg: f64,
    p_bdc_pa: f64,
    wiebe: Wiebe,
    params: BurnParams,
}

impl BurnCycle {
    /// Construye validando referencia, combustible y Wiebe.
    ///
    /// # Errors
    ///
    /// `InvalidInput` si p/T BDC, combustible o LHV/eff violan dominio.
    pub fn new(
        geom: GeometryPack,
        mix: Mixture,
        p_bdc_pa: f64,
        t_bdc_k: f64,
        wiebe: Wiebe,
        params: BurnParams,
    ) -> Result<Self, BurnError> {
        let bad = |reason: &str| BurnError::InvalidInput {
            reason: reason.to_owned(),
        };
        if !(p_bdc_pa.is_finite() && p_bdc_pa > 0.0) {
            return Err(bad("p_bdc must be finite and > 0"));
        }
        if !(t_bdc_k.is_finite() && t_bdc_k > 0.0) {
            return Err(bad("t_bdc must be finite and > 0"));
        }
        if !(params.fuel_mass_kg.is_finite() && params.fuel_mass_kg >= 0.0) {
            return Err(bad("fuel mass must be finite and >= 0"));
        }
        if !(params.lower_heating_value_j_per_kg.is_finite()
            && params.lower_heating_value_j_per_kg > 0.0)
        {
            return Err(bad("LHV must be finite and > 0"));
        }
        if !(params.combustion_efficiency.is_finite()
            && params.combustion_efficiency > 0.0
            && params.combustion_efficiency <= 1.0)
        {
            return Err(bad("efficiency must be in (0, 1]"));
        }
        if params.wall != WallHeat::Adiabatic {
            return Err(bad("only adiabatic wall in S10"));
        }
        let theta_bdc = std::f64::consts::PI;
        let v_bdc = geom.volume(theta_bdc);
        let m_air = p_bdc_pa * v_bdc / (mix.r() * t_bdc_k);
        Ok(Self {
            geom,
            gamma: mix.gamma(),
            m_air_kg: m_air,
            p_bdc_pa,
            wiebe,
            params,
        })
    }

    /// Masa de aire atrapada (kg, gas ideal en la referencia BDC).
    #[must_use]
    pub fn trapped_air_kg(&self) -> f64 {
        self.m_air_kg
    }

    /// `dQ/dθ` (J/rad) a `theta_rad`: contada una sola vez vía `dx/dθ`.
    fn dqdtheta(&self, theta_rad: f64) -> f64 {
        self.params.fuel_mass_kg
            * self.params.lower_heating_value_j_per_kg
            * self.params.combustion_efficiency
            * self.wiebe.burned_rate(theta_rad)
    }

    /// Integra BDC(180°)→BDC(540°) con RK4; serie `p` + trabajo por Simpson.
    /// El calor se audita por separado (`heat_release_integral`): la tasa de
    /// Wiebe tiene un kink en los bordes de ventana y la cuadratura sobre la
    /// malla degradaría el orden.
    fn integrate(&self, dtheta_rad: f64, fuel_scale: f64) -> (Vec<(f64, f64)>, f64) {
        let (t0, t1) = (std::f64::consts::PI, 3.0 * std::f64::consts::PI);
        let mut n = ((t1 - t0) / dtheta_rad).ceil() as usize;
        if n % 2 == 1 {
            n += 1;
        }
        let h = (t1 - t0) / n as f64;
        // Condición inicial: quemado nulo antes de θs ⇒ adiabática pura en BDC.
        let mut p = self.p_bdc_pa;
        let mut th = t0;
        let mut series = Vec::with_capacity(n + 1);
        let mut fw = Vec::with_capacity(n + 1);
        series.push((th, p));
        fw.push(p * self.geom.dvdtheta(th));
        for _ in 0..n {
            // RK4 con la escala de combustible aplicada al término fuente.
            let src = |t: f64, pp: f64| {
                let v = self.geom.volume(t);
                -self.gamma * pp / v * self.geom.dvdtheta(t)
                    + (self.gamma - 1.0) / v * fuel_scale * self.dqdtheta(t)
            };
            let k1 = src(th, p);
            let k2 = src(th + h / 2.0, p + h * k1 / 2.0);
            let k3 = src(th + h / 2.0, p + h * k2 / 2.0);
            let k4 = src(th + h, p + h * k3);
            p += h * (k1 + 2.0 * k2 + 2.0 * k3 + k4) / 6.0;
            th += h;
            series.push((th, p));
            fw.push(p * self.geom.dvdtheta(th));
        }
        (series, simpson(&fw, h))
    }

    /// Calor liberado total por Simpson por tramos suaves: la tasa de Wiebe
    /// tiene un kink de derivada en los bordes de ventana y la cuadratura
    /// sobre la malla uniforme degradaría el orden. Cada tramo C-suave se
    /// integra con 720 paneles.
    /// Calor liberado total por Gauss-Legendre (8 puntos) por tramos
    /// suaves: la tasa de Wiebe salta en el borde de ventana (tasa finita a
    /// la izquierda, 0 a la derecha), así que Simpson —que evalúa los
    /// extremos— hereda un artefacto O(h·salto) ~ 1e-4. GL no muestrea
    /// extremos y audita el calor con error despreciable.
    fn heat_release_integral(&self) -> f64 {
        // Nodos y pesos de Gauss-Legendre n=8 en (−1, 1) (estándar).
        const X: [f64; 8] = [
            -0.183_434_642_495_649_8,
            0.183_434_642_495_649_8,
            -0.525_532_409_916_329_0,
            0.525_532_409_916_329_0,
            -0.796_666_477_413_626_7,
            0.796_666_477_413_626_7,
            -0.960_289_856_497_536_2,
            0.960_289_856_497_536_2,
        ];
        const W: [f64; 8] = [
            0.362_683_783_378_362_0,
            0.362_683_783_378_362_0,
            0.313_706_645_877_887_3,
            0.313_706_645_877_887_3,
            0.222_381_034_453_374_5,
            0.222_381_034_453_374_5,
            0.101_228_536_290_376_3,
            0.101_228_536_290_376_3,
        ];
        let (t0, t1) = (std::f64::consts::PI, 3.0 * std::f64::consts::PI);
        let ts = self.wiebe.theta_start_rad();
        let te = self.wiebe.end_rad();
        let mut bounds = vec![t0];
        if ts > t0 && ts < t1 {
            bounds.push(ts);
        }
        if te > t0 && te < t1 {
            bounds.push(te);
        }
        bounds.push(t1);
        let mut total = 0.0;
        for w in bounds.windows(2) {
            let (a, b) = (w[0], w[1]);
            let (mid, half) = ((a + b) / 2.0, (b - a) / 2.0);
            let mut s = 0.0;
            for (x, ww) in X.iter().zip(W.iter()) {
                s += ww * self.dqdtheta(mid + half * x);
            }
            total += s * half;
        }
        total
    }

    /// Ejecuta y audita con paso `dtheta_deg`.
    #[must_use]
    pub fn run(&self, dtheta_deg: f64) -> (Vec<(f64, f64)>, BurnAudit) {
        let h = dtheta_deg.to_radians();
        let (series, w_net) = self.integrate(h, 1.0);
        let (_, w_mot) = self.integrate(h, 0.0);
        let q_int = self.heat_release_integral();
        let (mut pmax, mut pmax_th) = (0.0, 0.0);
        for (th, p) in &series {
            if *p > pmax {
                pmax = *p;
                pmax_th = *th;
            }
        }
        // Objetivo de calor con la completitud real de la ventana usada
        // (`burned_fraction` al final), no con un `a` hardcodeado.
        let completeness = self.wiebe.burned_fraction(self.wiebe.end_rad() + 1.0);
        let q_target_exact = self.params.fuel_mass_kg
            * self.params.lower_heating_value_j_per_kg
            * self.params.combustion_efficiency
            * completeness;
        let audit = BurnAudit {
            w_net_j: w_net,
            imep_pa: w_net / self.geom.displaced_m3(),
            pmax_pa: pmax,
            pmax_theta_deg: pmax_th.to_degrees(),
            q_target_j: q_target_exact,
            q_integrated_j: q_int,
            q_rel_err: if q_target_exact > 0.0 {
                ((q_int - q_target_exact) / q_target_exact).abs()
            } else {
                0.0
            },
            w_motored_j: w_mot,
        };
        (series, audit)
    }
}

/// Simpson compuesto (n par implícito por construcción del llamante).
fn simpson(f: &[f64], h: f64) -> f64 {
    let n = f.len() - 1;
    let mut s = f[0] + f[n];
    for (i, v) in f.iter().enumerate().take(n + 1).skip(1) {
        s += if i % 2 == 0 { 2.0 } else { 4.0 } * v;
    }
    s * h / 3.0
}

impl From<WiebeError> for BurnError {
    fn from(_: WiebeError) -> Self {
        BurnError::InvalidInput {
            reason: "wiebe".to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn cycle(start_deg: f64, fuel_mg: f64) -> BurnCycle {
        let geom = GeometryPack::illustrative_d16_like().unwrap();
        let mix = thermo::Mixture::new(287.0, 1.35).unwrap();
        let wiebe = Wiebe::new(5.0, 2.0, start_deg.to_radians(), 60f64.to_radians()).unwrap();
        BurnCycle::new(
            geom,
            mix,
            101_325.0,
            350.0,
            wiebe,
            BurnParams {
                fuel_mass_kg: fuel_mg * 1e-6,
                lower_heating_value_j_per_kg: 44e6,
                combustion_efficiency: 1.0,
                wall: WallHeat::Adiabatic,
            },
        )
        .unwrap()
    }

    #[test]
    fn retarding_heat_release_shifts_and_reduces_work() {
        // Gate S10: retrasar θs desplaza pmax más tarde y reduce el trabajo,
        // de forma monótona y explicable (misma masa, misma ventana).
        let a = cycle(340.0, 25.0).run(0.25);
        let b = cycle(350.0, 25.0).run(0.25);
        let c = cycle(360.0, 25.0).run(0.25);
        assert!(a.1.w_net_j > b.1.w_net_j, "W must fall when retarding");
        assert!(b.1.w_net_j > c.1.w_net_j, "W must fall when retarding");
        assert!(a.1.pmax_theta_deg < b.1.pmax_theta_deg);
        assert!(b.1.pmax_theta_deg < c.1.pmax_theta_deg);
        // El calor liberado es el mismo (misma masa): solo cambia el cuándo.
        assert!((a.1.q_target_j - b.1.q_target_j).abs() < 1e-9);
        assert!(a.1.q_rel_err < 1e-6);
        // IMEP ilustrativo positivo y razonable en orden de magnitud.
        assert!(a.1.imep_pa > 0.0);
    }

    #[test]
    fn zero_fuel_recovers_motored_and_inputs_rejected() {
        // Sin combustible: calor nulo y neto reversible ≈ 0 (hereda S09).
        let zero = cycle(340.0, 0.0).run(0.25);
        assert_eq!(zero.1.q_target_j, 0.0);
        assert_eq!(zero.1.q_integrated_j, 0.0);
        assert!(zero.1.w_net_j.abs() < 1e-3);
        // Con combustible: el neto supera ampliamente al motored.
        let fired = cycle(340.0, 25.0).run(0.25);
        assert!(fired.1.w_net_j > 100.0 * fired.1.w_motored_j.abs().max(1e-9));
        assert!(fired.1.q_target_j > 0.0);
        let geom = GeometryPack::illustrative_d16_like().unwrap();
        let mix = thermo::Mixture::new(287.0, 1.35).unwrap();
        let wiebe = Wiebe::new(5.0, 2.0, 0.0, 1.0).unwrap();
        let bad_params = BurnParams {
            fuel_mass_kg: -1e-6,
            lower_heating_value_j_per_kg: 44e6,
            combustion_efficiency: 1.0,
            wall: WallHeat::Adiabatic,
        };
        assert!(BurnCycle::new(geom, mix, 101_325.0, 350.0, wiebe, bad_params).is_err());
    }
}
