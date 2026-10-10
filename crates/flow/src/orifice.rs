//! Orificio compresible (S12): flujo isoentrópico por garganta con
//! coeficiente de descarga y régimen estrangulado.
//!
//! - `pr = p_down/p_up ∈ (0, 1]`; si `pr ≤ pr_crit` el caudal se satura
//!   (choked, independiente de `p_down`).
//! - `pr_crit = (2/(γ+1))^(γ/(γ−1))`.
//! - Reversible: el llamante ordena up/down por presión; el signo lo pone
//!   `FlowSide`. A `pr → 1` el caudal → 0 de forma continua (sin
//!   discontinuidades numéricas falsas en apertura/cierre).

use thiserror::Error;

/// Error de orificio.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum OrificeError {
    /// Presiones/temperaturas/área/Cd no físicos.
    #[error("invalid orifice input: {reason}")]
    InvalidInput { reason: String },
}

/// Lado del flujo respecto al orden de llamada (el caudal es downwind).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowSide {
    Forward,
    Reverse,
}

/// Relación crítica de presión para γ dado.
#[must_use]
pub fn critical_ratio(gamma: f64) -> f64 {
    (2.0 / (gamma + 1.0)).powf(gamma / (gamma - 1.0))
}

/// Función de flujo Ψ(pr, γ): satura bajo el crítico, → 0 en pr → 1.
#[must_use]
pub fn psi(pr: f64, gamma: f64) -> f64 {
    let pr = pr.clamp(0.0, 1.0);
    let crit = critical_ratio(gamma);
    if pr <= crit {
        let t = 2.0 / (gamma + 1.0);
        gamma.sqrt() * t.powf((gamma + 1.0) / (2.0 * (gamma - 1.0)))
    } else {
        (2.0 * gamma / (gamma - 1.0) * (pr.powf(2.0 / gamma) - pr.powf((gamma + 1.0) / gamma)))
            .max(0.0)
            .sqrt()
    }
}

/// Caudal (kg/s, ≥ 0) de `p_up` a `p_down` por área efectiva `cd_area_m2`.
///
/// # Errors
///
/// `InvalidInput` si presiones ≤ 0, `t_up ≤ 0`, área/Cd negativos, R ≤ 0 o
/// γ ≤ 1 (todo finito exigido).
#[allow(clippy::too_many_arguments)]
pub fn mass_flow(
    p_up_pa: f64,
    t_up_k: f64,
    p_down_pa: f64,
    cd_area_m2: f64,
    r_gas: f64,
    gamma: f64,
) -> Result<f64, OrificeError> {
    let bad = |reason: &str| OrificeError::InvalidInput {
        reason: reason.to_owned(),
    };
    for (v, name) in [
        (p_up_pa, "p_up"),
        (t_up_k, "t_up"),
        (p_down_pa, "p_down"),
        (r_gas, "R"),
        (gamma, "gamma"),
    ] {
        if !(v.is_finite() && v > 0.0) {
            return Err(bad(name));
        }
    }
    if !(cd_area_m2.is_finite() && cd_area_m2 >= 0.0) {
        return Err(bad("cd_area"));
    }
    if gamma <= 1.0 {
        return Err(bad("gamma"));
    }
    if p_down_pa >= p_up_pa {
        return Ok(0.0);
    }
    let pr = p_down_pa / p_up_pa;
    Ok(cd_area_m2 * p_up_pa / (r_gas * t_up_k).sqrt() * psi(pr, gamma))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn choked_saturates_and_matches_incompressible_limit() {
        let crit = critical_ratio(1.4);
        assert!((crit - 0.528_281_787_7).abs() < 1e-9);
        // Saturación: por debajo del crítico el caudal no depende de p_down.
        let a = mass_flow(200_000.0, 300.0, 50_000.0, 1e-4, 287.0, 1.4).unwrap();
        let b = mass_flow(200_000.0, 300.0, 20_000.0, 1e-4, 287.0, 1.4).unwrap();
        assert!((a - b).abs() < 1e-15);
        // Límite incompresible a Δp → 0: CdA·√(2ρΔp), ρ = p/RT.
        let rho = 101_325.0 / (287.0 * 300.0);
        let c = mass_flow(101_325.0, 300.0, 101_324.0, 1e-4, 287.0, 1.4).unwrap();
        let theory = 1e-4 * (2.0 * rho * 1.0f64).sqrt();
        assert!(
            ((c - theory) / theory).abs() < 0.02,
            "c={c} theory={theory}"
        );
        // Cierre hermético: área 0 ⇒ 0.
        assert_eq!(
            mass_flow(200_000.0, 300.0, 100_000.0, 0.0, 287.0, 1.4).unwrap(),
            0.0
        );
    }

    #[test]
    fn monotone_in_pressure_drop() {
        // A mayor caída (menor pr), mayor caudal: el signo lo pone el
        // llamante ordenando up/down (antisimetría estructural).
        let m1 = mass_flow(150_000.0, 300.0, 140_000.0, 1e-4, 287.0, 1.4).unwrap();
        let m2 = mass_flow(150_000.0, 300.0, 100_000.0, 1e-4, 287.0, 1.4).unwrap();
        assert!(m2 > m1 && m1 > 0.0);
        assert!(mass_flow(100_000.0, 300.0, 150_000.0, 1e-4, 287.0, 1.4).unwrap() == 0.0);
    }
}
