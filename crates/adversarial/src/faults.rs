//! Fallos S29: transformaciones puras de `EngineInputs`, registradas por
//! nombre. Cada variante documenta su mapeo al par del Foundation.

/// Delta físico sobre las entradas (aplicación conmuta por construcción).
#[derive(Debug, Clone, PartialEq)]
pub enum Fault {
    /// Restricción de admisión (× área): proxy de baja compresión por
    /// masa admitida (el blow-by real espera acople S23–motor, deuda).
    RestrictIntake {
        factor: f64,
    },
    /// Carga extra al freno (N·m): banco/auxiliar agarrotado.
    ExtraLoad {
        tau_n_m: f64,
    },
    /// Retraso de encendido (grados): timing offset / correa saltada.
    RetardSpark {
        deg: f64,
    },
    /// AFR objetivo pobre: inyector sucio / comando erróneo.
    LeanAfr {
        afr: f64,
    },
    /// AFR objetivo rico: inyector abierto / regulador alto.
    RichAfr {
        afr: f64,
    },
    /// Tensión de carga baja (V): batería/cableado → misfire por energía.
    LowVoltage {
        volts: f64,
    },
    /// Fricción ×3 (motor frío/degradado, S16 high_friction).
    HighFriction,
    /// Refrigeración limitada (× ha_rad): termostato/radiador tapado.
    CoolingRestricted {
        factor: f64,
    },
    /// Sin chispa / sin combustible (extremos de diagnóstico).
    NoSpark,
    NoFuel,
}

impl Fault {
    /// Nombre estable para logs (la física nunca lo lee).
    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Fault::RestrictIntake { factor } => format!("restrict_intake×{factor}"),
            Fault::ExtraLoad { tau_n_m } => format!("extra_load+{tau_n_m}Nm"),
            Fault::RetardSpark { deg } => format!("retard-{deg}deg"),
            Fault::LeanAfr { afr } => format!("lean-afr{afr}"),
            Fault::RichAfr { afr } => format!("rich-afr{afr}"),
            Fault::LowVoltage { volts } => format!("low_voltage{volts}V"),
            Fault::HighFriction => "high_friction".to_owned(),
            Fault::CoolingRestricted { factor } => format!("cooling×{factor}"),
            Fault::NoSpark => "no_spark".to_owned(),
            Fault::NoFuel => "no_fuel".to_owned(),
        }
    }

    /// Aplica el delta (puro; el orden conmuta).
    pub fn apply(&self, inp: &mut engine::EngineInputs) {
        match *self {
            Fault::RestrictIntake { factor } => {
                inp.throttle_area_m2 *= factor;
                inp.idle_bypass_area_m2 *= factor;
            }
            Fault::ExtraLoad { tau_n_m } => inp.accessory_tau_n_m += tau_n_m,
            Fault::RetardSpark { deg } => inp.control.spark_adv_btdc_deg -= deg,
            Fault::LeanAfr { afr } | Fault::RichAfr { afr } => inp.control.afr_target = afr,
            Fault::LowVoltage { volts } => inp.charge_volts = volts,
            Fault::HighFriction => {
                inp.friction = engine::high_friction_inputs().friction;
            }
            Fault::CoolingRestricted { factor } => {
                let mut tp = inp
                    .thermal
                    .unwrap_or_else(thermal::ThermalParams::illustrative);
                tp.ha_rad_full *= factor;
                inp.thermal = Some(tp);
            }
            Fault::NoSpark => inp.control.spark_enabled = false,
            Fault::NoFuel => inp.control.fuel_enabled = false,
        }
    }
}
