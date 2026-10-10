//! Eventos por cilindro desde UN árbol y UN cigüeñal compartidos (gate S11).
//!
//! Cada cilindro deriva su alzada de la misma leva desplazada a su fase de
//! ciclo + el offset de correa. El volumen atrapado usa la cinemática S06
//! en el cierre de admisión.

use crate::belt::BeltDrive;
use crate::cam::CamSpec;
use contacts::{check_piston_valve, ContactReport, PistonValveInput};
use kinematics::{cylinder_local_angle, phases::CylinderId, GeometryPack};
use thiserror::Error;

/// Error de eventos.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum EventsError {
    /// Entrada inválida (huelgo, paso).
    #[error("invalid valve-events input: {reason}")]
    InvalidInput { reason: String },
}

/// Eventos y estado de un cilindro.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CylinderValveEvents {
    pub cyl: CylinderId,
    /// IO/IC/EO/EC globales (grados de ciclo 0–720).
    pub io_deg: f64,
    pub ic_deg: f64,
    pub eo_deg: f64,
    pub ec_deg: f64,
    pub ic_local_deg: f64,
    /// Volumen atrapado en IC (m³) según cinemática S06.
    pub trapped_volume_m3: f64,
    /// Huelgo mínimo pistón–válvula en el ciclo (m) y su ángulo global.
    pub min_clearance_m: f64,
    pub min_clearance_at_deg: f64,
    /// ¿Hay solape geométrico en algún punto? (reporte, no daño).
    pub overlap: bool,
}

/// Deriva los cuatro cilindros del mismo árbol + cigüeñal.
///
/// `intake_center_global`/`exhaust_center_global`: centros de lóbulo en el
/// ciclo global para el cilindro de referencia (Cyl1); el resto rota por su
/// fase. `closed_gap_m`: huelgo con válvula cerrada (entrada ilustrativa).
///
/// # Errors
///
/// `InvalidInput` si el huelgo es negativo/no finito o el paso ≤ 0.
#[allow(clippy::too_many_arguments)]
pub fn cylinder_valve_events(
    geom: &GeometryPack,
    intake: &CamSpec,
    exhaust: &CamSpec,
    belt: &BeltDrive,
    closed_gap_m: f64,
    sample_step_deg: f64,
) -> Result<[CylinderValveEvents; 4], EventsError> {
    if !(closed_gap_m.is_finite() && closed_gap_m >= 0.0) {
        return Err(EventsError::InvalidInput {
            reason: "closed_gap must be finite and >= 0".to_owned(),
        });
    }
    if !(sample_step_deg.is_finite() && sample_step_deg > 0.0 && sample_step_deg <= 5.0) {
        return Err(EventsError::InvalidInput {
            reason: "sample step must be in (0, 5]".to_owned(),
        });
    }
    let shift = belt.cam_shift_crank_deg();
    // Centros globales para Cyl1: admisión centrada en su ICL tras el PMS
    // de cruce (local 0 ≡ global base+0)... se expresan en locales y se
    // trasladan por fase: el árbol es ÚNICO, el desplazamiento es común.
    let cyls = [
        CylinderId::Cyl1,
        CylinderId::Cyl2,
        CylinderId::Cyl3,
        CylinderId::Cyl4,
    ];
    let mut out = Vec::with_capacity(4);
    for cyl in cyls {
        // Local 0 del cilindro en global: invertir cylinder_local_angle.
        // local = (global − base + 360) % 720 ⇒ global = (local + base − 360) % 720.
        let base = kinematics::phases::firing_tdc_720(cyl);
        let to_global = |local: f64| (local + base - 360.0).rem_euclid(720.0);
        // Centros de lóbulo en locales (ICL tras cruce, ECL antes de cruce).
        let i_center_local = intake.centerline_atdc_deg;
        let e_center_local = 720.0 - exhaust.centerline_atdc_deg;
        let i_ev = intake.advertised_events();
        let e_ev = exhaust.advertised_events();
        let io = to_global(i_center_local + i_ev.open_deg + shift);
        let ic = to_global(i_center_local + i_ev.close_deg + shift);
        let eo = to_global(e_center_local + e_ev.open_deg + shift);
        let ec = to_global(e_center_local + e_ev.close_deg + shift);
        let ic_local = i_center_local + i_ev.close_deg + shift;
        // Volumen atrapado en IC (posición de manivela del IC).
        let crank_ic = kinematics::phases::crank_angle_for_piston(ic_local.rem_euclid(720.0));
        let trapped = geom.volume(crank_ic.to_radians());
        // Huelgo mínimo barriendo el ciclo con ambas válvulas.
        let mut min_c = f64::INFINITY;
        let mut min_at = 0.0;
        let mut overlap = false;
        let mut g = 0.0;
        while g < 720.0 {
            let local = cylinder_local_angle(g, cyl);
            let crank = kinematics::phases::crank_angle_for_piston(local);
            let x = geom.displacement(crank.to_radians());
            // Alzada de cada válvula a este global (centro + shift).
            let li = intake.lift_at(local - i_center_local - shift);
            let le = exhaust.lift_at(local - e_center_local - shift);
            for lift_mm in [li, le] {
                let rep: ContactReport = check_piston_valve(PistonValveInput {
                    closed_gap_m,
                    lift_m: lift_mm / 1_000.0,
                    cant_rad: 0.0,
                    valve_dia_m: 0.030,
                    piston_x_m: x,
                })
                .map_err(|_| EventsError::InvalidInput {
                    reason: "contact inputs out of range".to_owned(),
                })?;
                if rep.clearance_m < min_c {
                    min_c = rep.clearance_m;
                    min_at = g;
                }
                overlap = overlap || rep.overlap;
            }
            g += sample_step_deg;
        }
        out.push(CylinderValveEvents {
            cyl,
            io_deg: io,
            ic_deg: ic,
            eo_deg: eo,
            ec_deg: ec,
            ic_local_deg: ic_local,
            trapped_volume_m3: trapped,
            min_clearance_m: min_c,
            min_clearance_at_deg: min_at,
            overlap,
        });
    }
    Ok([out[0], out[1], out[2], out[3]])
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::cam::CamSpec;

    fn setup() -> (GeometryPack, CamSpec, CamSpec, BeltDrive) {
        (
            GeometryPack::illustrative_d16_like().unwrap(),
            CamSpec::proxy_d16a6_intake().unwrap(),
            CamSpec::proxy_d16a6_exhaust().unwrap(),
            BeltDrive::new(100.0, 0.0).unwrap(),
        )
    }

    #[test]
    fn all_cylinders_derive_from_shared_cam_and_crank() {
        // Gate S11: los 4 cilindros comparten árbol y cigüeñal; los eventos
        // difieren exactamente en el desfase de encendido (múltiplos de 180°).
        let (geom, i, e, belt) = setup();
        let ev = cylinder_valve_events(&geom, &i, &e, &belt, 0.006, 0.5).unwrap();
        let mut ios: Vec<f64> = ev.iter().map(|c| c.io_deg).collect();
        ios.sort_by(|a, b| a.total_cmp(b));
        for w in ios.windows(2) {
            assert!((w[1] - w[0] - 180.0).abs() < 1e-9, "ios={ios:?}");
        }
        // Volumen atrapado idéntico en los 4 (mismo árbol, misma leva).
        for c in &ev[1..] {
            assert!((c.trapped_volume_m3 - ev[0].trapped_volume_m3).abs() < 1e-15);
        }
        // Con leva stock a tiempo, huelgo 6 mm ⇒ sin solape: informe, no daño.
        assert!(ev.iter().all(|c| !c.overlap));
    }

    #[test]
    fn belt_offset_shifts_events_and_trapped_volume_without_damage() {
        let (geom, i, e, _) = setup();
        let straight = BeltDrive::new(100.0, 0.0).unwrap();
        let shifted = BeltDrive::new(100.0, 1.0).unwrap();
        let a = cylinder_valve_events(&geom, &i, &e, &straight, 0.006, 0.5).unwrap();
        let b = cylinder_valve_events(&geom, &i, &e, &shifted, 0.006, 0.5).unwrap();
        // 1 diente = 7.2° de cigüeñal en todos los eventos.
        for (x, y) in a.iter().zip(b.iter()) {
            let d = (y.io_deg - x.io_deg).rem_euclid(720.0);
            assert!((d - 7.2).abs() < 1e-9, "shift={d}");
            // El volumen atrapado cambia (clearance/volumen efectivo).
            assert!((y.trapped_volume_m3 - x.trapped_volume_m3).abs() > 1e-12);
        }
        // Desfase de 5 dientes (36° cigüeñal) con huelgo 0.5 mm: la leva de
        // escape cubre el PMS de cruce y el contacto geométrico aparece en
        // el informe —el veredicto vive en `contacts` (NotAssessed).
        let slipped = BeltDrive::new(100.0, 5.0).unwrap();
        let tight = cylinder_valve_events(&geom, &i, &e, &slipped, 0.0005, 0.5).unwrap();
        assert!(tight.iter().any(|c| c.overlap));
    }
}
