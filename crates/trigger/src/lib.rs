//! Triggers de distribuidor (S17): hardware CKP/TDC/CYP + transductores.
//!
//! Patrón GENERIC_TRIGGER de familia (TRIG-2026-10-10): 24/4/1 dientes a
//! mitad de velocidad del cigüeñal. **Prohibido etiquetarlo OEM**: no
//! existe constructor ni variante OEM en este crate (la procedencia solo
//! admite `generic_family()`). CKF: concepto registrado, no modelado
//! (presencia PENDING en la variante).

pub mod channels;
pub mod reluctor;
pub mod vr;

pub use channels::{DistributorTrigger, TriggerConfig, TriggerFault};
pub use reluctor::{CkpWheel, CypWheel, TdcWheel};
pub use vr::{Conditioner, Polarity, VrSensor};

/// Procedencia del patrón: solo existe la familiar genérica. Un futuro
/// patrón OEM verificado añadirá otro constructor aquí con su fuente.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TriggerProvenance(&'static str);

impl TriggerProvenance {
    /// Candidato familiar 24/4/1 (hondabase + rusEFI, TD-80U/98U sin
    /// confirmar, 16° de era OBD1). NO es el patrón de la variante.
    #[must_use]
    pub fn generic_family() -> Self {
        Self("GENERIC_TRIGGER per TRIG-2026-10-10 (family candidate, not OEM)")
    }

    #[must_use]
    pub fn label(self) -> &'static str {
        self.0
    }
}
