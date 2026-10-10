//! Sondas y puntos de conexión (S20): acceso físico, polaridad y masa.
//!
//! Una sonda solo mide donde está conectada, con la polaridad y referencia
//! declaradas. Sin acceso físico no hay lectura (no existe sonda global).

use thiserror::Error;

/// Error de sonda.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ProbeError {
    /// Sin acceso físico al punto (conector sellado, pin inaccesible).
    #[error("no physical access to probe point: {reason}")]
    NoAccess { reason: String },
}

/// Acceso físico al punto de medida.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    /// Backprobing del conector (permitido, no invasivo).
    Backprobe,
    /// Pin inaccesible (conector sellado/cubierto).
    Sealed,
}

/// Punto de conexión identificado (señal + ubicación).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbePoint {
    /// p. ej. "MAP.sensor_out", "CKP.ecu_pin", "battery.plus".
    pub signal_id: String,
    /// p. ej. "sensor connector backside", "ECU pin A11 (generic ref)".
    pub location: String,
    pub access: Access,
}

/// Sonda con polaridad y referencia de masa declaradas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Probe {
    /// true = punta al positivo (invertida si false).
    pub positive_lead: bool,
    /// Referencia de masa (p. ej. "battery_neg", "ecu_ground").
    pub ground_ref: String,
}

impl Probe {
    /// Conecta y aplica polaridad a una lectura diferencial. Sin acceso no
    /// hay medida (el llamante debe comprobar `access` antes).
    ///
    /// # Errors
    ///
    /// `NoAccess` si el punto está sellado.
    pub fn measure(
        &self,
        point: &ProbePoint,
        v_signal: f64,
        v_ground: f64,
    ) -> Result<f64, ProbeError> {
        if point.access == Access::Sealed {
            return Err(ProbeError::NoAccess {
                reason: point.signal_id.clone(),
            });
        }
        let d = v_signal - v_ground;
        Ok(if self.positive_lead { d } else { -d })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn polarity_and_access_matter() {
        let p = Probe {
            positive_lead: true,
            ground_ref: "batt_neg".to_owned(),
        };
        let pt = ProbePoint {
            signal_id: "MAP.out".to_owned(),
            location: "backprobe".to_owned(),
            access: Access::Backprobe,
        };
        assert_eq!(p.measure(&pt, 2.0, 0.1).unwrap(), 1.9);
        let rev = Probe {
            positive_lead: false,
            ..p.clone()
        };
        assert_eq!(rev.measure(&pt, 2.0, 0.1).unwrap(), -1.9);
        let sealed = ProbePoint {
            access: Access::Sealed,
            ..pt
        };
        assert!(p.measure(&sealed, 2.0, 0.1).is_err());
    }
}
