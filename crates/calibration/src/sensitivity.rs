//! Sensibilidad S25-T5: elasticidades locales + efectos elementales de
//! Morris (trayectorias deterministas, sin RNG). Opera sobre el modelo
//! inyectado; el ranking guía QUÉ medir, no sustituye la medición.

/// Elasticidad local `(p/y)·dy/dp` por diferencias centrales.
#[must_use]
pub fn local_elasticities<F>(f: F, p: &[f64], names: &[&str]) -> Vec<(String, f64)>
where
    F: Fn(&[f64]) -> f64,
{
    let y0 = f(p);
    names
        .iter()
        .enumerate()
        .map(|(k, n)| {
            let h = (p[k].abs() * 1e-5).max(1e-12);
            let mut pp = p.to_vec();
            pp[k] += h;
            let yp = f(&pp);
            let mut pm = p.to_vec();
            pm[k] -= h;
            let ym = f(&pm);
            let e = if y0.abs() > 1e-300 && p[k] != 0.0 {
                p[k] / y0 * (yp - ym) / (2.0 * h)
            } else {
                0.0
            };
            ((*n).to_owned(), e)
        })
        .collect()
}

/// Efectos elementales de Morris con rejilla determinista: `n_traj`
/// trayectorias desde esquinas alternas, paso `delta` en [0,1] normalizado
/// a los rangos `bounds`. Devuelve (nombre, mu*, sigma) ordenado por mu*.
#[must_use]
pub fn morris_effects<F>(
    f: F,
    names: &[&str],
    bounds: &[(f64, f64)],
    n_traj: usize,
) -> Vec<(String, f64, f64)>
where
    F: Fn(&[f64]) -> f64,
{
    let m = names.len();
    let delta = 0.4;
    let mut eff: Vec<Vec<f64>> = vec![Vec::new(); m];
    for t in 0..n_traj.max(1) {
        // Punto base alterno determinista (sin RNG).
        let base: Vec<f64> = bounds
            .iter()
            .enumerate()
            .map(|(k, (lo, hi))| {
                let frac = if (t + k) % 2 == 0 { 0.2 } else { 0.4 };
                lo + frac * (hi - lo)
            })
            .collect();
        // Orden de pasos rotado por trayectoria (determinista).
        for s in 0..m {
            let k = (s + t) % m;
            let (lo, hi) = bounds[k];
            let mut x0 = base.clone();
            x0[k] = lo + 0.2 * (hi - lo);
            let mut x1 = x0.clone();
            x1[k] = (x0[k] + delta * (hi - lo)).min(hi);
            let ee = (f(&x1) - f(&x0)) / (x1[k] - x0[k]) * (hi - lo);
            eff[k].push(ee.abs());
        }
    }
    let mut out: Vec<(String, f64, f64)> = names
        .iter()
        .enumerate()
        .map(|(k, n)| {
            let v = &eff[k];
            let mu = v.iter().sum::<f64>() / v.len().max(1) as f64;
            let sig = if v.len() > 1 {
                (v.iter().map(|e| (e - mu).powi(2)).sum::<f64>() / (v.len() - 1) as f64).sqrt()
            } else {
                0.0
            };
            ((*n).to_owned(), mu, sig)
        })
        .collect();
    out.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    out
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn ranking_orders_cranking_drivers() {
        // Pico de cranking L1: el área de fuga domina sobre el volumen
        // muerto en el punto nominal (A=2e-6, Vd=8e-6).
        let f = |p: &[f64]| {
            let (a, vd) = (p[0], p[1]);
            1500.0 * 400e-6 / (400e-6 + vd) / (1.0 + 8.0e5 * a * 0.6 * 4.0)
        };
        let names = ["leak_area", "dead_vol"];
        let el = local_elasticities(f, &[2e-6, 8e-6], &names);
        let ea = el.iter().find(|(n, _)| n == "leak_area").unwrap().1.abs();
        let ev = el.iter().find(|(n, _)| n == "dead_vol").unwrap().1.abs();
        assert!(ea > ev, "{el:?}");
        let mo = morris_effects(f, &names, &[(0.5e-6, 8e-6), (4e-6, 12e-6)], 4);
        assert_eq!(mo[0].0, "leak_area", "{mo:?}");
        assert!(mo.iter().all(|(_, mu, s)| mu.is_finite() && *s >= 0.0));
    }
}
