//! Cableado (S19): resistencia, abierto, cortos e intermitencia.
//!
//! El cable transforma la tensión del transductor ANTES del ADC: la ECU
//! solo ve el resultado. Fallos por estado físico con seed (reproducible).

use super::sensors::ChannelNoise;
use thiserror::Error;

/// Error de cableado.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum WireError {
    /// Parámetros inválidos (resistencia, impedancia, probabilidad).
    #[error("invalid wire setup: {reason}")]
    InvalidSetup { reason: String },
}

/// Fallo del cable/conector.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WireFault {
    /// Sano (solo R serie contra la impedancia de entrada).
    Ok,
    /// Circuito abierto (el pin flota: modelado a 0 V con pull-down,
    /// documentado).
    Open,
    /// Corto a masa.
    ShortToGround,
    /// Corto a alimentación.
    ShortToSupply,
    /// Intermitente reproducible (probabilidad de corte por muestra).
    Intermittent { seed: u64, cut_probability: f64 },
}

/// Cable + conector + impedancia de entrada del ADC.
#[derive(Debug, Clone)]
pub struct Wire {
    /// Resistencia serie (Ω).
    pub r_ohm: f64,
    /// Impedancia de entrada del ADC (Ω, ilustrativa 1 MΩ).
    pub input_z_ohm: f64,
    /// Tensión de alimentación para cortos (V).
    pub supply_v: f64,
    pub fault: WireFault,
    noise: ChannelNoise,
}

impl Wire {
    /// # Errors
    ///
    /// `InvalidSetup` si R/Z no son válidas o la probabilidad sale de [0,1].
    pub fn new(
        r_ohm: f64,
        input_z_ohm: f64,
        supply_v: f64,
        fault: WireFault,
    ) -> Result<Self, WireError> {
        let bad = |reason: &str| WireError::InvalidSetup {
            reason: reason.to_owned(),
        };
        if !(r_ohm.is_finite() && r_ohm >= 0.0) {
            return Err(bad("resistance"));
        }
        if !(input_z_ohm.is_finite() && input_z_ohm > 0.0) {
            return Err(bad("input impedance"));
        }
        if !(supply_v.is_finite() && supply_v >= 0.0) {
            return Err(bad("supply"));
        }
        if let WireFault::Intermittent {
            cut_probability, ..
        } = fault
        {
            if !(cut_probability.is_finite() && (0.0..=1.0).contains(&cut_probability)) {
                return Err(bad("cut probability"));
            }
        }
        Ok(Self {
            r_ohm,
            input_z_ohm,
            supply_v,
            fault,
            noise: ChannelNoise::new(7193),
        })
    }

    /// Tensión que ve el ADC ante `v_src` del transductor.
    pub fn voltage_at_adc(&mut self, v_src: f64) -> f64 {
        match self.fault {
            WireFault::Ok => v_src * self.input_z_ohm / (self.r_ohm + self.input_z_ohm),
            WireFault::Open => 0.0,
            WireFault::ShortToGround => 0.0,
            WireFault::ShortToSupply => self.supply_v,
            WireFault::Intermittent {
                cut_probability, ..
            } => {
                if self.noise.uniform01() < cut_probability {
                    0.0
                } else {
                    v_src * self.input_z_ohm / (self.r_ohm + self.input_z_ohm)
                }
            }
        }
    }
}

/// Monitor de rango genérico (S19): código REF-* tras persistencia, con
/// curación. Los monitores CKP/CYP viven en `ecu` (S18); este cubre
/// analógicos (MAP/ECT/TPS...).
#[derive(Debug, Clone, PartialEq)]
pub struct RangeMonitor {
    /// Código de referencia (no OEM).
    pub code: String,
    /// Umbrales [lo, hi] de la LECTURA (V).
    pub lo_v: f64,
    pub hi_v: f64,
    /// Ciclos para confirmar / curar.
    pub confirm: u32,
    pub heal: u32,
    pending: u32,
    healing: u32,
    pub active: bool,
}

/// Evento de monitor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeEvent {
    Pending,
    Confirmed,
    Healed,
    NoChange,
}

impl RangeMonitor {
    /// # Errors
    ///
    /// `InvalidSetup` si banda/persistencia no son válidas.
    pub fn new(
        code: &str,
        lo_v: f64,
        hi_v: f64,
        confirm: u32,
        heal: u32,
    ) -> Result<Self, WireError> {
        if !(lo_v.is_finite() && hi_v.is_finite() && lo_v < hi_v) {
            return Err(WireError::InvalidSetup {
                reason: "range band".to_owned(),
            });
        }
        if confirm == 0 || heal == 0 {
            return Err(WireError::InvalidSetup {
                reason: "persistence".to_owned(),
            });
        }
        Ok(Self {
            code: code.to_owned(),
            lo_v,
            hi_v,
            confirm,
            heal,
            pending: 0,
            healing: 0,
            active: false,
        })
    }

    /// Evalúa una lectura y actualiza el estado.
    pub fn evaluate(&mut self, reading_v: f64) -> RangeEvent {
        if !(reading_v.is_finite()) || reading_v < self.lo_v || reading_v > self.hi_v {
            self.healing = 0;
            if self.active {
                return RangeEvent::NoChange;
            }
            self.pending += 1;
            if self.pending >= self.confirm {
                self.active = true;
                return RangeEvent::Confirmed;
            }
            return RangeEvent::Pending;
        }
        self.pending = 0;
        if self.active {
            self.healing += 1;
            if self.healing >= self.heal {
                self.active = false;
                self.healing = 0;
                return RangeEvent::Healed;
            }
        }
        RangeEvent::NoChange
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn wire_faults_are_physical_states() {
        let mut ok = Wire::new(100.0, 1e6, 5.0, WireFault::Ok).unwrap();
        // 100 Ω contra 1 MΩ: caída despreciable pero calculada.
        assert!((ok.voltage_at_adc(3.0) - 3.0 * 1e6 / (100.0 + 1e6)).abs() < 1e-12);
        let mut open = Wire::new(100.0, 1e6, 5.0, WireFault::Open).unwrap();
        assert_eq!(open.voltage_at_adc(3.0), 0.0);
        let mut sh = Wire::new(100.0, 1e6, 5.0, WireFault::ShortToSupply).unwrap();
        assert_eq!(sh.voltage_at_adc(0.5), 5.0);
        // Intermitente reproducible: misma seed ⇒ misma serie.
        let mut a = Wire::new(
            10.0,
            1e6,
            5.0,
            WireFault::Intermittent {
                seed: 7,
                cut_probability: 0.5,
            },
        )
        .unwrap();
        let mut b = a.clone();
        let sa: Vec<f64> = (0..200).map(|_| a.voltage_at_adc(3.0)).collect();
        let sb: Vec<f64> = (0..200).map(|_| b.voltage_at_adc(3.0)).collect();
        assert_eq!(sa, sb);
        assert!(sa.iter().any(|v| *v == 0.0) && sa.iter().any(|v| *v > 0.0));
        assert!(Wire::new(
            10.0,
            1e6,
            5.0,
            WireFault::Intermittent {
                seed: 7,
                cut_probability: 2.0
            }
        )
        .is_err());
    }

    #[test]
    fn range_monitor_confirms_and_heals() {
        let mut m = RangeMonitor::new("REF-ECT-RANGE", 0.2, 4.8, 3, 2).unwrap();
        assert_eq!(m.evaluate(2.5), RangeEvent::NoChange);
        assert_eq!(m.evaluate(9.9), RangeEvent::Pending);
        assert_eq!(m.evaluate(9.9), RangeEvent::Pending);
        assert_eq!(m.evaluate(9.9), RangeEvent::Confirmed);
        assert!(m.active);
        assert_eq!(m.evaluate(2.5), RangeEvent::NoChange);
        assert_eq!(m.evaluate(2.5), RangeEvent::Healed);
        assert!(!m.active);
    }
}
