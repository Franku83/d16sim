//! Eventos de servicio S26: reemplazar/medir/rectificar/bruñir/cambiar
//! aceite/recalibrar, con registro y provenance. Un servicio cambia el
//! slow-state por declaración explícita, nunca por decaimiento mágico.

use crate::state::WearState;
use serde::{Deserialize, Serialize};

/// Evento de servicio con provenance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceEvent {
    pub hours: f64,
    pub action: String,
    pub detail: String,
    pub operator: String,
}

/// Registro cronológico (solo crece; horas monótonas).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceLog {
    pub events: Vec<ServiceEvent>,
}

impl ServiceLog {
    #[must_use]
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Aplica el evento al estado + lo registra. Acciones conocidas:
    /// `rings_replace`, `hone`, `oil_change`, `retorque`, `bearings_replace`.
    /// Desconocida ⇒ rechaza (fail-closed, no-op silencioso jamás).
    ///
    /// # Errors
    ///
    /// `Invalid`/`Unknown` si la acción no existe o viola dominio.
    pub fn apply(
        &mut self,
        state: &mut WearState,
        action: &str,
        detail: &str,
        operator: &str,
    ) -> Result<(), String> {
        if operator.trim().is_empty() || action.trim().is_empty() {
            return Err("service needs action+operator".to_owned());
        }
        match action {
            "rings_replace" => state.ring_gap_m = [0.3e-3; 4],
            "hone" => state.liner_wear_m = [0.0; 4],
            "oil_change" => state.oil_oxidation_01 = 0.0,
            "retorque" => state.gasket_relax_01 = 0.0,
            "bearings_replace" => state.bearing_extra_m = [0.0; 9],
            _ => return Err(format!("unknown service action {action}")),
        }
        self.events.push(ServiceEvent {
            hours: state.hours,
            action: action.to_owned(),
            detail: detail.to_owned(),
            operator: operator.to_owned(),
        });
        Ok(())
    }
}

impl Default for ServiceLog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::laws::WearLaw;
    use crate::state::{OperatingPoint, WearState};

    #[test]
    fn service_resets_logged_parts_only() {
        let mut s = WearState::new_seeded(1);
        let op = OperatingPoint {
            rpm: 3000.0,
            load_01: 0.8,
            oil_temp_c: 110.0,
            coolant_temp_c: 90.0,
            film_0_1: 0.5,
            ring_pressure_pa: 5e6,
            bearing_pressure_pa: 5e6,
        };
        let laws = (
            WearLaw::archard_ring(),
            WearLaw::bearing_starvation(),
            WearLaw::oil_aging(),
        );
        s.advance(50.0, &op, &laws.0, &laws.1, &laws.2).unwrap();
        assert!(s.ring_gap_m[0] > 0.3e-3 && s.oil_oxidation_01 > 0.0);
        let mut log = ServiceLog::new();
        log.apply(&mut s, "rings_replace", "std set", "op").unwrap();
        log.apply(&mut s, "oil_change", "10W30", "op").unwrap();
        assert_eq!(s.ring_gap_m, [0.3e-3; 4]);
        assert_eq!(s.oil_oxidation_01, 0.0);
        // Lo no tocado persiste (desgaste de camisa y cojinetes).
        assert!(s.liner_wear_m[0] > 0.0 && s.bearing_extra_m[0] > 0.0);
        assert_eq!(log.events.len(), 2);
        assert!(log.apply(&mut s, "miracle_flush", "x", "op").is_err());
    }
}
