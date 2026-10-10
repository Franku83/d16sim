//! Inyector reducido (S13): masa vs ΔP, duración, deadtime y tensión.
//!
//! - `Q(ΔP) = Q_ref·√(ΔP/P_ref)` (±2% [D] verificada en Bosch).
//! - `PW_total = PW_efectivo + deadtime(V)`; bajo ~2.0 ms efectivos rige
//!   zona no lineal (`nonlinear_region = true`, sin adder inventado).
//! - Deadtime por defecto: mapa analógico CBR600RR OEM Keihin ±0.3 ms
//!   (ANALOGÍA, no P2A-000). Q_ref por defecto: hipótesis 185 cc/min @3 bar
//!   (claim sin ΔP); el llamante debe sustituirla con banco.

use super::rail::Fuel;
use thiserror::Error;

/// Error de inyector.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum InjectorError {
    /// Entradas inválidas (masas, ΔP, tensión, caudal de referencia).
    #[error("invalid injector input: {reason}")]
    InvalidInput { reason: String },
}

/// Pulso COMANDADO (lo que la ECU pedirá en S18).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PulseCommand {
    /// Duración total comandada (s).
    pub width_s: f64,
}

/// Masa ENTREGADA (lo que realmente sale por la tobera).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeliveredMass {
    /// Masa entregada (kg).
    pub mass_kg: f64,
    /// Duración efectiva (s, sin deadtime).
    pub effective_s: f64,
    /// Deadtime aplicado (s).
    pub deadtime_s: f64,
    /// ¿Zona no lineal de pulsos cortos (< 2.0 ms efectivos)?
    pub nonlinear_region: bool,
    /// ¿Demanda recortada por capacidad de bomba?
    pub saturated: bool,
}

/// Estado de entrega (gate S13: sin DTC; la bomba sin presión es estado).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryStatus {
    Delivered,
    /// Pulso menor que el deadtime: no abre.
    BelowDeadtime,
    /// Sin ΔP (bomba apagada/despresurizado): entrega 0, sin código.
    NoPressure,
    /// Capacidad de bomba insuficiente: entrega parcial.
    Saturated,
}

/// Inyector con caudal de referencia y mapa de deadtime explícitos.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Injector {
    /// Caudal de referencia (m³/s) a `p_ref_pa`.
    pub q_ref_m3_s: f64,
    /// Presión diferencial de referencia (Pa).
    pub p_ref_pa: f64,
    /// Deadtime (s) a 10/12/13/14/15 V (mapa analógico por defecto).
    pub deadtime_s_at_v: [f64; 5],
    /// Incertidumbre del deadtime (s, ±).
    pub deadtime_unc_s: f64,
}

impl Injector {
    /// Hipótesis de trabajo P2A-000: 185 cc/min @3.0 bar + deadtime
    /// analógico CBR600RR. **No es medición**: el banco la sustituirá.
    #[must_use]
    pub fn p2a_working_hypothesis() -> Self {
        Self {
            q_ref_m3_s: 185e-6 / 60.0,
            p_ref_pa: 300_000.0,
            deadtime_s_at_v: [1.29e-3, 0.93e-3, 0.74e-3, 0.62e-3, 0.50e-3],
            deadtime_unc_s: 0.3e-3,
        }
    }

    /// Caudal (m³/s) a ΔP dada por ley √ΔP.
    ///
    /// # Errors
    ///
    /// `InvalidInput` si ΔP no es `> 0` finita.
    pub fn flow_at(&self, delta_p_pa: f64) -> Result<f64, InjectorError> {
        if !(delta_p_pa.is_finite() && delta_p_pa > 0.0) {
            return Err(InjectorError::InvalidInput {
                reason: "ΔP must be finite and > 0".to_owned(),
            });
        }
        Ok(self.q_ref_m3_s * (delta_p_pa / self.p_ref_pa).sqrt())
    }

    /// Deadtime (s) a tensión dada (interpola el mapa; satura en bordes).
    ///
    /// # Errors
    ///
    /// `InvalidInput` si V no es finita o ≤ 0.
    pub fn deadtime_at(&self, volts: f64) -> Result<f64, InjectorError> {
        if !(volts.is_finite() && volts > 0.0) {
            return Err(InjectorError::InvalidInput {
                reason: "voltage must be finite and > 0".to_owned(),
            });
        }
        const VS: [f64; 5] = [10.0, 12.0, 13.0, 14.0, 15.0];
        if volts <= VS[0] {
            return Ok(self.deadtime_s_at_v[0]);
        }
        if volts >= VS[4] {
            return Ok(self.deadtime_s_at_v[4]);
        }
        for w in VS.windows(2) {
            if volts <= w[1] {
                let i = VS.iter().position(|v| *v == w[0]).unwrap_or(0);
                let t = (volts - w[0]) / (w[1] - w[0]);
                let d = self.deadtime_s_at_v;
                return Ok(d[i] + t * (d[i + 1] - d[i]));
            }
        }
        Ok(self.deadtime_s_at_v[4])
    }

    /// Convierte masa comandada en pulso total (efectivo + deadtime).
    ///
    /// # Errors
    ///
    /// `InvalidInput` si masa < 0, ΔP ≤ 0 o V inválida.
    pub fn pulse_for_mass(
        &self,
        fuel: Fuel,
        mass_kg: f64,
        delta_p_pa: f64,
        volts: f64,
    ) -> Result<(PulseCommand, f64), InjectorError> {
        if !(mass_kg.is_finite() && mass_kg >= 0.0) {
            return Err(InjectorError::InvalidInput {
                reason: "mass must be finite and >= 0".to_owned(),
            });
        }
        let mdot = self.flow_at(delta_p_pa)? * fuel.density_kg_m3;
        let dt = self.deadtime_at(volts)?;
        let t_eff = mass_kg / mdot;
        Ok((
            PulseCommand {
                width_s: t_eff + dt,
            },
            t_eff,
        ))
    }

    /// Entrega real de un pulso comandado bajo condiciones dadas.
    /// `pump_capacity_kg_s = None` significa sin límite modelado.
    pub fn deliver(
        &self,
        fuel: Fuel,
        cmd: PulseCommand,
        delta_p_pa: f64,
        volts: f64,
        pump_capacity_kg_s: Option<f64>,
    ) -> (DeliveredMass, DeliveryStatus) {
        if delta_p_pa <= 0.0 || !delta_p_pa.is_finite() {
            return (
                DeliveredMass {
                    mass_kg: 0.0,
                    effective_s: 0.0,
                    deadtime_s: 0.0,
                    nonlinear_region: false,
                    saturated: false,
                },
                DeliveryStatus::NoPressure,
            );
        }
        let dt = self.deadtime_at(volts).unwrap_or(0.0);
        if cmd.width_s <= dt {
            return (
                DeliveredMass {
                    mass_kg: 0.0,
                    effective_s: 0.0,
                    deadtime_s: dt,
                    nonlinear_region: true,
                    saturated: false,
                },
                DeliveryStatus::BelowDeadtime,
            );
        }
        let t_eff = cmd.width_s - dt;
        let mdot = self.flow_at(delta_p_pa).unwrap_or(0.0) * fuel.density_kg_m3;
        let mut mass = t_eff * mdot;
        let mut saturated = false;
        if let Some(cap) = pump_capacity_kg_s {
            // La bomba limita el caudal MEDIO equivalente del evento: si la
            // demanda instantánea supera la capacidad, se recorta.
            if mdot > cap && cap >= 0.0 {
                mass = t_eff * cap;
                saturated = true;
            }
        }
        (
            DeliveredMass {
                mass_kg: mass,
                effective_s: t_eff,
                deadtime_s: dt,
                nonlinear_region: t_eff < 2.0e-3,
                saturated,
            },
            if saturated {
                DeliveryStatus::Saturated
            } else {
                DeliveryStatus::Delivered
            },
        )
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::rail::Fuel;

    fn inj() -> Injector {
        Injector::p2a_working_hypothesis()
    }

    #[test]
    fn idle_chain_matches_research_arithmetic() {
        // Contraste v0.1 §3.2: aire 0.113 g/cil/ciclo, AFR 14.7 ⇒ 7.69 mg;
        // con 185 cc/min ⇒ PW efectivo ≈ 3.37 ms (+deadtime ~0.62 @14V).
        let fuel = Fuel::illustrative_gasoline();
        let m = 0.113e-3 / 14.7;
        let (cmd, t_eff) = inj().pulse_for_mass(fuel, m, 300_000.0, 14.0).unwrap();
        assert!((t_eff * 1e3 - 3.37).abs() < 0.05, "t_eff={t_eff}");
        assert!((cmd.width_s * 1e3 - (3.37 + 0.62)).abs() < 0.05);
        // ±0.3 ms de deadtime ⇒ ±8% a ralentí (el MISSING-DATA más caro).
        assert!((0.3e-3 / cmd.width_s - 0.075).abs() < 0.01);
    }

    #[test]
    fn sqrt_law_and_voltage_sensitivity() {
        // Q(2.5 bar) = 185·√(2.5/3) ≈ 168.9 (ambigüedad ±6% del claim).
        let q = inj().flow_at(250_000.0).unwrap() * 60.0 * 1e6;
        assert!((q - 168.9).abs() < 0.5, "q={q}");
        // Caída a 12 V sin corrección ⇒ ~8% pobre a ralentí.
        let dt14 = inj().deadtime_at(14.0).unwrap();
        let dt12 = inj().deadtime_at(12.0).unwrap();
        assert!(dt12 > dt14);
        assert!(((dt12 - dt14) / 3.99e-3 - 0.078).abs() < 0.01);
    }

    #[test]
    fn commanded_vs_delivered_separated_with_failure_cases() {
        let fuel = Fuel::illustrative_gasoline();
        // Bomba apagada: 0 entregado, estado NoPressure (gate: sin DTC).
        let (_, st) = inj().deliver(fuel, PulseCommand { width_s: 4e-3 }, 0.0, 14.0, None);
        assert_eq!(st, DeliveryStatus::NoPressure);
        // Pulso bajo deadtime: no abre.
        let (d, st) = inj().deliver(
            fuel,
            PulseCommand { width_s: 0.3e-3 },
            300_000.0,
            14.0,
            None,
        );
        assert_eq!(st, DeliveryStatus::BelowDeadtime);
        assert_eq!(d.mass_kg, 0.0);
        // Capacidad insuficiente: recorte con flag, comandado intacto.
        let cmd = PulseCommand { width_s: 12e-3 };
        let (full, _) = inj().deliver(fuel, cmd, 300_000.0, 14.0, None);
        let (sat, st) = inj().deliver(fuel, cmd, 300_000.0, 14.0, Some(0.5e-3));
        assert_eq!(st, DeliveryStatus::Saturated);
        assert!(sat.mass_kg < full.mass_kg && sat.saturated);
        // Round-trip comandado→entregado sin saturación.
        let (cmd2, _) = inj()
            .pulse_for_mass(fuel, 7.69e-6, 300_000.0, 14.0)
            .unwrap();
        let (d2, st2) = inj().deliver(fuel, cmd2, 300_000.0, 14.0, None);
        assert_eq!(st2, DeliveryStatus::Delivered);
        assert!((d2.mass_kg - 7.69e-6).abs() < 1e-12);
        assert!(!d2.nonlinear_region);
    }
}
