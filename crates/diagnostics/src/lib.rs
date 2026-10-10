//! Instrumentos de taller (S20): sondas, multímetro, osciloscopio,
//! scanner y manual indexado. Sin panel secreto: este crate NO depende de
//! `engine` ni `assembly` (poseedores del ground truth) — solo mide lo que
//! se le conecta. Sin ayudas automáticas: no existe función que devuelva
//! causas; los casos ambiguos se resuelven con mediciones complementarias.

pub mod ambiguity;
pub mod mech;
pub mod multimeter;
pub mod probes;
pub mod scan;
pub mod scope;

pub use ambiguity::{edge_interval_variance, AmbiguityCase, Verdict};
pub use mech::{
    cranking_peak_kpa, diagnose_low_compression, BoreGauge, CompressionGauge, LeakDownTester,
    LeakSplit, MechCause, MechVerdict, Provenance, Reading, C_CRANK_ILLUSTRATIVE,
};
pub use multimeter::{Multimeter, ResistanceError};
pub use probes::{Access, Probe, ProbePoint};
pub use scan::{ManualIndex, ManualRef, ScanTool};
pub use scope::{Oscilloscope, ScopeCapture};
