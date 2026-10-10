//! Álgebra del núcleo (S03): interpolación 1D con límites y dominio.
//!
//! Dimensionalidad: estas funciones operan sobre `f64` ya validados por los
//! tipos de `units`; cada tabla lleva una `unit_label` que viaja a los
//! errores para que una mezcla de unidades sea rastreable. La seguridad
//! dimensional entre magnitudes distintas la impone el compilador en
//! `units` (tipos incompatibles).

use thiserror::Error;

/// Error de interpolación / dominio.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum MathError {
    /// Abscisas u ordenadas no finitas, vacías o de longitud distinta.
    #[error("invalid table {label}: {reason}")]
    InvalidTable {
        label: &'static str,
        reason: &'static str,
    },

    /// Punto de evaluación no finito.
    #[error("evaluation point is not finite in table {label}")]
    PointNotFinite { label: &'static str },

    /// Punto fuera del dominio con política de rechazo.
    #[error("out of domain [{lo}, {hi}] in table {label}: x={x}")]
    OutOfDomain {
        label: &'static str,
        lo: f64,
        hi: f64,
        x: f64,
    },
}

/// Política fuera de dominio.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutOfRange {
    /// Saturar al extremo más cercano.
    Clamp,
    /// Retornar [`MathError::OutOfDomain`].
    Reject,
}

/// Tabla 1D lineal por tramos sobre nodos estrictamente crecientes.
#[derive(Debug, Clone)]
pub struct Table1 {
    label: &'static str,
    xs: Vec<f64>,
    ys: Vec<f64>,
}

impl Table1 {
    /// Construye y valida la tabla.
    ///
    /// # Errors
    ///
    /// `InvalidTable` si hay menos de 2 nodos, longitudes distintas,
    /// valores no finitos o abscisas no estrictamente crecientes.
    pub fn new(label: &'static str, xs: &[f64], ys: &[f64]) -> Result<Self, MathError> {
        if xs.len() < 2 || xs.len() != ys.len() {
            return Err(MathError::InvalidTable {
                label,
                reason: "need >= 2 nodes with matching lengths",
            });
        }
        if !xs.iter().all(|v| v.is_finite()) || !ys.iter().all(|v| v.is_finite()) {
            return Err(MathError::InvalidTable {
                label,
                reason: "nodes must be finite",
            });
        }
        for w in xs.windows(2) {
            if w[1] <= w[0] {
                return Err(MathError::InvalidTable {
                    label,
                    reason: "abscissae must be strictly increasing",
                });
            }
        }
        Ok(Self {
            label,
            xs: xs.to_vec(),
            ys: ys.to_vec(),
        })
    }

    /// Evalúa por interpolación lineal.
    ///
    /// # Errors
    ///
    /// `PointNotFinite` si `x` no es finito; `OutOfDomain` si `x` cae fuera
    /// del dominio con política [`OutOfRange::Reject`].
    pub fn eval(&self, x: f64, policy: OutOfRange) -> Result<f64, MathError> {
        if !x.is_finite() {
            return Err(MathError::PointNotFinite { label: self.label });
        }
        let lo = self.xs[0];
        let hi = self.xs[self.xs.len() - 1];
        if x <= lo {
            if x < lo && policy == OutOfRange::Reject {
                return Err(MathError::OutOfDomain {
                    label: self.label,
                    lo,
                    hi,
                    x,
                });
            }
            return Ok(self.ys[0]);
        }
        if x >= hi {
            if x > hi && policy == OutOfRange::Reject {
                return Err(MathError::OutOfDomain {
                    label: self.label,
                    lo,
                    hi,
                    x,
                });
            }
            return Ok(self.ys[self.ys.len() - 1]);
        }
        // `partition_point` = búsqueda binaria determinista.
        let upper = self.xs.partition_point(|v| *v <= x);
        let i = upper - 1;
        let (x0, x1, y0, y1) = (self.xs[i], self.xs[i + 1], self.ys[i], self.ys[i + 1]);
        let t = (x - x0) / (x1 - x0);
        Ok(y0 + t * (y1 - y0))
    }

    #[must_use]
    pub fn domain(&self) -> (f64, f64) {
        (self.xs[0], self.xs[self.xs.len() - 1])
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn lcg(state: &mut u64) -> f64 {
        *state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((*state >> 11) as f64) / ((1u64 << 53) as f64)
    }

    #[test]
    fn nodes_are_exact_and_midpoints_linear() {
        let t = Table1::new("lift", &[0.0, 1.0, 3.0], &[0.0, 10.0, 20.0]).unwrap();
        assert_eq!(t.eval(0.0, OutOfRange::Reject).unwrap(), 0.0);
        assert_eq!(t.eval(3.0, OutOfRange::Reject).unwrap(), 20.0);
        assert!((t.eval(2.0, OutOfRange::Reject).unwrap() - 15.0).abs() < 1e-12);
    }

    #[test]
    fn clamp_vs_reject_at_edges() {
        let t = Table1::new("p", &[1.0, 2.0], &[10.0, 20.0]).unwrap();
        assert_eq!(t.eval(-5.0, OutOfRange::Clamp).unwrap(), 10.0);
        assert_eq!(t.eval(99.0, OutOfRange::Clamp).unwrap(), 20.0);
        assert!(matches!(
            t.eval(-5.0, OutOfRange::Reject),
            Err(MathError::OutOfDomain { .. })
        ));
        assert!(matches!(
            t.eval(f64::NAN, OutOfRange::Clamp),
            Err(MathError::PointNotFinite { .. })
        ));
    }

    #[test]
    fn invalid_tables_rejected() {
        assert!(Table1::new("e", &[], &[]).is_err());
        assert!(Table1::new("e", &[1.0], &[1.0, 2.0]).is_err());
        assert!(Table1::new("e", &[2.0, 1.0], &[1.0, 2.0]).is_err());
        assert!(Table1::new("e", &[1.0, 1.0], &[1.0, 2.0]).is_err());
        assert!(Table1::new("e", &[1.0, f64::NAN], &[1.0, 2.0]).is_err());
    }

    #[test]
    fn dense_sweep_matches_brute_force_segment() {
        // Propiedad: la búsqueda binaria coincide con el segmento por fuerza
        // bruta en 5000 puntos deterministas.
        let xs: Vec<f64> = (0..17).map(|i| f64::from(i) * 0.25).collect();
        let ys: Vec<f64> = xs.iter().map(|x| 3.0 * x + 1.0).collect();
        let t = Table1::new("line", &xs, &ys).unwrap();
        let mut s = 99;
        for _ in 0..5000 {
            let x = lcg(&mut s) * 4.0;
            let got = t.eval(x, OutOfRange::Clamp).unwrap();
            assert!((got - (3.0 * x + 1.0)).abs() < 1e-9, "x={x} got={got}");
        }
    }
}
