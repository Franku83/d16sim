//! Tres canales del distribuidor (S17): CKP 24 + TDC 4 + CYP 1 a mitad
//! de velocidad, con fallos por estado físico y API de muestreo.
//!
//! Fases ilustrativas (flancos en PMS de encendido + offset de montaje):
//! la FASE EXACTA por variante/distribuidor es PENDING (S01 + scope).

use crate::vr::{Conditioner, DeterministicNoise, Polarity, VrSensor};
use crate::{CkpWheel, TdcWheel, TriggerProvenance};
use thiserror::Error;

/// Error de trigger.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum TriggerError {
    /// Configuración o muestreo inválidos.
    #[error("invalid trigger setup: {reason}")]
    Invalid { reason: String },
}

/// Fallo por estado físico (nunca etiqueta mágica).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TriggerFault {
    /// Todo sano.
    None,
    /// Salta un flanco CKP (índice de diente, determinista).
    MissingCkpEdge { tooth_index: usize },
    /// Canal abierto (cable cortado): el sensor YA trae open_circuit, este
    /// flag elige el canal (CKP = 0, TDC = 1, CYP = 2).
    OpenChannel { channel: u8 },
    /// Entrehierro ×N (amplitud /N: se pierden flancos a bajo régimen).
    WideGap { channel: u8, factor: f64 },
    /// Rotación de la carcasa del distribuidor (° cigüeñal).
    DistributorOffset { offset_crank_deg: f64 },
    /// Caídas intermitentes con seed (determinista, reproducible).
    Intermittent {
        channel: u8,
        seed: u64,
        drop_probability: f64,
    },
}

/// Configuración de los tres canales (procedencia genérica obligatoria).
#[derive(Debug, Clone)]
pub struct TriggerConfig {
    pub provenance: TriggerProvenance,
    pub ckp: VrSensor,
    pub tdc: VrSensor,
    pub cyp: VrSensor,
    pub conditioner: Conditioner,
    pub fault: TriggerFault,
}

impl TriggerConfig {
    /// Configuración genérica de familia (única disponible en S17).
    #[must_use]
    pub fn generic_family() -> Self {
        Self {
            provenance: TriggerProvenance::generic_family(),
            ckp: VrSensor::illustrative(),
            tdc: VrSensor::illustrative(),
            cyp: VrSensor::illustrative(),
            conditioner: Conditioner::new(0.2, 0.1).unwrap_or(Conditioner {
                threshold_v: 0.2,
                hysteresis_v: 0.1,
            }),
            fault: TriggerFault::None,
        }
    }
}

/// Distribuidor virtual: genera forma de onda y flancos por canal.
#[derive(Debug, Clone)]
pub struct DistributorTrigger {
    cfg: TriggerConfig,
    ckp: CkpWheel,
    tdc: TdcWheel,
}

impl DistributorTrigger {
    /// # Errors
    ///
    /// `Invalid` si algún sensor no valida.
    pub fn new(cfg: TriggerConfig) -> Result<Self, TriggerError> {
        let bad = |reason: &str| TriggerError::Invalid {
            reason: reason.to_owned(),
        };
        cfg.ckp.validated().map_err(|_| bad("ckp"))?;
        cfg.tdc.validated().map_err(|_| bad("tdc"))?;
        cfg.cyp.validated().map_err(|_| bad("cyp"))?;
        Ok(Self {
            cfg,
            ckp: CkpWheel::generic_24(),
            tdc: TdcWheel::generic_4(),
        })
    }

    fn offset(&self) -> f64 {
        match self.cfg.fault {
            TriggerFault::DistributorOffset { offset_crank_deg } => offset_crank_deg,
            _ => 0.0,
        }
    }

    fn sensor(&self, channel: u8) -> VrSensor {
        let mut s = match channel {
            0 => self.cfg.ckp,
            1 => self.cfg.tdc,
            _ => self.cfg.cyp,
        };
        match self.cfg.fault {
            TriggerFault::OpenChannel { channel: c } if c == channel => {
                s.open_circuit = true;
            }
            TriggerFault::WideGap { channel: c, factor } if c == channel => {
                s.gap_factor = factor;
            }
            _ => {}
        }
        s
    }

    /// Tensión del canal a (t_s, ángulo global) con ruido determinista.
    fn voltage_at(
        &self,
        channel: u8,
        sensor: VrSensor,
        omega_distributor_rad_s: f64,
        global_deg: f64,
        noise: &mut DeterministicNoise,
    ) -> f64 {
        if sensor.open_circuit {
            return 0.0;
        }
        // Fase eléctrica: dientes × ángulo de distribuidor (mitad).
        // CYP lleva +π para que su flanco caiga en el PMS del cilindro 1
        // (global 360°); CKP/TDC cruzan en 0 por construcción y TDC coincide
        // con los PMS de encendido (ver test de alineación).
        let teeth = match channel {
            0 => f64::from(self.ckp.teeth()),
            1 => f64::from(self.tdc.teeth()),
            _ => 1.0,
        };
        let dist_deg = (global_deg - self.offset()) / 2.0;
        let mut phase = dist_deg.to_radians() * teeth;
        if channel == 2 {
            phase += std::f64::consts::PI;
        }
        let mut v = sensor.amplitude(omega_distributor_rad_s) * phase.sin();
        if sensor.polarity == Polarity::Reversed {
            v = -v;
        }
        v + noise.gaussian(sensor.noise_std_v)
    }

    /// Genera serie (t, v) del canal en `[rev_start, rev_end)` (ciclos
    /// globales) a `sample_rate_hz` y rpm de CIGÜEÑAL dadas.
    ///
    /// # Errors
    ///
    /// `Invalid` si rpm/tasa/revoluciones no son válidas.
    pub fn sample_channel(
        &self,
        channel: u8,
        rpm: f64,
        rev_start: f64,
        rev_end: f64,
        sample_rate_hz: f64,
        seed: u64,
    ) -> Result<Vec<(f64, f64)>, TriggerError> {
        let bad = |reason: &str| TriggerError::Invalid {
            reason: reason.to_owned(),
        };
        if !(rpm.is_finite() && rpm > 0.0) {
            return Err(bad("rpm"));
        }
        if !(sample_rate_hz.is_finite() && sample_rate_hz > 0.0) {
            return Err(bad("sample rate"));
        }
        if !(rev_end > rev_start) {
            return Err(bad("rev range"));
        }
        if channel > 2 {
            return Err(bad("channel 0=CKP 1=TDC 2=CYP"));
        }
        let sensor = self.sensor(channel);
        sensor.validated().map_err(|_| bad("sensor"))?;
        let omega_diz = rpm * 2.0 * std::f64::consts::PI / 60.0 / 2.0;
        // Guía de tasa: la frecuencia máxima de diente CKP es
        // 12 pulsos/rev × rpm/60; se exigen ≥ 20 muestras por periodo
        // (documentado; por debajo se advierte en el resumen, no se falla).
        let mut noise = DeterministicNoise::new(seed.wrapping_add(u64::from(channel) * 7919));
        let t0 = rev_start * 720.0 / 360.0 / (rpm / 60.0);
        let t1 = rev_end * 720.0 / 360.0 / (rpm / 60.0);
        let n = ((t1 - t0) * sample_rate_hz).ceil() as usize + 1;
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let t = t0 + f64::from(i as u32) / sample_rate_hz;
            if t > t1 {
                break;
            }
            // Ángulo global exacto por tiempo (régimen constante S17).
            let global_deg = rev_start * 720.0 + (t - t0) * rpm / 60.0 * 360.0;
            out.push((
                t,
                self.voltage_at(channel, sensor, omega_diz, global_deg, &mut noise),
            ));
        }
        Ok(out)
    }

    /// Flancos de subida del canal (cruces del acondicionador) con fallos
    /// aplicados: borde saltado, canal abierto (vacío) e intermitencia con
    /// seed (determinista).
    ///
    /// # Errors
    ///
    /// Propaga `sample_channel`.
    pub fn channel_edges(
        &self,
        channel: u8,
        rpm: f64,
        rev_start: f64,
        rev_end: f64,
        sample_rate_hz: f64,
        seed: u64,
    ) -> Result<Vec<f64>, TriggerError> {
        let series = self.sample_channel(channel, rpm, rev_start, rev_end, sample_rate_hz, seed)?;
        let mut edges = self.cfg.conditioner.rising_edges(&series);
        match self.cfg.fault {
            TriggerFault::MissingCkpEdge { tooth_index } if channel == 0 => {
                if tooth_index < edges.len() {
                    edges.remove(tooth_index);
                }
            }
            TriggerFault::Intermittent {
                channel: c,
                seed: s,
                drop_probability,
            } if c == channel => {
                let mut rng = DeterministicNoise::new(s);
                edges.retain(|_| rng.uniform01() >= drop_probability.clamp(0.0, 1.0));
            }
            _ => {}
        }
        Ok(edges)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::reluctor::{tdc_teeth_global_deg, CypWheel};

    fn trig() -> DistributorTrigger {
        DistributorTrigger::new(TriggerConfig::generic_family()).unwrap()
    }

    #[test]
    fn tooth_counts_per_720_match_candidate() {
        // 24 CKP + 4 TDC + 1 CYP por ciclo (a mitad de velocidad).
        let t = trig();
        let ckp = t.channel_edges(0, 750.0, 0.0, 1.0, 200_000.0, 1).unwrap();
        let tdc = t.channel_edges(1, 750.0, 0.0, 1.0, 200_000.0, 2).unwrap();
        let cyp = t.channel_edges(2, 750.0, 0.0, 1.0, 200_000.0, 3).unwrap();
        assert_eq!(ckp.len(), 24, "ckp={}", ckp.len());
        assert_eq!(tdc.len(), 4, "tdc={}", tdc.len());
        assert_eq!(cyp.len(), 1, "cyp={}", cyp.len());
        // Espaciado uniforme CKP (~30° de cigüeñal entre flancos).
        let om = 750.0 / 60.0 * 360.0;
        for w in ckp.windows(2) {
            let deg = (w[1] - w[0]) * om;
            assert!((deg - 30.0).abs() < 1.0, "spacing={deg}");
        }
    }

    #[test]
    fn faults_are_structural_not_labels() {
        // Borde saltado: 23 flancos.
        let mut cfg = TriggerConfig::generic_family();
        cfg.fault = TriggerFault::MissingCkpEdge { tooth_index: 5 };
        let t = DistributorTrigger::new(cfg).unwrap();
        assert_eq!(
            t.channel_edges(0, 750.0, 0.0, 1.0, 200_000.0, 1)
                .unwrap()
                .len(),
            23
        );
        // Canal abierto: 0 flancos, 0 V.
        let mut cfg = TriggerConfig::generic_family();
        cfg.fault = TriggerFault::OpenChannel { channel: 1 };
        let t = DistributorTrigger::new(cfg).unwrap();
        assert!(t
            .channel_edges(1, 750.0, 0.0, 1.0, 200_000.0, 2)
            .unwrap()
            .is_empty());
        // Gap ×8 a ralentí: la amplitud cae bajo el umbral y se pierden
        // flancos (a alto régimen se recuperan: amplitud ∝ ω).
        let mut cfg = TriggerConfig::generic_family();
        cfg.fault = TriggerFault::WideGap {
            channel: 0,
            factor: 8.0,
        };
        let t = DistributorTrigger::new(cfg).unwrap();
        let low = t
            .channel_edges(0, 100.0, 0.0, 1.0, 200_000.0, 1)
            .unwrap()
            .len();
        let high = t
            .channel_edges(0, 3000.0, 0.0, 1.0, 200_000.0, 1)
            .unwrap()
            .len();
        assert!(low < high, "low={low} high={high}");
        assert_eq!(high, 24);
    }

    #[test]
    fn distributor_offset_shifts_all_channels() {
        // La rotación de la carcasa desfasa los TRES canales por igual
        // (el cigüeñal y la secuencia mecánica no cambian: solo fase).
        let mut cfg = TriggerConfig::generic_family();
        cfg.fault = TriggerFault::DistributorOffset {
            offset_crank_deg: 10.0,
        };
        let t = DistributorTrigger::new(cfg).unwrap();
        let c0 = trig()
            .channel_edges(0, 750.0, 0.0, 1.0, 200_000.0, 1)
            .unwrap();
        let c1 = t.channel_edges(0, 750.0, 0.0, 1.0, 200_000.0, 1).unwrap();
        assert_eq!(c0.len(), c1.len());
        let om = 750.0 / 60.0 * 360.0;
        assert!(((c1[0] - c0[0]) * om - 10.0).abs() < 1.0);
    }

    #[test]
    fn edges_align_with_firing_tdcs_and_cyl1() {
        // Los flancos TDC caen cerca de los PMS de encendido (0/180/360/540)
        // y el CYP cerca del cilindro 1 (360°): el cruce a umbral fijo
        // retrasa arcsin(hi/A)/2 ≈ 3.7° de forma sistemática y documentada
        // (dependencia de amplitud del VR, no error de fase).
        let t = trig();
        let om = 750.0 / 60.0 * 360.0;
        let tdc = t.channel_edges(1, 750.0, 0.0, 1.0, 200_000.0, 2).unwrap();
        let cyp = t.channel_edges(2, 750.0, 0.0, 1.0, 200_000.0, 3).unwrap();
        assert_eq!(tdc.len(), 4);
        assert_eq!(cyp.len(), 1);
        let tdc_deg: Vec<f64> = tdc.iter().map(|e| e * om).collect();
        for (got, want) in tdc_deg.iter().zip(tdc_teeth_global_deg(0.0).iter()) {
            let mut d = (got - want).abs();
            if d > 360.0 {
                d = 720.0 - d;
            }
            assert!(d < 6.0, "tdc edge at {got}, want {want}");
        }
        let mut dc = (cyp[0] * om - CypWheel::tooth_global_deg(0.0)).abs();
        if dc > 360.0 {
            dc = 720.0 - dc;
        }
        // El CYP (1 diente/rev de árbol) cruza lento: el retardo de umbral
        // fijo se duplica en grados de cigüeñal (~15°). Identificación del
        // diente (único por ciclo), no precisión de flanco.
        assert!(dc < 20.0);
    }
}
