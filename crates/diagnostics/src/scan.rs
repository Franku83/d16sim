//! Scanner + manual (S20): solo lo que la ECU genera + índice de
//! referencias legales (enlaces, nunca contenido copiado).
//!
//! El scanner recibe lecturas y DTC explícitamente: no ve nada más por
//! construcción (sus campos son todo lo que contiene).

use ecu::{DtcSet, SyncState};

/// Herramienta de scan: espejo de lo entregado (sin sonda secreta).
#[derive(Debug, Clone, PartialEq)]
pub struct ScanTool {
    /// Lecturas vivas entregadas (nombre, valor, unidad).
    pub readings: Vec<(String, f64, String)>,
    /// DTC entregados por la ECU.
    pub dtcs: DtcSet,
    /// Estado de sinc entregado (si lo hay).
    pub sync: Option<SyncState>,
}

impl ScanTool {
    /// Construye SOLO con lo entregado (nada implícito).
    #[must_use]
    pub fn with_data(
        readings: Vec<(String, f64, String)>,
        dtcs: DtcSet,
        sync: Option<SyncState>,
    ) -> Self {
        Self {
            readings,
            dtcs,
            sync,
        }
    }

    /// Lectura por nombre (`None` si no fue entregada: el scanner no la
    /// inventa).
    #[must_use]
    pub fn reading(&self, name: &str) -> Option<(f64, String)> {
        self.readings
            .iter()
            .find(|(n, _, _)| n == name)
            .map(|(_, v, u)| (*v, u.clone()))
    }
}

/// Referencia al manual (enlace + sección, sin contenido copiado).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManualRef {
    /// p. ej. "CF01", "TRIG-2026-10-10", "R01".
    pub source_id: String,
    /// Título del procedimiento.
    pub title: String,
    /// URL pública de referencia.
    pub url: String,
}

/// Índice de servicio (curado, trazable a provenance).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ManualIndex {
    pub entries: Vec<ManualRef>,
}

impl ManualIndex {
    /// Índice mínimo S20 (enlaces del Foundation/provenance).
    #[must_use]
    pub fn builtin() -> Self {
        Self {
            entries: vec![
                ManualRef {
                    source_id: "CF01".to_owned(),
                    title: "Compression check: 1270/930/200 kPa @250rpm WOT".to_owned(),
                    url: "https://workshop-manuals.com/honda/civic_dx_hatchback/l4-1590cc_1.6l_sohc_mfi/engine_cooling_and_exhaust/engine/tune-up_and_engine_performance_checks/compression_check/system_information/specifications/".to_owned(),
                },
                ManualRef {
                    source_id: "CF07".to_owned(),
                    title: "Fuel pressure: vacuum on/off rail ranges".to_owned(),
                    url: "https://workshop-manuals.com/honda/civic_lx_sedan/l4-1590cc_1.6l_sohc_mfi/powertrain_management/fuel_delivery_and_air_induction/fuel_pressure/system_information/technical_service_bulletins/page_4115/".to_owned(),
                },
                ManualRef {
                    source_id: "TRIG-2026-10-10".to_owned(),
                    title: "Distributor trigger 24/4/1 family candidate (not OEM)".to_owned(),
                    url: "https://www.hondabase.com/cars/electronics/honda-dbh-distributor-trigger-decoding".to_owned(),
                },
                ManualRef {
                    source_id: "R01".to_owned(),
                    title: "Honda Civic 1996-2000 Service Manual index".to_owned(),
                    url: "https://www.manualslib.com/manual/464698/Honda-Civic.html".to_owned(),
                },
            ],
        }
    }

    /// Búsqueda por palabra en título o ID (el manual no diagnostica solo).
    #[must_use]
    pub fn lookup(&self, keyword: &str) -> Vec<&ManualRef> {
        let k = keyword.to_lowercase();
        self.entries
            .iter()
            .filter(|e| {
                e.title.to_lowercase().contains(&k) || e.source_id.to_lowercase().contains(&k)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn scanner_shows_only_what_it_was_given() {
        let s = ScanTool::with_data(
            vec![("rpm".to_owned(), 750.0, "rev/min".to_owned())],
            DtcSet::default(),
            Some(SyncState::Synced720),
        );
        assert_eq!(s.reading("rpm"), Some((750.0, "rev/min".to_owned())));
        assert_eq!(s.reading("oil_pressure"), None);
        assert!(s.dtcs.active.is_empty());
    }

    #[test]
    fn manual_index_resolves_links_not_content() {
        let m = ManualIndex::builtin();
        assert_eq!(m.lookup("compression").len(), 1);
        assert!(m
            .lookup("trigger")
            .iter()
            .any(|e| e.source_id.starts_with("TRIG")));
        assert!(m.lookup("gearbox").is_empty());
    }
}
