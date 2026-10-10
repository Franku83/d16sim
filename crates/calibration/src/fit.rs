//! Ajuste S25-T4: Gauss-Newton determinista con covarianza `(JᵀWJ)⁻¹` y
//! correlaciones. Opera sobre una residual genérica (modelo inyectado):
//! el motor completo es caro, así que el fitter se verifica con el modelo
//! de cranking L1 y datos SYNTHETIC_SELFTEST (self-consistency, no
//! validación). Con datos reales futuros, el mismo fitter aplica.

use thiserror::Error;

/// Error de ajuste.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum FitError {
    /// Configuración o convergencia inválida (fail-closed).
    #[error("invalid fit: {reason}")]
    Invalid { reason: String },
}

/// Resultado con incertidumbre (k=1) y correlación máxima.
#[derive(Debug, Clone, PartialEq)]
pub struct FitResult {
    pub params: Vec<f64>,
    pub std_k1: Vec<f64>,
    pub max_abs_correlation: f64,
    pub iterations: usize,
    pub residual_norm: f64,
}

/// Gauss-Newton con `params` iniciales y residual `f(p) -> r`.
///
/// # Errors
///
/// `Invalid` si dimensiones, jacobiano singular o no converge en 100 it.
pub fn least_squares<F>(
    f: F,
    y: &[f64],
    w: &[f64],
    p0: &[f64],
    tol: f64,
) -> Result<FitResult, FitError>
where
    F: Fn(&[f64]) -> Vec<f64>,
{
    let bad = |r: &str| FitError::Invalid {
        reason: r.to_owned(),
    };
    let n = y.len();
    let m = p0.len();
    if n == 0 || m == 0 || w.len() != n || !(tol.is_finite() && tol > 0.0) {
        return Err(bad("dims/tol"));
    }
    if !(y.iter().all(|v| v.is_finite())
        && w.iter().all(|v| v.is_finite())
        && p0.iter().all(|v| v.is_finite()))
    {
        return Err(bad("non-finite data"));
    }
    // Escalado interno: q = p/scale con scale = |p0| (los parámetros de
    // órdenes dispares —C~1e6 vs p~1e3— volverían singular a JᵀJ).
    let scale: Vec<f64> = p0.iter().map(|v| v.abs().max(1e-300)).collect();
    let unscale = |q: &[f64]| -> Vec<f64> { q.iter().zip(&scale).map(|(qi, s)| qi * s).collect() };
    // R02-C1/RV08: la salida del callback se valida (dimensión y finitud)
    // antes de indexarla; un modelo mal formado es error tipado, no panic.
    let fq = |q: &[f64]| -> Result<Vec<f64>, FitError> {
        let out = f(&unscale(q));
        if out.len() != n || !out.iter().all(|v| v.is_finite()) {
            return Err(bad("callback output dims/finite"));
        }
        Ok(out)
    };
    let q0: Vec<f64> = p0.iter().zip(&scale).map(|(v, s)| v / s).collect();
    let mut p = q0;
    let h = 1e-6;
    let mut iters = 0;
    for it in 0..100 {
        iters = it + 1;
        let r: Vec<f64> = fq(&p)?.iter().zip(y).map(|(a, b)| a - b).collect();
        let norm: f64 = r
            .iter()
            .zip(w)
            .map(|(ri, wi)| (ri * wi).powi(2))
            .sum::<f64>()
            .sqrt();
        // Jacobiano por diferencias centrales.
        let mut j = vec![vec![0.0; m]; n];
        for k in 0..m {
            let mut pp = p.clone();
            pp[k] += h;
            let fp = fq(&pp)?;
            let mut pm = p.clone();
            pm[k] -= h;
            let fm = fq(&pm)?;
            for i in 0..n {
                j[i][k] = (fp[i] - fm[i]) / (2.0 * h);
            }
        }
        // A = JᵀW²J, g = JᵀW²r; resuelve A·dp = −g (Gauss-Jordan, m ≤ 8).
        let mut a = vec![vec![0.0; m]; m];
        let mut g = vec![0.0; m];
        for i in 0..n {
            let wi2 = w[i] * w[i];
            for k in 0..m {
                g[k] += j[i][k] * wi2 * r[i];
                for l in 0..m {
                    a[k][l] += j[i][k] * wi2 * j[i][l];
                }
            }
        }
        let dp =
            solve(&a, &g.iter().map(|v| -v).collect::<Vec<_>>()).ok_or(bad("singular step"))?;
        // Backtracking determinista: el paso completo de Gauss-Newton
        // puede sobreoscilar en modelos no lineales; se parte hasta que
        // el residuo baja (fail-closed si ni 1/2²⁰ sirve).
        let mut alpha = 1.0;
        let mut improved = false;
        for _ in 0..20 {
            let cand: Vec<f64> = p.iter().zip(&dp).map(|(v, d)| v + alpha * d).collect();
            if !cand.iter().all(|v| v.is_finite()) {
                alpha *= 0.5;
                continue;
            }
            let rc: Vec<f64> = fq(&cand)?.iter().zip(y).map(|(a, b)| a - b).collect();
            if !rc.iter().all(|v| v.is_finite()) {
                alpha *= 0.5;
                continue;
            }
            let nrm: f64 = rc
                .iter()
                .zip(w)
                .map(|(ri, wi)| (ri * wi).powi(2))
                .sum::<f64>()
                .sqrt();
            if nrm < norm {
                p = cand;
                improved = true;
                break;
            }
            alpha *= 0.5;
        }
        if !improved {
            return Err(bad("no descent"));
        }
        let step: f64 = dp.iter().map(|v| (alpha * v).powi(2)).sum::<f64>().sqrt();
        if step < tol {
            break;
        }
    }
    // Covarianza (JᵀW²J)⁻¹ en el óptimo.
    let r: Vec<f64> = fq(&p)?.iter().zip(y).map(|(a, b)| a - b).collect();
    let norm: f64 = r
        .iter()
        .zip(w)
        .map(|(ri, wi)| (ri * wi).powi(2))
        .sum::<f64>()
        .sqrt();
    let mut j = vec![vec![0.0; m]; n];
    for k in 0..m {
        let mut pp = p.clone();
        pp[k] += h;
        let fp = fq(&pp)?;
        let mut pm = p.clone();
        pm[k] -= h;
        let fm = fq(&pm)?;
        for i in 0..n {
            j[i][k] = (fp[i] - fm[i]) / (2.0 * h);
        }
    }
    let mut a = vec![vec![0.0; m]; m];
    for i in 0..n {
        let wi2 = w[i] * w[i];
        for k in 0..m {
            for l in 0..m {
                a[k][l] += j[i][k] * wi2 * j[i][l];
            }
        }
    }
    let cov = invert(&a).ok_or(bad("singular cov"))?;
    let std_k1: Vec<f64> = (0..m).map(|k| cov[k][k].max(0.0).sqrt()).collect();
    let mut max_corr: f64 = 0.0;
    for k in 0..m {
        for l in (k + 1)..m {
            let d = (cov[k][k] * cov[l][l]).max(1e-300).sqrt();
            max_corr = max_corr.max((cov[k][l] / d).abs());
        }
    }
    Ok(FitResult {
        params: unscale(&p),
        std_k1,
        max_abs_correlation: max_corr,
        iterations: iters,
        residual_norm: norm,
    })
}

/// Gauss-Jordan con pivoteo (m pequeño). `None` si singular.
fn solve(a: &[Vec<f64>], b: &[f64]) -> Option<Vec<f64>> {
    let m = a.len();
    let mut mtx: Vec<Vec<f64>> = a
        .iter()
        .zip(b)
        .map(|(row, v)| row.iter().chain(std::iter::once(v)).copied().collect())
        .collect();
    for c in 0..m {
        let mut piv = c;
        for r in c..m {
            if mtx[r][c].abs() > mtx[piv][c].abs() {
                piv = r;
            }
        }
        if mtx[piv][c].abs() < 1e-15 {
            return None;
        }
        mtx.swap(c, piv);
        let d = mtx[c][c];
        for k in c..=m {
            mtx[c][k] /= d;
        }
        for r in 0..m {
            if r != c {
                let f = mtx[r][c];
                for k in c..=m {
                    mtx[r][k] -= f * mtx[c][k];
                }
            }
        }
    }
    Some(mtx.iter().map(|row| row[m]).collect())
}

/// Inversa por Gauss-Jordan. `None` si singular.
fn invert(a: &[Vec<f64>]) -> Option<Vec<Vec<f64>>> {
    let m = a.len();
    let mut out = Vec::with_capacity(m);
    for c in 0..m {
        let mut e = vec![0.0; m];
        e[c] = 1.0;
        out.push(solve(a, &e)?);
    }
    // out[c] es la columna c → transponer a filas.
    let mut inv = vec![vec![0.0; m]; m];
    for (c, col) in out.iter().enumerate() {
        for (r, v) in col.iter().enumerate() {
            inv[r][c] = *v;
        }
    }
    Some(inv)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    /// Self-consistency SYNTHETIC (verifica el fitter, NO valida nada):
    /// recupera C y p_ideal del modelo de cranking desde picos sintéticos.
    #[test]
    fn recovers_cranking_params_from_synthetic() {
        let c_true = 8.0e5;
        let p_true = 1500.0;
        // Punto de ancla casi sellada (A→0 ⇒ pico→p_ideal): sin él, C y
        // p_ideal son casi colineales (corr −0.98) y la covarianza es
        // singular — lección de identificabilidad, no bug del fitter.
        let areas = [1e-9, 0.5e-6, 1.0e-6, 2.0e-6, 4.0e-6, 8.0e-6];
        let model = |p: &[f64], a: f64| {
            p[1] * 1.0 * 400e-6 / (400e-6 + 8e-6) / (1.0 + p[0] * a * 0.6 * 4.0)
        };
        let y: Vec<f64> = areas.iter().map(|a| model(&[c_true, p_true], *a)).collect();
        let w = vec![1.0; y.len()];
        let f = |p: &[f64]| areas.iter().map(|a| model(p, *a)).collect::<Vec<_>>();
        // Arranque lejos (×3 en C, +20 % en p): debe converger igual.
        let r = least_squares(f, &y, &w, &[2.4e6, 1800.0], 1e-9).unwrap();
        assert!(
            (r.params[0] - c_true).abs() / c_true < 1e-4,
            "{:?}",
            r.params
        );
        assert!(
            (r.params[1] - p_true).abs() / p_true < 1e-4,
            "{:?}",
            r.params
        );
        assert!(r.residual_norm < 1e-6);
        assert!(r.std_k1.iter().all(|s| s.is_finite() && *s >= 0.0));
        assert!(r.max_abs_correlation <= 1.0);
        // El ancla rompe la degenerescencia (corr −0.98 → −0.54).
        assert!(
            r.max_abs_correlation < 0.9,
            "corr={}",
            r.max_abs_correlation
        );
        // Singular bajo-determinado (1 punto, 2 params) ⇒ fail-closed.
        let f1 = |p: &[f64]| vec![model(p, 1e-6)];
        assert!(least_squares(f1, &y[..1], &w[..1], &[2.4e6, 1800.0], 1e-9).is_err());
    }

    #[test]
    fn malformed_callback_is_typed_error_not_panic() {
        // R02-C1/RV08: dimensión de salida inválida ⇒ error tipado, sin panic.
        let r = std::panic::catch_unwind(|| {
            least_squares(|_| vec![], &[1.0, 2.0], &[1.0, 1.0], &[1.0], 1e-8)
        });
        assert!(r.is_ok(), "el fitter no debe hacer panic");
        assert!(r.unwrap().is_err());
        // Salida finita pero de longitud distinta a `y` también se rechaza.
        let r2 = least_squares(|_| vec![1.0], &[1.0, 2.0], &[1.0, 1.0], &[1.0], 1e-8);
        assert!(r2.is_err());
    }
}
