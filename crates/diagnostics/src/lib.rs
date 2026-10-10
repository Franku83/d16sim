//! Instrumentos de taller (S20): sondas, multímetro, osciloscopio,
//! scanner y manual indexado. Sin panel secreto: este crate NO depende de
//! `engine` ni `assembly` (poseedores del ground truth) — solo mide lo que
//! se le conecta. Sin ayudas automáticas: no existe función que devuelva
//! causas; los casos ambiguos se resuelven con mediciones complementarias.

pub mod ambiguity;
pub mod multimeter;
pub mod probes;
pub mod scan;
pub mod scope;

pub use ambiguity::{edge_interval_variance, AmbiguityCase, Verdict};
pub use multimeter::{Multimeter, ResistanceError};
pub use probes::{Access, Probe, ProbePoint};
pub use scan::{ManualIndex, ManualRef, ScanTool};
pub use scope::{Oscilloscope, ScopeCapture};
