//! Escenarios S23 sobre el MISMO solver con suministro regulado.
//!
//! - La severidad missing > underclamped y anillo×2 > anillo×1 emergen de
//!   la ley por construcción (contacto nulo vs parcial; área ×2), sin
//!   importar la calibración pendiente. NO se afirma orden físico entre
//!   mecanismos distintos (gasket vs anillo): se reporta lado a lado.
//! - `p_seal_ref` = presión de contacto a clamp COMPLETO calculada con el
//!   campo S08 (ancla autoconsistente, no ajuste).
//! - Anillo: área geométrica × `CD_RING` (laberinto/asiento reales reducen
//!   10–100×; 0.03 ilustrativo NO calibrado, pendiente S25).

use crate::conductance::{annulus_leak_area, gasket_leak_area, SealContactState};
use crate::network::{Fluid, LeakDomain, LeakNetwork, LeakPort};
use fastener::{distribute_clamp, SealSegment};
use thiserror::Error;

/// Descarga efectiva del laberinto de anillo (ilustrativa, NO calibrada).
pub const CD_RING: f64 = 0.03;

/// Error de escenario.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ScenarioError {
    /// Configuración inválida.
    #[error("invalid leak scenario: {reason}")]
    Invalid { reason: String },
}

/// Escenario S23 (+ referencia sana para comparar, documentada).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LeakScenario {
    /// Referencia sana: clamp completo + gap stock 0.3 mm.
    HealthyBaseline,
    /// Culata asentada sin junta: contacto nulo en 4 anillos + aceite.
    MissingHeadGasket,
    /// Precarga al 50% vía cadena S08 (conductancia desde el campo).
    UnderclampedHead,
    /// Gap de anillo ×2 (0.6 mm) en un cilindro, a cárter.
    RingLeakCyl3,
}

/// Gap de anillo (m): stock 0.3 mm ilustrativo (S01 PENDING).
pub const RING_GAP_STOCK_M: f64 = 0.3e-3;
/// Profundidad radial del laberinto (m, ilustrativa).
pub const RING_DEPTH_M: f64 = 3e-3;
/// Carga total de clamp completa/incompleta (N, demo arbitraria S08).
pub const CLAMP_FULL_N: f64 = 200_000.0;
pub const CLAMP_HALF_N: f64 = 100_000.0;

/// Red del escenario. Dominios: [cámara, cárter, refrigerante, aceite,
/// atmósfera]. Cámara a 500 kPa/350 K (banco ilustrativo, no combustión).
///
/// # Errors
///
/// `Invalid` si algún parámetro viola dominio.
pub fn leak_scenario(s: LeakScenario) -> Result<LeakNetwork, ScenarioError> {
    let bad = |reason: &str| ScenarioError::Invalid {
        reason: reason.to_owned(),
    };
    let gas = Fluid::Gas {
        r_gas: 287.0,
        cp: 1005.0,
    };
    let chamber = LeakDomain {
        v_m3: 400e-6,
        m_kg: 500_000.0 * 400e-6 / (287.0 * 350.0),
        t_k: 350.0,
        fluid: gas,
        fresh_frac: 1.0,
    };
    let crankcase = LeakDomain {
        v_m3: 4e-3,
        m_kg: 101_325.0 * 4e-3 / (287.0 * 300.0),
        t_k: 300.0,
        fluid: gas,
        fresh_frac: 0.5,
    };
    let coolant = LeakDomain {
        v_m3: 1e-3,
        m_kg: 1.0,
        t_k: 353.0,
        fluid: Fluid::Coolant,
        fresh_frac: 1.0,
    };
    let oil = LeakDomain {
        v_m3: 1e-3,
        m_kg: 0.87,
        t_k: 353.0,
        fluid: Fluid::Oil,
        fresh_frac: 1.0,
    };
    let atmosphere = LeakDomain {
        v_m3: 1.0,
        m_kg: 101_325.0 * 1.0 / (287.0 * 293.0),
        t_k: 293.0,
        fluid: gas,
        fresh_frac: 1.0,
    };
    let p_set = vec![0.0, 0.0, 110_000.0, 105_000.0, 0.0];

    // Ancla autoconsistente: contacto a clamp completo desde S08.
    let segs = vec![
        SealSegment {
            area_m2: 400e-6,
            stiffness_weight: 1.0,
        };
        4
    ];
    let field_full = distribute_clamp(CLAMP_FULL_N, &segs).map_err(|_| bad("gasket field"))?;
    let p_seal_ref = field_full
        .contact_pressure_pa
        .iter()
        .fold(0.0f64, |a, b| a.max(*b));
    if !(p_seal_ref.is_finite() && p_seal_ref > 0.0) {
        return Err(bad("seal anchor"));
    }

    let mut ports = Vec::new();
    // Respiradero del cárter a la atmósfera (PCV/breather, área efectiva
    // ilustrativa 30 mm²): sin él ningún escenario alcanza estacionario
    // (el cárter se presurizaría hasta igualar la cámara).
    ports.push(LeakPort {
        from: 1,
        to: 4,
        conductance: 30e-6,
        compressible: true,
    });
    match s {
        LeakScenario::HealthyBaseline => {
            // Clamp completo ⇒ sellado (ley) + anillo stock con Cd.
            let a_ring = annulus_leak_area(RING_GAP_STOCK_M, RING_DEPTH_M)
                .map_err(|_| bad("ring gap"))?
                * CD_RING;
            ports.push(LeakPort {
                from: 0,
                to: 1,
                conductance: a_ring,
                compressible: true,
            });
        }
        LeakScenario::MissingHeadGasket => {
            let seg = SealContactState {
                contact_pressure_pa: 0.0,
                area_m2: 400e-6,
            };
            let a_fire = gasket_leak_area(seg, p_seal_ref);
            for _ in 0..4 {
                ports.push(LeakPort {
                    from: 0,
                    to: 1,
                    conductance: a_fire,
                    compressible: true,
                });
            }
            ports.push(LeakPort {
                from: 0,
                to: 2,
                conductance: a_fire / 2.0,
                compressible: true,
            });
            // Sin puerto aceite→exterior: el rezume externo exige modelo de
            // retén con unidades propias (S26), no una k arbitraria.
        }
        LeakScenario::UnderclampedHead => {
            // Cadena causal: torque mitad ⇒ precarga mitad (S08) ⇒ campo
            // ⇒ conductancia por ley (nunca daño booleano).
            let field_half =
                distribute_clamp(CLAMP_HALF_N, &segs).map_err(|_| bad("gasket field"))?;
            for p in &field_half.contact_pressure_pa {
                let a = gasket_leak_area(
                    SealContactState {
                        contact_pressure_pa: *p,
                        area_m2: 400e-6,
                    },
                    p_seal_ref,
                );
                ports.push(LeakPort {
                    from: 0,
                    to: 1,
                    conductance: a,
                    compressible: true,
                });
            }
            // + anillo stock (el motor respira también por ahí).
            let a_ring = annulus_leak_area(RING_GAP_STOCK_M, RING_DEPTH_M)
                .map_err(|_| bad("ring gap"))?
                * CD_RING;
            ports.push(LeakPort {
                from: 0,
                to: 1,
                conductance: a_ring,
                compressible: true,
            });
        }
        LeakScenario::RingLeakCyl3 => {
            let a = annulus_leak_area(2.0 * RING_GAP_STOCK_M, RING_DEPTH_M)
                .map_err(|_| bad("ring gap"))?
                * CD_RING;
            ports.push(LeakPort {
                from: 0,
                to: 1,
                conductance: a,
                compressible: true,
            });
        }
    }
    Ok(LeakNetwork {
        domains: vec![chamber, crankcase, coolant, oil, atmosphere],
        p_set_pa: p_set,
        ports,
        mu_liquid_pa_s: 0.01,
        supply_t_k: 293.0,
    })
}

/// Sensibilidad declarada del umbral (contacto fijo de media clamp):
/// reporta el rango ×0.5/×2 sin ajustar nada.
#[must_use]
pub fn pseal_sensitivity() -> [(f64, f64); 3] {
    let seg = SealContactState {
        contact_pressure_pa: 62.5e6,
        area_m2: 400e-6,
    };
    [
        (75e6, gasket_leak_area(seg, 75e6)),
        (150e6, gasket_leak_area(seg, 150e6)),
        (300e6, gasket_leak_area(seg, 300e6)),
    ]
}

/// Sensibilidad declarada de la descarga de anillo (×0.3/×3).
#[must_use]
pub fn cdring_sensitivity_gap06() -> [(f64, f64); 3] {
    let base = annulus_leak_area(2.0 * RING_GAP_STOCK_M, RING_DEPTH_M).unwrap_or(0.0);
    [
        (0.01, base * 0.01),
        (CD_RING, base * CD_RING),
        (0.09, base * 0.09),
    ]
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::network::run_leakdown;

    fn steady(s: LeakScenario) -> f64 {
        // Paso fino (fugas enormes exigen dt pequeño: fail-closed en caso
        // contrario, nunca salto silencioso).
        let net = leak_scenario(s).unwrap();
        let (_, audit, _) = run_leakdown(&net, 0, 5.0, 2e-4).unwrap();
        assert!(audit.mass_closure.abs() < 1e-9);
        assert!(audit.energy_closure.abs() < 1e-6);
        audit.steady_leak_g_s
    }

    #[test]
    fn robust_orderings_hold_regardless_of_calibration() {
        // Junta ausente (contacto 0 ⇒ área completa) > subapretada
        // (contacto parcial ⇒ ley monótona): vale para todo p_ref > 0.
        let missing = steady(LeakScenario::MissingHeadGasket);
        let under = steady(LeakScenario::UnderclampedHead);
        assert!(
            missing > under && under > 0.0,
            "missing={missing} under={under}"
        );
        // Anillo ×2 > anillo stock con el mismo Cd: vale para todo Cd > 0.
        let ring2 = steady(LeakScenario::RingLeakCyl3);
        let healthy = steady(LeakScenario::HealthyBaseline);
        assert!(
            ring2 > healthy && healthy > 0.0,
            "ring2={ring2} healthy={healthy}"
        );
    }

    #[test]
    fn sensitivities_reported_not_tuned() {
        // Umbral: a mayor p_ref exigido, más fuga con el mismo clamp
        // (dirección de la forma; la magnitud es lo pendiente).
        let s = pseal_sensitivity();
        assert!(s[0].1 < s[1].1 && s[1].1 < s[2].1);
        let c = cdring_sensitivity_gap06();
        assert!(c[0].1 < c[1].1 && c[1].1 < c[2].1);
    }
}
