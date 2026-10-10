//! Control open-loop declarado (S16): estima aire por speed-density con
//! ηv FIJO (supuesto de controlador, no física), dosifica por AFR objetivo
//! y dispara con avance FIJO. Lazo cerrado, aprendizaje de VE y mapas
//! reales son S18/S25. La ECU aquí es un marcador de posición honesto.

use fuel::{Fuel, Injector};

/// Controlador open-loop (placeholder S18).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OpenLoopControl {
    /// Rendimiento volumétrico supuesto por el controlador (NO el VE real
    /// del gas exchange: el error entre ambos es mezcla rica/pobre real).
    pub eta_v_assumed: f64,
    /// AFR objetivo.
    pub afr_target: f64,
    /// Avance de encendido (grados BTDC del PMS de encendido, fijo).
    pub spark_adv_btdc_deg: f64,
    /// ¿Chispa habilitada? (falla `no_spark`).
    pub spark_enabled: bool,
    /// ¿Inyección habilitada? (falla `no_fuel`).
    pub fuel_enabled: bool,
}

/// Comando de combustible para un ciclo de un cilindro.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FuelCommand {
    /// Masa a inyectar (kg, 0 si deshabilitado).
    pub mass_kg: f64,
}

impl OpenLoopControl {
    /// Masa de combustible desde aire ESTIMADO (speed-density con ηv fijo).
    /// `rho_manifold` sale del plenum real; el ηv fijo es el supuesto.
    #[must_use]
    pub fn fuel_command(
        &self,
        rho_manifold_kg_m3: f64,
        displaced_m3: f64,
        _fuel: Fuel,
        _injector: Injector,
    ) -> FuelCommand {
        if !self.fuel_enabled {
            return FuelCommand { mass_kg: 0.0 };
        }
        let m_air_est = self.eta_v_assumed * rho_manifold_kg_m3 * displaced_m3;
        FuelCommand {
            mass_kg: (m_air_est / self.afr_target).max(0.0),
        }
    }
}
