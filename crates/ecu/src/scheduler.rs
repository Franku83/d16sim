//! Scheduler y FSM de ECU (S18): de sinc + mapas a comandos medibles.
//!
//! Timing SOLICITADO (mapa) vs EFECTIVO (con latencia de proceso y estado
//! de sinc): ambos visibles para pruebas. Sin sinc no hay comandos (salvo
//! fallback documentado); el DTC lo ponen los monitores (dtc.rs).

use crate::decoder::{SyncEstimate, SyncState};
use crate::dtc::DtcSet;
use crate::maps::ReferenceMaps;
use fuel::{Fuel, Injector};
use thiserror::Error;

/// Error de scheduler.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum SchedulerError {
    /// Entradas inválidas (rpm/carga/latencia/deadtime).
    #[error("invalid scheduler input: {reason}")]
    InvalidInput { reason: String },
}

/// Estado de la ECU (FSM S18).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EcuState {
    PowerOff,
    Booting,
    CrankingUnsynced,
    CrankingSynced,
    Running,
    SyncLost,
    /// Degradado 360° (batch + chispa desperdiciada) cuando hay CKP sin CYP.
    FallbackWasted,
}

/// Comando de inyección programado (por cilindro).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InjectionCommand {
    pub cylinder: u8,
    pub start_cycle_deg: f64,
    pub pulse_width_s: f64,
    pub fuel_kg: f64,
}

/// Comando de encendido programado (por cilindro).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IgnitionCommand {
    pub cylinder: u8,
    /// Solicitado por el mapa (° BTDC).
    pub requested_btdc_deg: f64,
    /// Efectivo tras latencia (° BTDC): visible para pruebas.
    pub effective_btdc_deg: f64,
    pub dwell_s: f64,
}

/// Plan de actuación + bitácora de decisiones (inspeccionable).
#[derive(Debug, Clone, PartialEq)]
pub struct ActuationPlan {
    pub ecu_state: EcuState,
    pub injections: Vec<InjectionCommand>,
    pub ignitions: Vec<IgnitionCommand>,
    pub log: Vec<String>,
}

/// ECU de referencia: sinc + mapas + inyector + latencia.
#[derive(Debug, Clone)]
pub struct Ecu {
    pub maps: ReferenceMaps,
    pub fuel: Fuel,
    pub injector: Injector,
    /// Latencia de proceso (s): desplaza el timing efectivo.
    pub processing_latency_s: f64,
    /// Tensión de trabajo para deadtime (V).
    pub volts: f64,
    /// ΔP de trabajo para PW (Pa).
    pub delta_p_pa: f64,
}

impl Ecu {
    /// # Errors
    ///
    /// `InvalidInput` si latencia/tensión/ΔP no son válidas.
    pub fn new(
        maps: ReferenceMaps,
        processing_latency_s: f64,
        volts: f64,
        delta_p_pa: f64,
    ) -> Result<Self, SchedulerError> {
        let bad = |reason: &str| SchedulerError::InvalidInput {
            reason: reason.to_owned(),
        };
        if !(processing_latency_s.is_finite() && processing_latency_s >= 0.0) {
            return Err(bad("latency"));
        }
        if !(volts.is_finite() && volts > 0.0) {
            return Err(bad("volts"));
        }
        if !(delta_p_pa.is_finite() && delta_p_pa > 0.0) {
            return Err(bad("delta_p"));
        }
        Ok(Self {
            maps,
            fuel: Fuel::illustrative_gasoline(),
            injector: Injector::p2a_working_hypothesis(),
            processing_latency_s,
            volts,
            delta_p_pa,
        })
    }

    /// Clasifica el estado desde la estima (umbral de arranque 400 rpm
    /// ilustrativo y declarado).
    #[must_use]
    pub fn classify(&self, est: &SyncEstimate) -> EcuState {
        match est.state {
            SyncState::Unsynced => EcuState::CrankingUnsynced,
            SyncState::SyncLost => EcuState::SyncLost,
            SyncState::Synced360 => EcuState::FallbackWasted,
            SyncState::Synced720 => {
                if est.rpm < 400.0 {
                    EcuState::CrankingSynced
                } else {
                    EcuState::Running
                }
            }
        }
    }

    /// Programa actuación para 4 cilindros desde estima + carga [0, 1].
    /// Registra cada decisión (sinc, mapa, latencia, saturación).
    #[must_use]
    pub fn schedule(&self, est: &SyncEstimate, load_0_1: f64, dtcs: &DtcSet) -> ActuationPlan {
        let mut log = Vec::new();
        let state = self.classify(est);
        log.push(format!(
            "sync={:?} rpm={:.0} conf={:.2} -> {:?}",
            est.state, est.rpm, est.confidence, state
        ));
        if !dtcs.active.is_empty() {
            log.push(format!("dtcs active: {:?}", dtcs.active));
        }
        let load = load_0_1.clamp(0.0, 1.0);
        let mut injections = Vec::new();
        let mut ignitions = Vec::new();
        // Sin sinc no hay comandos (el fallback 360° sí actúa, degradado).
        let sequential = state == EcuState::CrankingSynced || state == EcuState::Running;
        let degraded = state == EcuState::FallbackWasted;
        if !sequential && !degraded {
            log.push("no sync: actuation inhibited".to_owned());
            return ActuationPlan {
                ecu_state: state,
                injections,
                ignitions,
                log,
            };
        }
        if degraded {
            log.push("fallback 360°: batch fuel + wasted spark (CYP missing)".to_owned());
        }
        let fuel_kg = self.maps.fuel_kg.lookup(est.rpm.max(0.0), load);
        let adv = self.maps.spark_btdc_deg.lookup(est.rpm.max(0.0), load);
        // Latencia en grados a este régimen (solicitado vs efectivo).
        let latency_deg = self.processing_latency_s * est.rpm / 60.0 * 360.0;
        let (pulse, _) = self
            .injector
            .pulse_for_mass(self.fuel, fuel_kg.max(0.0), self.delta_p_pa, self.volts)
            .unwrap_or((fuel::PulseCommand { width_s: 0.0 }, 0.0));
        log.push(format!(
            "map: load={load:.2} fuel={fuel_kg:.2e}kg adv={adv:.1}btdc lat={latency_deg:.2}deg"
        ));
        // Orden de encendido ilustrativo 1-3-4-2 (PENDING S01) con PMS en
        // 360/540/0/180 globales (fases S06).
        const ORDER: [(u8, f64); 4] = [(1, 360.0), (3, 540.0), (4, 0.0), (2, 180.0)];
        for (cyl, tdc) in ORDER {
            injections.push(InjectionCommand {
                cylinder: cyl,
                start_cycle_deg: (tdc + 300.0) % 720.0,
                pulse_width_s: pulse.width_s,
                fuel_kg,
            });
            ignitions.push(IgnitionCommand {
                cylinder: cyl,
                requested_btdc_deg: adv,
                effective_btdc_deg: adv - latency_deg,
                dwell_s: 3e-3,
            });
        }
        ActuationPlan {
            ecu_state: state,
            injections,
            ignitions,
            log,
        }
    }
}

/// EcuState de arranque para tests (sin arranque eléctrico aquí).
#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::decoder::{Decoder, SyncState};
    use crate::dtc::DtcSet;

    fn ecu() -> Ecu {
        Ecu::new(
            ReferenceMaps::illustrative().unwrap(),
            1e-3,
            13.5,
            300_000.0,
        )
        .unwrap()
    }

    fn est720(rpm: f64) -> SyncEstimate {
        SyncEstimate {
            state: SyncState::Synced720,
            rpm,
            confidence: 1.0,
            cycle_deg_at: 0.0,
            at_time_s: 0.0,
        }
    }

    #[test]
    fn wrong_signals_alter_actuation_causally() {
        // Gate S18 (i): SyncLost inhibe; 360° degrada; 720° secuencia.
        let e = ecu();
        let empty = DtcSet { active: vec![] };
        let lost = SyncEstimate {
            state: SyncState::SyncLost,
            rpm: 750.0,
            confidence: 0.0,
            cycle_deg_at: 0.0,
            at_time_s: 0.0,
        };
        let p = e.schedule(&lost, 0.4, &empty);
        assert_eq!(p.ecu_state, EcuState::SyncLost);
        assert!(p.injections.is_empty() && p.ignitions.is_empty());
        let deg = SyncEstimate {
            state: SyncState::Synced360,
            ..est720(750.0)
        };
        let p2 = e.schedule(&deg, 0.4, &empty);
        assert_eq!(p2.ecu_state, EcuState::FallbackWasted);
        assert_eq!(p2.injections.len(), 4);
        let ok = e.schedule(&est720(750.0), 0.4, &empty);
        assert_eq!(ok.ecu_state, EcuState::Running);
        assert_eq!(ok.injections.len(), 4);
        // Cranking por debajo de 400 rpm ⩽ Running.
        let cr = e.schedule(&est720(250.0), 0.4, &empty);
        assert_eq!(cr.ecu_state, EcuState::CrankingSynced);
    }

    #[test]
    fn requested_vs_effective_timing_visible() {
        // Gate S18 (ii): la latencia desplaza el efectivo (4.5° a 750 rpm
        // con 1 ms) y queda registrado en el log.
        let e = ecu();
        let empty = DtcSet { active: vec![] };
        let p = e.schedule(&est720(750.0), 0.4, &empty);
        let ig = &p.ignitions[0];
        assert!((ig.requested_btdc_deg - ig.effective_btdc_deg - 4.5).abs() < 1e-9);
        assert!(p.log.iter().any(|l| l.contains("lat=")));
        let _ = Decoder::new(vec![0.0, 0.01], vec![], 0.01).unwrap();
    }
}
