//! Datos de flujo medidos (S12): Cd por válvula con procedencia.
//!
//! - Admisión Y7: shootout Bad Guys 2020 @28" H₂O, cifras POR CILINDRO ⇒
//!   por válvula = publicado/2 (prueba isentrópica, research v0.1 §1.3–1.4).
//!   Clase FORUM-MEASURED; ±7% banco-a-banco + sesgo de vendor desconocido.
//! - Escape: dataset P2A-06 (dodo-upgrades 2010, banco casero), misma
//!   interpretación por-cilindro; ±10% propuesto (research donantes §2).
//!   La identidad de fundición con el Y7 requiere verificación.
//! - Diámetros 30/26 mm: catálogos MFR-TECH (Ø OEM en doc Honda: MISSING).
//! - Guardrail §24.4: estos Cd ya contienen la limitación del puerto
//!   (área de cortina como referencia); NO multiplicar por
//!   min(cortina, garganta) —sería doble conteo.

/// Diámetro de válvula de admisión (mm, MFR-TECH candidato).
pub const INTAKE_VALVE_DIA_MM: f64 = 30.0;
/// Diámetro de válvula de escape (mm, MFR-TECH candidato).
pub const EXHAUST_VALVE_DIA_MM: f64 = 26.0;

/// (lift_pulgadas, Cd_por_válvula) admisión Y7 stock.
pub const INTAKE_CD: [(f64, f64); 10] = [
    (0.050, 0.641),
    (0.100, 0.669),
    (0.150, 0.704),
    (0.200, 0.678),
    (0.250, 0.622),
    (0.300, 0.547),
    (0.350, 0.478),
    (0.400, 0.425),
    (0.450, 0.381),
    (0.500, 0.343),
];

/// (lift_pulgadas, Cd_por_válvula) escape P2A-06 (huecos en 0.250/0.350:
/// la interpolación cubre; mayor incertidumbre declarada).
pub const EXHAUST_CD: [(f64, f64); 7] = [
    (0.050, 0.628),
    (0.100, 0.671),
    (0.150, 0.667),
    (0.200, 0.611),
    (0.300, 0.460),
    (0.400, 0.353),
    (0.450, 0.316),
];

/// Cd de admisión por válvula a alzada `lift_mm` (interpola; fuera de
/// 0.050–0.500" satura al extremo: extrapolación marcada por el llamante).
#[must_use]
pub fn intake_cd(lift_mm: f64) -> f64 {
    lookup(&INTAKE_CD, lift_mm / 25.4)
}

/// Cd de escape por válvula a alzada `lift_mm`.
#[must_use]
pub fn exhaust_cd(lift_mm: f64) -> f64 {
    lookup(&EXHAUST_CD, lift_mm / 25.4)
}

fn lookup(table: &[(f64, f64)], lift_in: f64) -> f64 {
    if lift_in <= 0.0 {
        return 0.0;
    }
    if lift_in <= table[0].0 {
        return table[0].1;
    }
    if lift_in >= table[table.len() - 1].0 {
        return table[table.len() - 1].1;
    }
    for w in table.windows(2) {
        if lift_in <= w[1].0 {
            let t = (lift_in - w[0].0) / (w[1].0 - w[0].0);
            return w[0].1 + t * (w[1].1 - w[0].1);
        }
    }
    table[table.len() - 1].1
}

/// Área de cortina (m²) = π·D·L.
#[must_use]
pub fn curtain_area_m2(valve_dia_mm: f64, lift_mm: f64) -> f64 {
    std::f64::consts::PI * (valve_dia_mm / 1_000.0) * (lift_mm / 1_000.0).max(0.0)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn cd_shape_matches_measured_peak_and_decay() {
        // Pico ≈0.70 @0.150" y decaimiento monótono (research §1.4).
        assert!((intake_cd(0.150 * 25.4) - 0.704).abs() < 1e-12);
        assert!(intake_cd(0.300 * 25.4) < intake_cd(0.150 * 25.4));
        assert!(intake_cd(0.500 * 25.4) < intake_cd(0.300 * 25.4));
        assert_eq!(intake_cd(0.0), 0.0);
        assert_eq!(intake_cd(-1.0), 0.0);
        // Escape decae más rápido desde 0.200" (research donantes §2.2).
        assert!(exhaust_cd(0.400 * 25.4) < exhaust_cd(0.200 * 25.4));
    }
}
