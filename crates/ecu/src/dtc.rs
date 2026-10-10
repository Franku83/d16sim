//! Monitores y DTC (S18): el código nace de lecturas temporales con
//! umbrales y persistencia, nunca de un `fault_id`.
//!
//! Códigos `REF-*` de referencia (no Honda OEM): el mapeo a P0xxx exige el
//! manual de la variante y estrategia documentada (S01 pendiente).

use thiserror::Error;

/// Error de monitores.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum DtcError {
    /// Configuración inválida (umbrales, persistencia).
    #[error("invalid monitor config: {reason}")]
    InvalidConfig { reason: String },
}

/// Código de referencia con contador de persistencia.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dtc {
    /// p. ej. "REF-CKP-LOSS" (referencia, no P0335).
    pub code: String,
    /// Ciclos de confirmación acumulados.
    pub confirmed_cycles: u32,
}

/// Conjunto activo (lo que el scan tool leería en S20).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DtcSet {
    pub active: Vec<Dtc>,
}

/// Monitor de señal CKP: confirma REF-CKP-LOSS tras N ciclos con confianza
/// bajo umbral o estado SyncLost; se borra tras M ciclos sanos.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CkpLossMonitor {
    /// Ciclos para confirmar.
    pub confirm_cycles: u32,
    /// Ciclos sanos para borrar.
    pub heal_cycles: u32,
    /// Umbral de confianza.
    pub confidence_threshold: f64,
    pending: u32,
    healing: u32,
}

/// Evento de monitor (para tests y bitácora).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitorEvent {
    Pending,
    Confirmed,
    Healed,
    NoChange,
}

impl CkpLossMonitor {
    /// # Errors
    ///
    /// `InvalidConfig` si persistencia/umbral no son válidos.
    pub fn new(
        confirm_cycles: u32,
        heal_cycles: u32,
        confidence_threshold: f64,
    ) -> Result<Self, DtcError> {
        if confirm_cycles == 0 || heal_cycles == 0 {
            return Err(DtcError::InvalidConfig {
                reason: "persistence must be >= 1".to_owned(),
            });
        }
        if !(confidence_threshold.is_finite() && (0.0..1.0).contains(&confidence_threshold)) {
            return Err(DtcError::InvalidConfig {
                reason: "threshold in [0, 1)".to_owned(),
            });
        }
        Ok(Self {
            confirm_cycles,
            heal_cycles,
            confidence_threshold,
            pending: 0,
            healing: 0,
        })
    }

    /// Evalúa un ciclo (confianza + ¿sync perdido?) y actualiza `set`.
    pub fn evaluate(&mut self, set: &mut DtcSet, confidence: f64, sync_lost: bool) -> MonitorEvent {
        const CODE: &str = "REF-CKP-LOSS";
        let bad = sync_lost || confidence < self.confidence_threshold;
        if bad {
            self.healing = 0;
            if set.active.iter().any(|d| d.code == CODE) {
                return MonitorEvent::NoChange;
            }
            self.pending += 1;
            if self.pending >= self.confirm_cycles {
                set.active.push(Dtc {
                    code: CODE.to_owned(),
                    confirmed_cycles: self.pending,
                });
                return MonitorEvent::Confirmed;
            }
            return MonitorEvent::Pending;
        }
        self.pending = 0;
        if let Some(pos) = set.active.iter().position(|d| d.code == CODE) {
            self.healing += 1;
            if self.healing >= self.heal_cycles {
                set.active.remove(pos);
                self.healing = 0;
                return MonitorEvent::Healed;
            }
        }
        MonitorEvent::NoChange
    }
}

/// Monitor CYP: REF-CYP-LOSS cuando la sinc cae a 360° (CYP ausente).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CypLossMonitor;

impl CypLossMonitor {
    /// Evalúa: `degraded_360` sostenido `confirm` ciclos ⇒ código.
    pub fn evaluate(
        degraded_360: bool,
        confirm: &mut u32,
        need: u32,
        set: &mut DtcSet,
    ) -> MonitorEvent {
        const CODE: &str = "REF-CYP-LOSS";
        if degraded_360 {
            *confirm += 1;
            if *confirm >= need && !set.active.iter().any(|d| d.code == CODE) {
                set.active.push(Dtc {
                    code: CODE.to_owned(),
                    confirmed_cycles: *confirm,
                });
                return MonitorEvent::Confirmed;
            }
            return MonitorEvent::Pending;
        }
        *confirm = 0;
        MonitorEvent::NoChange
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn dtc_needs_temporal_persistence_not_fault_id() {
        // Un ciclo malo no confirma; N sí; M sanos lo borran.
        let mut mon = CkpLossMonitor::new(3, 2, 0.5).unwrap();
        let mut set = DtcSet::default();
        assert_eq!(mon.evaluate(&mut set, 0.0, true), MonitorEvent::Pending);
        assert!(set.active.is_empty());
        assert_eq!(mon.evaluate(&mut set, 0.0, true), MonitorEvent::Pending);
        assert_eq!(mon.evaluate(&mut set, 0.0, true), MonitorEvent::Confirmed);
        assert_eq!(set.active.len(), 1);
        assert_eq!(set.active[0].code, "REF-CKP-LOSS");
        assert_eq!(mon.evaluate(&mut set, 1.0, false), MonitorEvent::NoChange);
        assert_eq!(mon.evaluate(&mut set, 1.0, false), MonitorEvent::Healed);
        assert!(set.active.is_empty());
        assert!(CkpLossMonitor::new(0, 2, 0.5).is_err());
    }
}
