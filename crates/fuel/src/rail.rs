//! Combustible, rail, regulador y bomba (S13).
//!
//! Regulador referenciado al múltiple (ideal L1): `p_rail = p_man + ΔP_base`
//! con bomba encendida; con bomba apagada el rail se despresuriza (0 Pa,
//! simplificación declarada: sin curva de caída temporal). La presión que
//! importa al inyector es la DIFERENCIAL, no la manométrica del rail.

use thiserror::Error;

/// Error de suministro.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum RailError {
    /// Entradas inválidas (presiones, capacidad).
    #[error("invalid fuel-supply input: {reason}")]
    InvalidInput { reason: String },
}

/// Combustible identificado (propiedades ilustrativas de gasolina
/// comercial, NO lote medido).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fuel {
    /// Densidad (kg/m³).
    pub density_kg_m3: f64,
    /// Poder calorífico inferior (J/kg).
    pub lhv_j_per_kg: f64,
    /// AFR estequiométrico.
    pub stoich_afr: f64,
}

impl Fuel {
    /// Gasolina comercial ilustrativa (0.74 g/cm³ del research v0.1 §3.2,
    /// LHV 44 MJ/kg coherente con S10, stoich 14.7).
    #[must_use]
    pub fn illustrative_gasoline() -> Self {
        Self {
            density_kg_m3: 740.0,
            lhv_j_per_kg: 44e6,
            stoich_afr: 14.7,
        }
    }
}

/// Regulador referenciado al múltiple con ΔP base explícita.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Regulator {
    /// ΔP nominal inyector (Pa) con bomba capaz y vacuums compensados.
    pub base_delta_p_pa: f64,
}

impl Regulator {
    /// # Errors
    ///
    /// `InvalidInput` si ΔP no es `> 0` finita.
    pub fn new(base_delta_p_pa: f64) -> Result<Self, RailError> {
        if !(base_delta_p_pa.is_finite() && base_delta_p_pa > 0.0) {
            return Err(RailError::InvalidInput {
                reason: "base ΔP must be finite and > 0".to_owned(),
            });
        }
        Ok(Self { base_delta_p_pa })
    }
}

/// Bomba: encendida/apagada + capacidad máxima (None = ilimitada declarada).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PumpState {
    pub on: bool,
    /// Caudal máximo (kg/s). `None` = sin límite modelado.
    pub max_flow_kg_s: Option<f64>,
}

/// Estado del rail resuelto.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RailState {
    /// Presión absoluta del rail (Pa).
    pub p_rail_abs_pa: f64,
    /// ΔP real sobre el inyector = rail − múltiple (Pa).
    pub delta_p_pa: f64,
    /// ¿Puede la bomba sostener la demanda dada?
    pub pump_ok: bool,
}

/// Resuelve el rail frente a la presión de múltiple dada.
///
/// # Errors
///
/// `InvalidInput` si el múltiple no es `≥ 0` finito o la demanda es
/// negativa/no finita.
pub fn resolve_rail(
    regulator: Regulator,
    pump: PumpState,
    p_manifold_abs_pa: f64,
    demand_kg_s: f64,
) -> Result<RailState, RailError> {
    let bad = |reason: &str| RailError::InvalidInput {
        reason: reason.to_owned(),
    };
    if !(p_manifold_abs_pa.is_finite() && p_manifold_abs_pa >= 0.0) {
        return Err(bad("manifold pressure must be finite and >= 0"));
    }
    if !(demand_kg_s.is_finite() && demand_kg_s >= 0.0) {
        return Err(bad("demand must be finite and >= 0"));
    }
    if !pump.on {
        return Ok(RailState {
            p_rail_abs_pa: 0.0,
            delta_p_pa: 0.0,
            pump_ok: false,
        });
    }
    let pump_ok = pump.max_flow_kg_s.map_or(true, |cap| demand_kg_s <= cap);
    // Regulador ideal: mantiene ΔP aunque el rail absoluto siga al múltiple
    // (rail 40–47 psi sin vacío / 28–36 con vacío, OEM-TEXT candidato).
    Ok(RailState {
        p_rail_abs_pa: p_manifold_abs_pa + regulator.base_delta_p_pa,
        delta_p_pa: regulator.base_delta_p_pa,
        pump_ok,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn regulator_tracks_manifold_and_pump_off_depressurizes() {
        let reg = Regulator::new(300_000.0).unwrap();
        let pump = PumpState {
            on: true,
            max_flow_kg_s: None,
        };
        // WOT (100 kPa) vs ralentí con vacío (35 kPa): el rail absoluto
        // cambia, el ΔP se conserva (regulador referenciado ideal).
        let wot = resolve_rail(reg, pump, 100_000.0, 0.01).unwrap();
        let idle = resolve_rail(reg, pump, 35_000.0, 0.002).unwrap();
        assert!((wot.p_rail_abs_pa - 400_000.0).abs() < 1e-9);
        assert!((idle.p_rail_abs_pa - 335_000.0).abs() < 1e-9);
        assert_eq!(wot.delta_p_pa, idle.delta_p_pa);
        // Bomba apagada: sin presión (estado, no DTC).
        let off = resolve_rail(
            reg,
            PumpState {
                on: false,
                max_flow_kg_s: None,
            },
            100_000.0,
            0.01,
        )
        .unwrap();
        assert_eq!(off.delta_p_pa, 0.0);
        assert!(!off.pump_ok);
        // Demanda sobre capacidad: flag, no entrega fingida.
        let weak = resolve_rail(
            reg,
            PumpState {
                on: true,
                max_flow_kg_s: Some(0.001),
            },
            100_000.0,
            0.01,
        )
        .unwrap();
        assert!(!weak.pump_ok);
    }
}
