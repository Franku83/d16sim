//! Reluctores del distribuidor (S17): geometría de dientes y fase.
//!
//! El distribuidor gira a velocidad de árbol (mitad del cigüeñal): 24
//! dientes CKP ⇒ 12 pulsos/rev cigüeñal; 4 TDC ⇒ 2/rev; 1 CYP ⇒
//! 0.5/rev. Sin dientes faltantes (la sinc de 720° la da CYP).

use thiserror::Error;

/// Error de reluctor.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ReluctorError {
    /// Dientes/posición inválidos.
    #[error("invalid reluctor: {reason}")]
    Invalid { reason: String },
}

/// Rueda CKP genérica de 24 dientes (familia 95+, candidata).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CkpWheel {
    teeth: u32,
}

impl CkpWheel {
    /// 24 dientes (candidato familiar, no OEM).
    #[must_use]
    pub fn generic_24() -> Self {
        Self { teeth: 24 }
    }

    #[must_use]
    pub fn teeth(&self) -> u32 {
        self.teeth
    }

    /// Periodo en grados de DISTRIBUIDOR entre dientes.
    #[must_use]
    pub fn pitch_distributor_deg(&self) -> f64 {
        360.0 / f64::from(self.teeth)
    }
}

/// Rueda TDC genérica de 4 dientes (pares 1/4 y 2/3).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TdcWheel {
    teeth: u32,
}

impl TdcWheel {
    /// 4 dientes (candidato familiar, no OEM).
    #[must_use]
    pub fn generic_4() -> Self {
        Self { teeth: 4 }
    }

    #[must_use]
    pub fn teeth(&self) -> u32 {
        self.teeth
    }
}

/// Rueda CYP genérica de 1 diente (sinc secuencial del cilindro 1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CypWheel;

impl CypWheel {
    /// Fase del diente CYP en grados de ciclo global (ilustrativa:
    /// flanco en el PMS de encendido del cilindro 1 ± offset de montaje).
    #[must_use]
    pub fn tooth_global_deg(distributor_offset_crank_deg: f64) -> f64 {
        360.0 + distributor_offset_crank_deg
    }
}

/// Posiciones de dientes TDC en el ciclo global (ilustrativo: un diente
/// por PMS de encendido de cada cilindro: 0/180/360/540 + offset).
#[must_use]
pub fn tdc_teeth_global_deg(distributor_offset_crank_deg: f64) -> [f64; 4] {
    [
        0.0 + distributor_offset_crank_deg,
        180.0 + distributor_offset_crank_deg,
        360.0 + distributor_offset_crank_deg,
        540.0 + distributor_offset_crank_deg,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tooth_counts_per_cycle_match_family_candidate() {
        // 24/4/1 por 720° de cigüeñal (distribuidor a mitad de velocidad).
        assert_eq!(CkpWheel::generic_24().teeth(), 24);
        assert_eq!(TdcWheel::generic_4().teeth(), 4);
        assert!((CkpWheel::generic_24().pitch_distributor_deg() - 15.0).abs() < 1e-12);
    }
}
