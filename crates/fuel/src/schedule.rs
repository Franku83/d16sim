//! Agenda de inyección (S13): eventos por cilindro y estrategia de
//! agrupación (la estrategia P2E real se fija en S18; aquí solo el
//! mecanismo de reparto).

use thiserror::Error;

/// Error de agenda.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ScheduleError {
    /// Cilindro fuera de 1–4 o lista vacía.
    #[error("invalid injection schedule: {reason}")]
    InvalidSchedule { reason: String },
}

/// Estrategia de agrupación (entrada, no calibración).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InjectionStrategy {
    Sequential,
    GroupedPairs,
    Simultaneous,
}

/// Evento de inyección: temporización por cilindro. `pulse_s` lo fijará
/// el scheduler S18 (aquí 0.0 = aún no programado, nunca una duración).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InjectionEvent {
    pub cylinder: u8,
    pub start_crank_deg: f64,
    pub pulse_s: f64,
}

/// Reparte una masa por cilindro según la estrategia (órdenes de disparo
/// relativos al ciclo; los ángulos absolutos los fija el scheduler S18).
///
/// # Errors
///
/// `InvalidSchedule` si algún cilindro sale de 1–4.
pub fn schedule_masses(
    masses_kg: [f64; 4],
    strategy: InjectionStrategy,
) -> Result<Vec<InjectionEvent>, ScheduleError> {
    for m in masses_kg.iter() {
        if !m.is_finite() || *m < 0.0 {
            return Err(ScheduleError::InvalidSchedule {
                reason: "masses must be finite and >= 0".to_owned(),
            });
        }
    }
    // Ángulos ilustrativos de ventana de inyección por cilindro
    // (el scheduler real S18 los programará por θ/t).
    const BASE: [f64; 4] = [300.0, 120.0, 480.0, 660.0];
    let mut events = Vec::new();
    match strategy {
        InjectionStrategy::Sequential => {
            for cyl in 0..4 {
                events.push(InjectionEvent {
                    cylinder: cyl as u8 + 1,
                    start_crank_deg: BASE[cyl],
                    pulse_s: 0.0,
                });
            }
        }
        InjectionStrategy::GroupedPairs => {
            for cyl in 0..4 {
                events.push(InjectionEvent {
                    cylinder: cyl as u8 + 1,
                    start_crank_deg: if cyl % 2 == 0 { 300.0 } else { 120.0 },
                    pulse_s: 0.0,
                });
            }
        }
        InjectionStrategy::Simultaneous => {
            for cyl in 0..4 {
                events.push(InjectionEvent {
                    cylinder: cyl as u8 + 1,
                    start_crank_deg: 300.0,
                    pulse_s: 0.0,
                });
            }
        }
    }
    Ok(events)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn strategies_cover_all_cylinders() {
        for s in [
            InjectionStrategy::Sequential,
            InjectionStrategy::GroupedPairs,
            InjectionStrategy::Simultaneous,
        ] {
            let ev = schedule_masses([1e-6, 2e-6, 3e-6, 4e-6], s).unwrap();
            let mut cyls: Vec<u8> = ev.iter().map(|e| e.cylinder).collect();
            cyls.sort_unstable();
            assert_eq!(cyls, vec![1, 2, 3, 4]);
        }
        assert!(
            schedule_masses([1e-6, f64::NAN, 0.0, 0.0], InjectionStrategy::Sequential).is_err()
        );
    }
}
