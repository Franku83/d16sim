//! Política `f64`: redondeo, serialización y valores no finitos.

use thiserror::Error;

/// Error dimensional / de dominio (nunca un pánico en rutas de producción).
#[derive(Debug, Clone, PartialEq, Error)]
pub enum UnitsError {
    /// NaN o infinito donde se exigía un valor físico.
    #[error("non-finite value for {what}")]
    NonFinite { what: &'static str },

    /// Valor fuera del rango físico (p. ej. Kelvin negativo).
    #[error("out of range for {what}: {value}")]
    OutOfRange { what: &'static str, value: f64 },
}

/// Exige un `f64` finito.
///
/// # Errors
///
/// Retorna [`UnitsError::NonFinite`] si `v` es NaN o infinito.
pub fn require_finite(v: f64, what: &'static str) -> Result<f64, UnitsError> {
    if v.is_finite() {
        Ok(v)
    } else {
        Err(UnitsError::NonFinite { what })
    }
}

/// Exige `v >= 0` finito (masas, presiones absolutas, potencias S03).
///
/// # Errors
///
/// `NonFinite` si no es finito; `OutOfRange` si es negativo.
pub fn require_non_negative(v: f64, what: &'static str) -> Result<f64, UnitsError> {
    let v = require_finite(v, what)?;
    if v >= 0.0 {
        Ok(v)
    } else {
        Err(UnitsError::OutOfRange { what, value: v })
    }
}

/// Subnormales se llevan a `+0.0` (política S03): evita la no-portabilidad
/// de flushing denormal entre targets sin ramas dependientes de plataforma.
/// Determinista en toda plataforma para la misma entrada.
#[must_use]
pub fn flush_subnormal(v: f64) -> f64 {
    if v.is_subnormal() {
        0.0
    } else {
        v
    }
}

/// Serialización: `serde_json` (Ryū, round-trip exacto de `f64` con 17
/// dígitos significativos). Sin truncado a `f32` en el core; la UI puede
/// recibir `f32`/resúmenes aguas abajo.
#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn rejects_nan_and_infinities() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(matches!(
                require_finite(bad, "probe"),
                Err(UnitsError::NonFinite { .. })
            ));
            assert!(require_non_negative(bad, "probe").is_err());
        }
    }

    #[test]
    fn subnormals_flush_to_zero_deterministically() {
        assert_eq!(flush_subnormal(f64::MIN_POSITIVE / 2.0), 0.0);
        assert_eq!(flush_subnormal(0.0), 0.0);
        assert_eq!(flush_subnormal(1.0), 1.0);
    }

    #[test]
    fn negatives_rejected_where_unphysical() {
        assert!(require_non_negative(-1.0, "mass").is_err());
        assert_eq!(require_non_negative(0.0, "mass").unwrap(), 0.0);
    }
}
