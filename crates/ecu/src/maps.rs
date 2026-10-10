//! Mapas de referencia P2E-like (S18): malla bilineal + tablas propias.
//!
//! Estos mapas son IMPLEMENTACIÓN DE REFERENCIA etiquetada, nunca mapas
//! OEM Honda (inexistentes en la comunidad: MISSING-DATA S01). Valores
//! ilustrativos de orden plausible, saturados en borde (sin extrapolación
//! silenciosa).

use thiserror::Error;

/// Error de mapas.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum MapError {
    /// Malla inválida (ejes no crecientes, tamaños incoherentes).
    #[error("invalid map grid: {reason}")]
    InvalidGrid { reason: String },
}

/// Malla bilineal con ejes estrictamente crecientes y saturación en borde.
#[derive(Debug, Clone, PartialEq)]
pub struct BilinearGrid {
    pub label: &'static str,
    pub rpm_axis: Vec<f64>,
    pub load_axis: Vec<f64>,
    pub values: Vec<Vec<f64>>,
}

impl BilinearGrid {
    /// Construye validando ejes y dimensiones.
    ///
    /// # Errors
    ///
    /// `InvalidGrid` si hay < 2 puntos por eje, no crecientes, no finitos
    /// o la tabla no es [n_rpm × n_load].
    pub fn new(
        label: &'static str,
        rpm_axis: &[f64],
        load_axis: &[f64],
        values: Vec<Vec<f64>>,
    ) -> Result<Self, MapError> {
        let bad = |reason: &str| MapError::InvalidGrid {
            reason: reason.to_owned(),
        };
        for (ax, name) in [(rpm_axis, "rpm"), (load_axis, "load")] {
            if ax.len() < 2 {
                return Err(bad(name));
            }
            if !ax.iter().all(|v| v.is_finite()) {
                return Err(bad(name));
            }
            for w in ax.windows(2) {
                if w[1] <= w[0] {
                    return Err(bad(name));
                }
            }
        }
        if values.len() != rpm_axis.len() || values.iter().any(|row| row.len() != load_axis.len()) {
            return Err(bad("values shape"));
        }
        if values.iter().flatten().any(|v| !v.is_finite()) {
            return Err(bad("values finite"));
        }
        Ok(Self {
            label,
            rpm_axis: rpm_axis.to_vec(),
            load_axis: load_axis.to_vec(),
            values,
        })
    }

    ///interpola (satura fuera de dominio, documentado).
    #[must_use]
    pub fn lookup(&self, rpm: f64, load: f64) -> f64 {
        let (i, tx) = locate(&self.rpm_axis, rpm);
        let (j, ty) = locate(&self.load_axis, load);
        let v00 = self.values[i][j];
        let v10 = self.values[i + 1][j];
        let v01 = self.values[i][j + 1];
        let v11 = self.values[i + 1][j + 1];
        v00 * (1.0 - tx) * (1.0 - ty)
            + v10 * tx * (1.0 - ty)
            + v01 * (1.0 - tx) * ty
            + v11 * tx * ty
    }
}

/// Localiza segmento + fracción (satura en bordes).
fn locate(axis: &[f64], x: f64) -> (usize, f64) {
    if x <= axis[0] {
        return (0, 0.0);
    }
    if x >= axis[axis.len() - 1] {
        return (axis.len() - 2, 1.0);
    }
    let mut i = 0;
    while i + 2 < axis.len() && x > axis[i + 1] {
        i += 1;
    }
    let t = (x - axis[i]) / (axis[i + 1] - axis[i]);
    (i, t)
}

/// Mapas de referencia (P2E-like, no OEM).
#[derive(Debug, Clone)]
pub struct ReferenceMaps {
    /// Masa de combustible por ciclo y cilindro (kg).
    pub fuel_kg: BilinearGrid,
    /// Avance de encendido (° BTDC).
    pub spark_btdc_deg: BilinearGrid,
}

impl ReferenceMaps {
    /// Tablas ilustrativas de orden plausible (ralentí→WOT, 750→6000 rpm).
    /// Masa ~7.7 mg a ralentí/baja carga hasta ~30 mg a plena carga;
    /// avance 12° base con variación suave. NO son la P2E.
    ///
    /// # Errors
    ///
    /// Nunca falla con estas constantes (el `Result` existe para tablas
    /// cargadas de disco en el futuro).
    pub fn illustrative() -> Result<Self, MapError> {
        let rpm = vec![500.0, 750.0, 1500.0, 3000.0, 4600.0, 6000.0];
        let load = vec![0.2, 0.4, 0.6, 0.8, 1.0];
        let fuel = vec![
            vec![6.0e-6, 7.0e-6, 8.0e-6, 9.0e-6, 10.0e-6],
            vec![6.5e-6, 7.7e-6, 9.0e-6, 10.5e-6, 12.0e-6],
            vec![7.0e-6, 9.0e-6, 11.0e-6, 14.0e-6, 17.0e-6],
            vec![8.0e-6, 11.0e-6, 15.0e-6, 20.0e-6, 25.0e-6],
            vec![9.0e-6, 13.0e-6, 18.0e-6, 24.0e-6, 30.0e-6],
            vec![9.5e-6, 14.0e-6, 20.0e-6, 26.0e-6, 32.0e-6],
        ];
        let spark = vec![
            vec![10.0, 11.0, 12.0, 12.0, 12.0],
            vec![12.0, 14.0, 16.0, 16.0, 15.0],
            vec![14.0, 18.0, 22.0, 20.0, 18.0],
            vec![16.0, 22.0, 26.0, 24.0, 20.0],
            vec![16.0, 24.0, 28.0, 26.0, 22.0],
            vec![16.0, 24.0, 28.0, 26.0, 22.0],
        ];
        Ok(Self {
            fuel_kg: BilinearGrid::new("fuel_kg_ref (P2E-like, not OEM)", &rpm, &load, fuel)?,
            spark_btdc_deg: BilinearGrid::new(
                "spark_btdc_ref (P2E-like, not OEM)",
                &rpm,
                &load,
                spark,
            )?,
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn nodes_exact_and_edges_clamp() {
        let m = ReferenceMaps::illustrative().unwrap();
        // Nodos exactos (ralentí 7.7 mg, base 12°).
        assert!((m.fuel_kg.lookup(750.0, 0.4) - 7.7e-6).abs() < 1e-12);
        assert!((m.spark_btdc_deg.lookup(750.0, 0.2) - 12.0).abs() < 1e-12);
        // Saturación documentada fuera de dominio (sin extrapolación).
        assert_eq!(m.fuel_kg.lookup(200.0, 0.4), m.fuel_kg.lookup(500.0, 0.4));
        assert_eq!(
            m.spark_btdc_deg.lookup(8000.0, 1.0),
            m.spark_btdc_deg.lookup(6000.0, 1.0)
        );
        assert!(BilinearGrid::new("x", &[1.0], &[1.0, 2.0], vec![vec![1.0, 2.0]]).is_err());
    }
}
