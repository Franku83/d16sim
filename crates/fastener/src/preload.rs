//! Precarga L1 por torque-factor: `F = T / (K·d)`.
//!
//! `K` agrega fricción de rosca/asiento, acabado y lubricación: cambia con
//! la condición, por eso el par aplicado no es fuerza de apriete. Los
//! rangos ilustrativos son orden de magnitud de literatura genérica de
//! fijaciones — NO Honda OEM, NO valores ARP trasladados — y solo sirven
//! para estudios de sensibilidad. El gate se prueba con `K` explícito.

use thiserror::Error;
use units::float::require_finite;

/// Error del modelo de precarga.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum PreloadError {
    /// Entrada inválida (torque/diámetro/K no físicos).
    #[error("invalid preload input: {reason}")]
    InvalidInput { reason: String },

    /// Método no soportado en L1 (p. ej. apriete solo por ángulo sin curva
    /// de clamp-load: requiere modelo de interacción elástica, no L1).
    #[error("method not supported in L1: {reason}")]
    MethodNotSupported { reason: String },
}

/// Condición de lubricación/asiento (presets ilustrativos, ver
/// [`illustrative_k_range`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LubricantCondition {
    DryAsReceived,
    OiledGeneric,
    AntiSeizeGeneric,
    /// K explícito del llamante (rango): el camino honesto para datos reales.
    ExplicitK {
        k_lo_x1000: u32,
        k_hi_x1000: u32,
    },
}

/// Rango ilustrativo de K por condición (adimensional).
///
/// Valores de orden de magnitud de literatura genérica; **prohibido**
/// usarlos como especificación Honda o como valores ARP. Calibración
/// pendiente (S25). Devuelve `(K_lo, K_hi)`.
///
/// # Errors
///
/// `InvalidInput` si la variante explícita trae un rango vacío o no finito.
pub fn illustrative_k_range(cond: LubricantCondition) -> Result<(f64, f64), PreloadError> {
    match cond {
        LubricantCondition::DryAsReceived => Ok((0.18, 0.22)),
        LubricantCondition::OiledGeneric => Ok((0.13, 0.17)),
        LubricantCondition::AntiSeizeGeneric => Ok((0.10, 0.14)),
        LubricantCondition::ExplicitK {
            k_lo_x1000,
            k_hi_x1000,
        } => {
            let (lo, hi) = (
                f64::from(k_lo_x1000) / 1000.0,
                f64::from(k_hi_x1000) / 1000.0,
            );
            if !(lo.is_finite() && hi.is_finite() && lo > 0.0 && hi >= lo) {
                return Err(PreloadError::InvalidInput {
                    reason: "explicit K range must be finite with 0 < lo <= hi".to_owned(),
                });
            }
            Ok((lo, hi))
        }
    }
}

/// Paso de apriete registrado por perno (historial ordenado).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TorqueStep {
    /// Par aplicado (N·m). `None` = paso solo por ángulo.
    pub torque_nm: Option<f64>,
    /// Ángulo girado (rad, informativo en L1).
    pub angle_rad: f64,
    pub lubricant: LubricantCondition,
}

/// Estimación de precarga con incertidumbre por K.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PreloadEstimate {
    /// Precarga con K_hi (mínima fuerza).
    pub force_lo_n: f64,
    /// Precarga con K medio.
    pub force_nom_n: f64,
    /// Precarga con K_lo (máxima fuerza).
    pub force_hi_n: f64,
    /// Chequeo de fluencia: `None` = sin datos de material (S27).
    pub yield_utilization: Option<YieldCheck>,
}

/// Uso respecto a fluencia con área y límite dados por el llamante.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct YieldCheck {
    pub utilization_lo: f64,
    pub utilization_hi: f64,
    /// true si el rango supera 1.0 (estado, no rotura decretada).
    pub beyond_yield_per_inputs: bool,
}

/// Estima precarga del último paso con torque.
///
/// # Errors
///
/// `InvalidInput` si torque ≤ 0, diámetro ≤ 0 o no finitos;
/// `MethodNotSupported` si el paso es solo por ángulo (sin curva de
/// clamp-load no hay L1 honesto).
pub fn preload_of_step(
    step: TorqueStep,
    nominal_dia_m: f64,
    tensile_area_m2: Option<f64>,
    yield_strength_pa: Option<f64>,
) -> Result<PreloadEstimate, PreloadError> {
    let bad = |reason: &str| PreloadError::InvalidInput {
        reason: reason.to_owned(),
    };
    let torque = step
        .torque_nm
        .ok_or_else(|| PreloadError::MethodNotSupported {
            reason: "angle-only step needs a clamp-load curve (not L1)".to_owned(),
        })?;
    require_finite(torque, "torque").map_err(|_| bad("non-finite torque"))?;
    require_finite(nominal_dia_m, "diameter").map_err(|_| bad("non-finite diameter"))?;
    if torque <= 0.0 {
        return Err(bad("torque must be > 0"));
    }
    if nominal_dia_m <= 0.0 {
        return Err(bad("diameter must be > 0"));
    }
    let (k_lo, k_hi) = illustrative_k_range(step.lubricant)?;
    // F = T/(K·d): K alto ⇒ F baja.
    let force_lo_n = torque / (k_hi * nominal_dia_m);
    let force_hi_n = torque / (k_lo * nominal_dia_m);
    let force_nom_n = torque / (((k_lo + k_hi) / 2.0) * nominal_dia_m);

    let yield_utilization = match (tensile_area_m2, yield_strength_pa) {
        (Some(a), Some(y)) => {
            if !(a.is_finite() && y.is_finite() && a > 0.0 && y > 0.0) {
                return Err(bad("area and yield must be finite and > 0"));
            }
            let u_lo = (force_lo_n / a) / y;
            let u_hi = (force_hi_n / a) / y;
            Some(YieldCheck {
                utilization_lo: u_lo,
                utilization_hi: u_hi,
                beyond_yield_per_inputs: u_hi > 1.0,
            })
        }
        _ => None,
    };
    Ok(PreloadEstimate {
        force_lo_n,
        force_nom_n,
        force_hi_n,
        yield_utilization,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn step(torque: f64, lubricant: LubricantCondition) -> TorqueStep {
        TorqueStep {
            torque_nm: Some(torque),
            angle_rad: 0.0,
            lubricant,
        }
    }

    #[test]
    fn same_torque_different_lubricant_gives_different_preload() {
        // Gate S08 (i): el lubricante cambia la precarga según el modelo.
        let dry = preload_of_step(
            step(50.0, LubricantCondition::DryAsReceived),
            0.010,
            None,
            None,
        )
        .unwrap();
        let oiled = preload_of_step(
            step(50.0, LubricantCondition::OiledGeneric),
            0.010,
            None,
            None,
        )
        .unwrap();
        assert!(oiled.force_nom_n > dry.force_nom_n);
        // 50 N·m, d = 10 mm, K medio seco 0.20 ⇒ F = 25 kN.
        assert!((dry.force_nom_n - 25_000.0).abs() < 1e-9);
        assert!((oiled.force_nom_n - 50.0 / (0.15 * 0.010)).abs() < 1e-9);
    }

    #[test]
    fn torque_sweep_is_monotone_and_k_uncertainty_propagates() {
        let a = preload_of_step(
            step(40.0, LubricantCondition::DryAsReceived),
            0.010,
            None,
            None,
        )
        .unwrap();
        let b = preload_of_step(
            step(60.0, LubricantCondition::DryAsReceived),
            0.010,
            None,
            None,
        )
        .unwrap();
        assert!(b.force_nom_n > a.force_nom_n);
        // F_hi/F_lo = K_hi/K_lo exactamente (propagación declarada).
        assert!(((a.force_hi_n / a.force_lo_n) - (0.22 / 0.18)).abs() < 1e-12);
    }

    #[test]
    fn angle_only_and_bad_inputs_rejected() {
        let angle_only = TorqueStep {
            torque_nm: None,
            angle_rad: 1.0,
            lubricant: LubricantCondition::OiledGeneric,
        };
        assert!(matches!(
            preload_of_step(angle_only, 0.010, None, None),
            Err(PreloadError::MethodNotSupported { .. })
        ));
        assert!(preload_of_step(
            step(-5.0, LubricantCondition::DryAsReceived),
            0.010,
            None,
            None
        )
        .is_err());
        assert!(preload_of_step(
            step(50.0, LubricantCondition::DryAsReceived),
            0.0,
            None,
            None
        )
        .is_err());
    }

    #[test]
    fn yield_check_needs_material_and_never_decrees_breakage() {
        let no_data = preload_of_step(
            step(50.0, LubricantCondition::DryAsReceived),
            0.010,
            None,
            None,
        )
        .unwrap();
        assert_eq!(no_data.yield_utilization, None);
        // Área ilustrativa diminuta ⇒ supera fluencia ilustrativa: estado,
        // no rotura (la rotura es S27 con material/geometría/calibración).
        let over = preload_of_step(
            step(200.0, LubricantCondition::AntiSeizeGeneric),
            0.010,
            Some(20e-6),
            Some(640e6),
        )
        .unwrap();
        let check = over.yield_utilization.unwrap();
        assert!(check.beyond_yield_per_inputs);
        assert!(check.utilization_hi > check.utilization_lo);
    }
}
