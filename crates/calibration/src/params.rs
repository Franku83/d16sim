//! Parámetros calibrables S25-T2: solo identificables con priors físicos.
//! Todo lo demás (mapas OEM, curvas de bomba, k de película, p_ref de
//! junta, Cd de anillo fino...) figura como NO_IDENTIFIABLE_YET con el
//! dato que lo desbloquearía.

/// Parámetro con prior físico y estado de identificabilidad.
#[derive(Debug, Clone, PartialEq)]
pub struct CalParam {
    /// Nombre estable (p. ej. `crank_c_leak`).
    pub name: String,
    pub unit: String,
    pub prior_lo: f64,
    pub prior_hi: f64,
    pub subsystem: String,
    /// true = ajustable con el banco disponible; false = pendiente de dato.
    pub identifiable: bool,
    /// Dato que desbloquearía el ajuste (vacío si ya es identificable).
    pub unlocks_with: String,
}

/// Set de calibración S25: 6 identificables (banco de taller: compresión,
/// leak-down, presión de aceite, ECT) + 8 pendientes de dyno/banco motor.
#[must_use]
pub fn calibration_set() -> Vec<CalParam> {
    let p =
        |name: &str, unit: &str, lo: f64, hi: f64, sub: &str, id: bool, unlock: &str| CalParam {
            name: name.to_owned(),
            unit: unit.to_owned(),
            prior_lo: lo,
            prior_hi: hi,
            subsystem: sub.to_owned(),
            identifiable: id,
            unlocks_with: unlock.to_owned(),
        };
    vec![
        // Identificables con taller (compresímetro + leak-down + manómetros).
        p("crank_c_leak", "1/(m2 s)", 1e5, 5e6, "mech", true, ""),
        p("crank_p_ideal_kpa", "kPa", 800.0, 2200.0, "mech", true, ""),
        p(
            "gasket_p_seal_ref_MPa",
            "MPa",
            20.0,
            400.0,
            "leak",
            true,
            "",
        ),
        p("ring_cd", "1", 0.005, 0.2, "leak", true, ""),
        p(
            "oil_pump_disp_cc_rev",
            "cc/rev",
            4.0,
            14.0,
            "lube",
            true,
            "",
        ),
        p("thermal_wall_frac", "1", 0.1, 0.5, "thermal", true, ""),
        // Pendientes de dyno / banco motor / instrumentación (P1 externo).
        p(
            "wiebe_a",
            "1",
            2.0,
            8.0,
            "combustion",
            false,
            "p-theta o torque vs CA",
        ),
        p(
            "wiebe_m",
            "1",
            1.0,
            4.0,
            "combustion",
            false,
            "p-theta o torque vs CA",
        ),
        p(
            "wiebe_dur_deg",
            "deg",
            40.0,
            90.0,
            "combustion",
            false,
            "p-theta",
        ),
        p(
            "friction_k_base",
            "adim",
            0.5,
            2.0,
            "crank",
            false,
            "motored torque vs rpm",
        ),
        p(
            "vol_eff_scale",
            "adim",
            0.7,
            1.1,
            "gas-exchange",
            false,
            "MAF/MAP vs rpm dyno",
        ),
        p(
            "injector_Q_stat",
            "cc/min",
            150.0,
            260.0,
            "fuel",
            false,
            "banco de inyectores",
        ),
        p(
            "lube_k_film",
            "adim",
            1e3,
            1e7,
            "lube",
            false,
            "presión vs T+rpm mapeada",
        ),
        p(
            "ecu_spark_base_deg",
            "deg",
            -30.0,
            10.0,
            "ecu",
            false,
            "mapa P2E medido en banco",
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_marks_identifiable_and_blocked() {
        let s = calibration_set();
        let id = s.iter().filter(|p| p.identifiable).count();
        let blocked = s.iter().filter(|p| !p.identifiable).count();
        assert!(id >= 4 && blocked >= 4, "id={id} blocked={blocked}");
        // Todo bloqueado nombra el dato que lo desbloquea (vacío = deuda).
        for p in s.iter().filter(|p| !p.identifiable) {
            assert!(!p.unlocks_with.is_empty(), "{}", p.name);
            assert!(p.prior_lo < p.prior_hi, "{}", p.name);
        }
    }
}
