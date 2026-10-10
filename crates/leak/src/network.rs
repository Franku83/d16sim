//! Red de fugas (S23): dominios de fluido + puertos con conductancia,
//! integración con balances auditados de masa y energía.
//!
//! Fluidos: gas (orificio compresible S12), aceite y refrigerante
//! (laminar `Q = k·Δp/μ`, explícito L1). Entalpías por fluido con cp
//! declarado. La red es estática en estructura; el tiempo solo integra.

use thiserror::Error;

/// Error de red.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum LeakError {
    /// Topología o estado inválidos (fail-closed).
    #[error("invalid leak network/state: {reason}")]
    Invalid { reason: String },
}

/// Fluido de un dominio (cp ilustrativo declarado por dominio).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fluid {
    /// Gas (aire/residual): R y cp dados.
    Gas { r_gas: f64, cp: f64 },
    /// Aceite (cp ilustrativo 1900 J/kg·K).
    Oil,
    /// Refrigerante (cp ilustrativo 3500 J/kg·K).
    Coolant,
}

impl Fluid {
    /// cp (J/kg·K) declarado por fluido.
    #[must_use]
    pub fn cp(&self) -> f64 {
        match *self {
            Fluid::Gas { cp, .. } => cp,
            Fluid::Oil => 1900.0,
            Fluid::Coolant => 3500.0,
        }
    }

    /// ¿Flujo compresible (gas) o laminar (líquidos)?
    #[must_use]
    pub fn is_gas(&self) -> bool {
        matches!(*self, Fluid::Gas { .. })
    }

    /// Densidad (kg/m³) de líquidos para convertir caudal volumétrico a
    /// masa (R01-F09). `None` en gas: la rama compresible ya devuelve masa
    /// directamente y no debe aplicar densidad dos veces.
    #[must_use]
    pub fn liquid_density_kg_m3(&self) -> Option<f64> {
        match *self {
            Fluid::Gas { .. } => None,
            // Valores ilustrativos declarados (aceite 870, refrigerante 1060);
            // S25 los medirá contra banco. No se usan como calibración.
            Fluid::Oil => Some(870.0),
            Fluid::Coolant => Some(1060.0),
        }
    }
}

/// Dominio de fluido (volumen + estado homogéneo).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LeakDomain {
    /// Volumen (m³, fijo en S23).
    pub v_m3: f64,
    /// Masa (kg).
    pub m_kg: f64,
    /// Temperatura (K).
    pub t_k: f64,
    pub fluid: Fluid,
    /// Fracción fresca (solo gas; 1.0 en líquidos por convención).
    pub fresh_frac: f64,
}

impl LeakDomain {
    /// Presión (Pa): gas ideal, o 0.0 si masa nula; líquidos: presión
    /// impuesta externamente (campo `p_set_pa`).
    #[must_use]
    pub fn pressure(&self, p_set_pa: f64) -> f64 {
        match self.fluid {
            Fluid::Gas { r_gas, .. } => {
                if self.m_kg <= 0.0 {
                    0.0
                } else {
                    self.m_kg * r_gas * self.t_k / self.v_m3
                }
            }
            Fluid::Oil | Fluid::Coolant => p_set_pa,
        }
    }
}

/// Puerto de fuga dirigido con conductancia efectiva.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LeakPort {
    pub from: usize,
    pub to: usize,
    /// Área efectiva (m²) para gas, o permeabilidad k (m³) para laminar.
    pub conductance: f64,
    /// true = orificio compresible (gas–gas), false = laminar.
    pub compressible: bool,
}

/// Red + condiciones de borde.
#[derive(Debug, Clone, PartialEq)]
pub struct LeakNetwork {
    pub domains: Vec<LeakDomain>,
    /// Presión impuesta por dominio líquido (Pa); ignorado en gas.
    pub p_set_pa: Vec<f64>,
    pub ports: Vec<LeakPort>,
    /// Viscosidad para laminar (Pa·s, p. ej. aceite a T).
    pub mu_liquid_pa_s: f64,
    /// Temperatura del aire de suministro del banco (K).
    pub supply_t_k: f64,
}

/// Auditoría de una corrida de leak-down con suministro regulado.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LeakAudit {
    /// Fuga estacionaria (g/s de suministro para mantener la prueba).
    pub steady_leak_g_s: f64,
    /// Fracción del total por el puerto 0 (reparto).
    pub split_first: f64,
    /// Cierre de masa relativo (con frontera contabilizada).
    pub mass_closure: f64,
    /// Cierre de energía relativo (con frontera contabilizada).
    pub energy_closure: f64,
    /// Cierre de la especie fresca relativo (advección por puertos,
    /// suministro fresco y venteo) — R01-F08.
    pub fresh_closure: f64,
}

/// Integra la red `duration_s` con suministro regulado que mantiene la
/// fuente a su presión inicial (como un tester real: se mide el caudal de
/// suministro en régimen, no el vaciado). Euler explícito con balances
/// auditados de masa y energía (frontera contabilizada).
///
/// # Errors
///
/// `Invalid` si la topología mezcla regímenes, hay índices mal, masas/T
/// no positivas, un nodo se vaciaría (paso excesivo o config imposible:
/// fail-closed, nunca salto silencioso) o el suministro no es finito.
pub fn run_leakdown(
    net: &LeakNetwork,
    source: usize,
    duration_s: f64,
    dt_s: f64,
) -> Result<(Vec<LeakDomain>, LeakAudit, Vec<f64>), LeakError> {
    let bad = LeakError::Invalid {
        reason: "leak network".to_owned(),
    };
    if net.domains.is_empty() || net.domains.len() != net.p_set_pa.len() {
        return Err(bad);
    }
    if !(duration_s.is_finite() && duration_s > 0.0 && dt_s.is_finite() && dt_s > 0.0) {
        return Err(bad);
    }
    if source >= net.domains.len() {
        return Err(bad);
    }
    for p in &net.ports {
        if p.from >= net.domains.len() || p.to >= net.domains.len() || p.from == p.to {
            return Err(bad);
        }
        if !(p.conductance.is_finite() && p.conductance >= 0.0) {
            return Err(bad);
        }
        let gf = net.domains[p.from].fluid.is_gas();
        // Régimen por el fluido de ORIGEN: el gas impone orificio
        // compresible (aunque descargue en líquido); líquido-líquido o
        // líquido-gas por lado de salida usan laminar con μ del origen.
        if p.compressible != gf {
            return Err(bad);
        }
    }
    if !(net.mu_liquid_pa_s.is_finite() && net.mu_liquid_pa_s > 0.0) {
        return Err(bad);
    }
    if !(net.supply_t_k.is_finite() && net.supply_t_k > 0.0) {
        return Err(bad);
    }
    let mut d: Vec<LeakDomain> = net.domains.clone();
    for x in &d {
        if !(x.m_kg.is_finite() && x.m_kg > 0.0 && x.t_k.is_finite() && x.t_k > 0.0) {
            return Err(bad);
        }
    }
    // Presión de prueba = presión inicial de la fuente (regulador ideal).
    let p_test_pa = d[source].pressure(net.p_set_pa[source]);
    if !(p_test_pa.is_finite() && p_test_pa > 0.0) {
        return Err(bad);
    }
    let (r_src, cp_src) = match d[source].fluid {
        Fluid::Gas { r_gas, cp } => (r_gas, cp),
        _ => return Err(bad),
    };
    let m0: f64 = d.iter().map(|x| x.m_kg).sum();
    let u0: f64 = d.iter().map(|x| x.m_kg * cv_of(x.fluid) * x.t_k).sum();
    let fresh0: f64 = d.iter().map(|x| x.m_kg * x.fresh_frac).sum();
    let mut port_totals = vec![0.0; net.ports.len()];
    let mut supply_mass_kg = 0.0;
    let mut supply_enth_j = 0.0;
    let mut supply_fresh_kg = 0.0;
    let mut vent_mass_kg = 0.0;
    let mut vent_enth_j = 0.0;
    let mut vent_fresh_kg = 0.0;
    // Serie del suministro para el estacionario (último 20%).
    let mut supply_series: Vec<f64> = Vec::new();
    let mut t = 0.0;
    while t < duration_s - 1e-15 {
        let h = dt_s.min(duration_s - t);
        // Presiones al inicio del paso (splitting explícito).
        let ps: Vec<f64> = d
            .iter()
            .enumerate()
            .map(|(i, x)| x.pressure(net.p_set_pa[i]))
            .collect();
        for (pi, port) in net.ports.iter().enumerate() {
            let (pa, pb) = (ps[port.from], ps[port.to]);
            if pa <= pb {
                continue;
            }
            let (dm, h_flow) = if port.compressible {
                let fa = &d[port.from];
                let (r, cp) = match fa.fluid {
                    Fluid::Gas { r_gas, cp } => (r_gas, cp),
                    _ => return Err(bad),
                };
                let m = flow::mass_flow(pa, fa.t_k, pb, port.conductance, r, 1.4)
                    .map_err(|_| bad.clone())?;
                (m * h, m * h * cp * fa.t_k)
            } else {
                let fa = &d[port.from];
                // Caudal volumétrico laminar Q = k·Δp/μ (m³/s) → masa con
                // la densidad del líquido (R01-F09). No aplicar dos veces en
                // gas: la rama compresible ya devuelve kg/s.
                let rho = fa.fluid.liquid_density_kg_m3().ok_or_else(|| bad.clone())?;
                let vol = port.conductance * (pa - pb) / net.mu_liquid_pa_s * h;
                let m = vol * rho;
                (m, m * fa.fluid.cp() * fa.t_k)
            };
            if !(dm.is_finite() && h_flow.is_finite() && dm >= 0.0) {
                return Err(bad);
            }
            // Conservación exacta por construcción (lo que sale entra):
            // préstamos disjuntos (from ≠ to validado).
            let (lo, hi) = if port.from < port.to {
                let (a, b) = d.split_at_mut(port.to);
                (&mut a[port.from], &mut b[0])
            } else {
                let (a, b) = d.split_at_mut(port.from);
                (&mut b[0], &mut a[port.to])
            };
            let (a, b) = (lo, hi);
            // Fail-closed: vaciar un nodo es paso excesivo o config
            // imposible, nunca salto silencioso.
            if !(a.m_kg - dm > 0.0) {
                return Err(bad);
            }
            // Advección de especie fresca (R01-F08): el gas que sale lleva la
            // fracción fresca del dominio donante y se mezcla físicamente con
            // el receptor. Los líquidos usan la convención fresca = 1.
            let fresh_out = a.fresh_frac;
            let mb = b.m_kg;
            // U' = U − h·dm con la T del nodo (outflow homogéneo).
            let cva = cv_of(a.fluid);
            a.m_kg -= dm;
            a.t_k = (a.t_k * (a.m_kg + dm) * cva - h_flow) / (a.m_kg * cva);
            let ub = b.m_kg * cv_of(b.fluid) * b.t_k + h_flow;
            b.m_kg += dm;
            b.fresh_frac = (b.fresh_frac * mb + fresh_out * dm) / b.m_kg;
            b.t_k = ub / (b.m_kg * cv_of(b.fluid));
            if !(a.t_k.is_finite() && a.t_k > 0.0 && b.t_k.is_finite() && b.t_k > 0.0) {
                return Err(bad);
            }
            port_totals[pi] += dm;
        }
        // Regulador ideal: devuelve la fuente a la presión de prueba con
        // aire de suministro (frontera contabilizada en masa y entalpía).
        {
            let s = &mut d[source];
            let m_target = p_test_pa * s.v_m3 / (r_src * s.t_k);
            let dm_sup = m_target - s.m_kg;
            if dm_sup > 0.0 {
                let h_sup = dm_sup * cp_src * net.supply_t_k;
                let m_old = s.m_kg;
                s.m_kg += dm_sup;
                s.t_k = (s.t_k * m_old * cv_of(s.fluid) + h_sup) / (s.m_kg * cv_of(s.fluid));
                // El aire de suministro entra fresco (fracción 1).
                s.fresh_frac = (s.fresh_frac * m_old + dm_sup) / s.m_kg;
                supply_mass_kg += dm_sup;
                supply_enth_j += h_sup;
                supply_fresh_kg += dm_sup;
            } else if dm_sup < 0.0 {
                // Sobrepresión: ventea al banco. En términos de U sale
                // dm·cv·T (la T no cambia al extraer masa homogénea); la
                // fracción fresca tampoco cambia con el venteo (outflow
                // homogéneo).
                let cvs = cv_of(s.fluid);
                let fresh_vent = -dm_sup * s.fresh_frac;
                s.m_kg += dm_sup;
                vent_mass_kg += -dm_sup;
                vent_enth_j += -dm_sup * cvs * s.t_k;
                vent_fresh_kg += fresh_vent;
            }
            if !(s.m_kg.is_finite() && s.m_kg > 0.0 && s.t_k.is_finite() && s.t_k > 0.0) {
                return Err(bad);
            }
        }
        supply_series.push(supply_mass_kg);
        t += h;
    }
    let m1: f64 = d.iter().map(|x| x.m_kg).sum();
    let u1: f64 = d.iter().map(|x| x.m_kg * cv_of(x.fluid) * x.t_k).sum();
    let fresh1: f64 = d.iter().map(|x| x.m_kg * x.fresh_frac).sum();
    // Fuga estacionaria: media del suministro en el último 20%.
    let tail = &supply_series[supply_series.len() * 4 / 5..];
    let steady_leak_kg_s = if tail.len() >= 2 {
        (tail[tail.len() - 1] - tail[0]) / (duration_s / 5.0)
    } else {
        0.0
    };
    let total_out: f64 = port_totals.iter().sum();
    Ok((
        d,
        LeakAudit {
            steady_leak_g_s: steady_leak_kg_s * 1e3,
            split_first: if total_out > 0.0 {
                port_totals[0] / total_out
            } else {
                0.0
            },
            // Frontera contabilizada (suministro menos venteo inicial).
            mass_closure: ((m1 - m0) - (supply_mass_kg - vent_mass_kg)) / m0.max(1e-30),
            energy_closure: ((u1 - u0) - (supply_enth_j - vent_enth_j)) / u0.abs().max(1e-30),
            fresh_closure: ((fresh1 - fresh0) - (supply_fresh_kg - vent_fresh_kg)) / m0.max(1e-30),
        },
        port_totals,
    ))
}

/// cv por fluido (gas L1: cp − R con R del dominio).
fn cv_of(f: Fluid) -> f64 {
    match f {
        Fluid::Gas { r_gas, cp } => cp - r_gas,
        Fluid::Oil => 1900.0,
        Fluid::Coolant => 3500.0,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn gas_domain(m: f64) -> LeakDomain {
        LeakDomain {
            v_m3: 400e-6,
            m_kg: m,
            t_k: 350.0,
            fluid: Fluid::Gas {
                r_gas: 287.0,
                cp: 1005.0,
            },
            fresh_frac: 1.0,
        }
    }

    #[test]
    fn regulated_supply_conserves_and_splits_by_area() {
        // Fuente regulada → A/B → atmósfera (sumidero): el estacionario
        // existe y el reparto de la fuente sigue áreas 1:3 (estrangulado).
        let atm = LeakDomain {
            v_m3: 10.0,
            m_kg: 101_325.0 * 10.0 / (287.0 * 293.0),
            t_k: 293.0,
            fluid: Fluid::Gas {
                r_gas: 287.0,
                cp: 1005.0,
            },
            fresh_frac: 1.0,
        };
        let net = LeakNetwork {
            domains: vec![
                gas_domain(0.5e-3),
                gas_domain(0.4e-3),
                gas_domain(0.4e-3),
                atm,
            ],
            p_set_pa: vec![0.0, 0.0, 0.0, 0.0],
            ports: vec![
                LeakPort {
                    from: 0,
                    to: 1,
                    conductance: 1e-6,
                    compressible: true,
                },
                LeakPort {
                    from: 0,
                    to: 2,
                    conductance: 3e-6,
                    compressible: true,
                },
                LeakPort {
                    from: 1,
                    to: 3,
                    conductance: 10e-6,
                    compressible: true,
                },
                LeakPort {
                    from: 2,
                    to: 3,
                    conductance: 10e-6,
                    compressible: true,
                },
            ],
            mu_liquid_pa_s: 0.01,
            supply_t_k: 293.0,
        };
        let (_, audit, totals) = run_leakdown(&net, 0, 5.0, 0.02).unwrap();
        assert!(audit.mass_closure.abs() < 1e-9);
        assert!(audit.energy_closure.abs() < 1e-6);
        assert!(audit.steady_leak_g_s > 0.0);
        assert!((totals[1] / totals[0] - 3.0).abs() < 0.3, "{totals:?}");
        // Topología mezclada rechazada (gas→aceite como compresible).
        let mut bad = net.clone();
        bad.domains[1].fluid = Fluid::Oil;
        assert!(run_leakdown(&bad, 0, 5.0, 0.02).is_err());
    }

    #[test]
    fn fresh_species_advects_and_closes() {
        // R01-F08: donante fresco = 1, receptor = 0 ⇒ la fracción fresca
        // receptora debe ser la masa fresca recibida / nueva masa; el cierre
        // de especie fresca es exacto (frontera = suministro regulado).
        let gas = Fluid::Gas {
            r_gas: 287.0,
            cp: 1005.0,
        };
        let net = LeakNetwork {
            domains: vec![
                LeakDomain {
                    v_m3: 0.1,
                    m_kg: 200000.0 * 0.1 / (287.0 * 300.0),
                    t_k: 300.0,
                    fluid: gas,
                    fresh_frac: 1.0,
                },
                LeakDomain {
                    v_m3: 0.1,
                    m_kg: 100000.0 * 0.1 / (287.0 * 300.0),
                    t_k: 300.0,
                    fluid: gas,
                    fresh_frac: 0.0,
                },
            ],
            p_set_pa: vec![0.0, 0.0],
            ports: vec![LeakPort {
                from: 0,
                to: 1,
                conductance: 1e-5,
                compressible: true,
            }],
            mu_liquid_pa_s: 0.1,
            supply_t_k: 300.0,
        };
        let (d, audit, tot) = run_leakdown(&net, 0, 0.1, 1e-4).unwrap();
        assert!(tot[0] > 0.0);
        assert!((d[1].fresh_frac - tot[0] / d[1].m_kg).abs() < 1e-12);
        assert!(d[1].fresh_frac > 0.0 && d[1].fresh_frac < 1.0);
        assert!(audit.fresh_closure.abs() < 1e-9, "{}", audit.fresh_closure);
    }

    #[test]
    fn liquid_volume_converts_to_mass_by_density() {
        // R01-F09: k·Δp/μ·dt es volumen; multiplicar por ρ (870 kg/m³).
        let gas = Fluid::Gas {
            r_gas: 287.0,
            cp: 1005.0,
        };
        let net = LeakNetwork {
            domains: vec![
                LeakDomain {
                    v_m3: 0.1,
                    m_kg: 1.0,
                    t_k: 300.0,
                    fluid: gas,
                    fresh_frac: 1.0,
                },
                LeakDomain {
                    v_m3: 1e-3,
                    m_kg: 0.87,
                    t_k: 300.0,
                    fluid: Fluid::Oil,
                    fresh_frac: 1.0,
                },
                LeakDomain {
                    v_m3: 1e-3,
                    m_kg: 0.87,
                    t_k: 300.0,
                    fluid: Fluid::Oil,
                    fresh_frac: 1.0,
                },
            ],
            p_set_pa: vec![0.0, 200000.0, 100000.0],
            ports: vec![LeakPort {
                from: 1,
                to: 2,
                conductance: 1e-12,
                compressible: false,
            }],
            mu_liquid_pa_s: 0.1,
            supply_t_k: 300.0,
        };
        let (_, _, tot) = run_leakdown(&net, 0, 1.0, 1e-4).unwrap();
        let volume_m3 = 1e-12 * 100000.0 / 0.1;
        assert!(
            (tot[0] / (870.0 * volume_m3) - 1.0).abs() < 1e-9,
            "reported {} kg vs {} kg",
            tot[0],
            870.0 * volume_m3
        );
    }

    #[test]
    fn liquid_and_gas_use_distinct_unit_paths() {
        // Dos densidades con el mismo volumen ⇒ masas en proporción exacta;
        // el gas no aplica densidad dos veces (ya devuelve kg).
        assert_eq!(Fluid::Oil.liquid_density_kg_m3(), Some(870.0));
        assert_eq!(Fluid::Coolant.liquid_density_kg_m3(), Some(1060.0));
        assert_eq!(
            Fluid::Gas {
                r_gas: 287.0,
                cp: 1005.0
            }
            .liquid_density_kg_m3(),
            None
        );
    }
}
