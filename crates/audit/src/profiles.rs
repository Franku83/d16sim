//! Perfiles numéricos S30-T2: mismo punto en 3 pasos de integración.
//! Cuantifica diferencias sin ajustar nada (medido: spread 0.6 % en rpm,
//! cierre de energía ∝ dt por el splitting de primer orden, S16).

use thiserror::Error;

/// Error de perfiles.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ProfileError {
    /// Configuración inválida o el motor falló.
    #[error("invalid profile: {reason}")]
    Invalid { reason: String },
}

/// Fila de perfil (un paso temporal).
#[derive(Debug, Clone, PartialEq)]
pub struct ProfileRow {
    pub dt_s: f64,
    pub rpm: f64,
    pub mass_closure: f64,
    pub energy_closure: f64,
}

/// Corre el punto (mariposa 0.3e-4, carga 8+2 Nm, 3 s) en los 3 perfiles.
///
/// # Errors
///
/// `Invalid` si el motor falla en algún perfil.
pub fn audit_profiles() -> Result<Vec<ProfileRow>, ProfileError> {
    let mut rows = Vec::new();
    for dt in [25e-6, 50e-6, 100e-6] {
        let mut inp = engine::healthy_inputs();
        inp.throttle_area_m2 = 0.3e-4;
        inp.accessory_tau_n_m = 10.0;
        inp.t_end_s = 3.0;
        inp.dt_s = dt;
        let t = engine::run_engine(&inp).map_err(|e| ProfileError::Invalid {
            reason: e.to_string(),
        })?;
        rows.push(ProfileRow {
            dt_s: dt,
            rpm: t.audit.final_rpm,
            mass_closure: t.audit.mass_closure,
            energy_closure: t.audit.energy_closure,
        });
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn profiles_agree_and_scale_as_documented() {
        let rows = audit_profiles().unwrap();
        assert_eq!(rows.len(), 3);
        // Spread de régimen < 2 % entre 25 y 100 µs (medido 0.6 %).
        let spread = (rows[0].rpm - rows[2].rpm).abs() / rows[1].rpm.max(1e-9);
        assert!(spread < 0.02, "{rows:?}");
        // Masa siempre ~1e-11; energía ∝ dt (splitting primer orden).
        for r in &rows {
            assert!(r.mass_closure.abs() < 1e-9, "{r:?}");
        }
        assert!(rows[2].energy_closure > rows[0].energy_closure, "{rows:?}");
        assert!(rows[2].energy_closure.abs() < 2e-4, "{rows:?}");
        // Determinismo: repetir el perfil fino da lo mismo.
        let again = audit_profiles().unwrap();
        assert_eq!(rows, again);
    }
}
