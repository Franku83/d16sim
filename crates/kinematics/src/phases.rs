//! Fases de 4 cilindros sobre un cigüeñal compartido (orden 1-3-4-2).
//!
//! Dos cilindros pueden estar en PMS simultáneamente pero en carreras
//! distintas: la posición del pistón depende del ángulo de cigüeñal
//! (módulo 360°) mientras la carrera depende del ángulo de ciclo (720°).

/// Identificador de cilindro (1–4, orden físico del bloque).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CylinderId {
    Cyl1,
    Cyl2,
    Cyl3,
    Cyl4,
}

/// Carreras del ciclo de 4 tiempos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stroke {
    Intake,
    Compression,
    Expansion,
    Exhaust,
}

/// Eventos de combustión cada 180° de ciclo solo si orden y configuración
/// se validan: aquí el orden 1-3-4-2 es candidato PENDING (S01), usado como
/// ilustración estructural, no como dato OEM.
pub const FIRING_ORDER: [CylinderId; 4] = [
    CylinderId::Cyl1,
    CylinderId::Cyl3,
    CylinderId::Cyl4,
    CylinderId::Cyl2,
];

/// Las cuatro carreras en orden de ciclo.
pub const STROKES: [Stroke; 4] = [
    Stroke::Intake,
    Stroke::Compression,
    Stroke::Expansion,
    Stroke::Exhaust,
];

/// Offset del PMS de encendido de cada cilindro en el ciclo 720°.
/// Asignación compatible con orden 1-3-4-2 y compañeras (1,4) y (2,3):
/// Cyl1→360, Cyl3→540, Cyl4→0, Cyl2→180. El orden es candidato PENDING
/// (S01); la estructura de compañeras es ilustrativa.
#[must_use]
pub fn firing_tdc_720(cyl: CylinderId) -> f64 {
    match cyl {
        CylinderId::Cyl1 => 360.0,
        CylinderId::Cyl3 => 540.0,
        CylinderId::Cyl4 => 0.0,
        CylinderId::Cyl2 => 180.0,
    }
}

/// Ángulo local de ciclo (0–720) de un cilindro dado el ángulo global.
#[must_use]
pub fn cylinder_local_angle(global_theta720_deg: f64, cyl: CylinderId) -> f64 {
    let base = firing_tdc_720(cyl);
    // Local 360° = PMS de encendido propio.
    (global_theta720_deg - base + 360.0).rem_euclid(720.0)
}

/// Carrera de un cilindro a su ángulo local.
#[must_use]
pub fn stroke_at(local_theta720_deg: f64) -> Stroke {
    match local_theta720_deg {
        x if x < 180.0 => Stroke::Intake,
        x if x < 360.0 => Stroke::Compression,
        x if x < 540.0 => Stroke::Expansion,
        _ => Stroke::Exhaust,
    }
}

/// Ángulo de manivela (0–360, 0 = PMS) para la cinemática del pistón.
#[must_use]
pub fn crank_angle_for_piston(local_theta720_deg: f64) -> f64 {
    local_theta720_deg.rem_euclid(360.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_cylinders_share_one_crank_with_distinct_strokes() {
        // Global 360° = PMS de encendido de Cyl1. Cyl4 está en PMS al mismo
        // tiempo pero en cruce (escape→admisión), no en encendido.
        let g = 360.0;
        assert_eq!(cylinder_local_angle(g, CylinderId::Cyl1), 360.0);
        assert_eq!(cylinder_local_angle(g, CylinderId::Cyl4), 0.0);
        assert_eq!(
            stroke_at(cylinder_local_angle(g, CylinderId::Cyl1)),
            Stroke::Expansion
        );
        assert_eq!(
            stroke_at(cylinder_local_angle(g, CylinderId::Cyl4)),
            Stroke::Intake
        );
        // Misma posición de manivela (PMS), carreras distintas.
        assert_eq!(
            crank_angle_for_piston(cylinder_local_angle(g, CylinderId::Cyl1)),
            crank_angle_for_piston(cylinder_local_angle(g, CylinderId::Cyl4))
        );
    }

    #[test]
    fn firing_spacing_is_180_degrees() {
        let mut tdcs = FIRING_ORDER.map(firing_tdc_720);
        tdcs.sort_by(|a, b| a.total_cmp(b));
        for w in tdcs.windows(2) {
            assert!((w[1] - w[0] - 180.0).abs() < 1e-12);
        }
    }

    #[test]
    fn companions_share_crank_angle() {
        // (1,4) y (2,3) comparten posición de manivela en todo global.
        for g in [0.0, 90.0, 200.0, 359.0, 541.0] {
            assert_eq!(
                crank_angle_for_piston(cylinder_local_angle(g, CylinderId::Cyl1)),
                crank_angle_for_piston(cylinder_local_angle(g, CylinderId::Cyl4))
            );
            assert_eq!(
                crank_angle_for_piston(cylinder_local_angle(g, CylinderId::Cyl2)),
                crank_angle_for_piston(cylinder_local_angle(g, CylinderId::Cyl3))
            );
        }
    }
}
