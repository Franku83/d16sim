//! ECU de referencia P2E-like (S18): decoder, mapas propios, scheduler y
//! monitores. Reactiva a sensores; nunca atribuye mapas aftermarket ni
//! códigos OEM a Honda. El DTC nace de monitores temporales, jamás de un
//! `fault_id`.
//!
//! Contraste Speeduino (referencia conceptual, sin copiar GPL): Speeduino
//! decodifica ruedas con diente faltante por log de dientes y predice ángulo
//! por último intervalo; aquí la sinc de 720° la da CYP (24+1) con TDC de
//! respaldo (4+1), y la fase entre flancos por dead-reckoning con el rpm
//! estimado. Estrategia distinta sobre hardware distinto.

pub mod decoder;
pub mod dtc;
pub mod maps;
pub mod scheduler;

pub use decoder::{Decoder, SyncState};
pub use dtc::{CkpLossMonitor, CypLossMonitor, Dtc, DtcSet, MonitorEvent};
pub use maps::{BilinearGrid, ReferenceMaps};
pub use scheduler::{ActuationPlan, Ecu};
