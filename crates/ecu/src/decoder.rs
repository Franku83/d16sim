//! Decoder de flancos (S18): RPM por intervalos CKP, sinc 360°/720° por
//! CYP (24+1) con respaldo TDC (4+1), fase por dead-reckoning entre flancos.
//!
//! La pérdida de sincronía nace de timeout (3× periodo esperado sin flanco)
//! o de CYP contradictorio; nunca de una etiqueta de fallo.

use thiserror::Error;

/// Error del decoder.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum DecoderError {
    /// Flancos no monótonos o vacíos donde se exigían.
    #[error("invalid edge input: {reason}")]
    InvalidEdges { reason: String },
}

/// Estado de sincronización (FSM de sinc, parte de la FSM de ECU).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncState {
    /// Sin flancos aún.
    Unsynced,
    /// CKP visto, fase de 360° (sin CYP: chispa desperdiciada / batch).
    Synced360,
    /// CYP visto: fase de 720° (secuencial).
    Synced720,
    /// Timeout o contradicción: se perdió la sinc.
    SyncLost,
}

/// Estimación de régimen + fase.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SyncEstimate {
    pub state: SyncState,
    /// RPM estimadas (mediana de los últimos intervalos CKP).
    pub rpm: f64,
    /// Confianza [0, 1] (fracción de intervalos coherentes recientes).
    pub confidence: f64,
    /// Ángulo de ciclo estimado (° 0–720) en `at_time_s`.
    pub cycle_deg_at: f64,
    pub at_time_s: f64,
}

/// Decoder con memoria de flancos (24 CKP por ciclo genérico).
#[derive(Debug, Clone)]
pub struct Decoder {
    ckp_edges: Vec<f64>,
    cyp_edges: Vec<f64>,
    state: SyncState,
    last_time_s: f64,
}

impl Decoder {
    /// Crea con los flancos observados hasta `now_s` (listas crecientes).
    ///
    /// # Errors
    ///
    /// `InvalidEdges` si algún flanco no es finito, si una lista no es
    /// estrictamente creciente, o si se declara un flanco futuro (> `now_s`).
    pub fn new(ckp_edges: Vec<f64>, cyp_edges: Vec<f64>, now_s: f64) -> Result<Self, DecoderError> {
        if !now_s.is_finite() {
            return Err(DecoderError::InvalidEdges {
                reason: "now must be finite".to_owned(),
            });
        }
        for (name, edges) in [("ckp", &ckp_edges), ("cyp", &cyp_edges)] {
            for t in edges {
                if !t.is_finite() {
                    return Err(DecoderError::InvalidEdges {
                        reason: format!("{name} edge not finite"),
                    });
                }
                // Causalidad temporal: no se aceptan flancos futuros.
                if *t > now_s {
                    return Err(DecoderError::InvalidEdges {
                        reason: format!("{name} edge in the future"),
                    });
                }
            }
        }
        for w in ckp_edges.windows(2) {
            if w[1] <= w[0] {
                return Err(DecoderError::InvalidEdges {
                    reason: "ckp edges must be strictly increasing".to_owned(),
                });
            }
        }
        for w in cyp_edges.windows(2) {
            if w[1] <= w[0] {
                return Err(DecoderError::InvalidEdges {
                    reason: "cyp edges must be strictly increasing".to_owned(),
                });
            }
        }
        let mut d = Self {
            ckp_edges,
            cyp_edges,
            state: SyncState::Unsynced,
            last_time_s: now_s,
        };
        d.update(now_s);
        Ok(d)
    }

    /// Actualiza el estado a `now_s` (timeout = 3× periodo esperado).
    ///
    /// R02-C1/RV07: una observación no finita no corrompe el estado; se
    /// declara pérdida de sincronía sin tocar `last_time_s`.
    pub fn update(&mut self, now_s: f64) {
        if !now_s.is_finite() {
            self.state = SyncState::SyncLost;
            return;
        }
        self.last_time_s = now_s;
        let rpm = self.rpm_estimate().unwrap_or(0.0);
        let expected_period = if rpm > 1.0 {
            // Periodo entre flancos CKP: 30° de cigüeñal.
            30.0 / 360.0 / (rpm / 60.0)
        } else {
            f64::INFINITY
        };
        let last_ckp = self.ckp_edges.last().copied().unwrap_or(f64::NEG_INFINITY);
        let gap = now_s - last_ckp;
        // CYP visto alguna vez ⇒ 720°; si no, CKP solo ⇒ 360°.
        let has_bank = self.cyp_edges.iter().any(|t| *t <= now_s);
        if self.ckp_edges.is_empty() {
            self.state = SyncState::Unsynced;
        } else if gap > 3.0 * expected_period {
            self.state = SyncState::SyncLost;
        } else if has_bank {
            self.state = SyncState::Synced720;
        } else {
            self.state = SyncState::Synced360;
        }
        // CYP contradictorio: dos CYP sin ~24 CKP entre ellos ⇒ pérdida.
        if self.state == SyncState::Synced720 {
            let recent_cyp: Vec<f64> = self
                .cyp_edges
                .iter()
                .copied()
                .filter(|t| *t <= now_s)
                .collect();
            if recent_cyp.len() >= 2 {
                let last_two = &recent_cyp[recent_cyp.len() - 2..];
                let ckp_between = self
                    .ckp_edges
                    .iter()
                    .filter(|t| **t > last_two[0] && **t <= last_two[1])
                    .count();
                if ckp_between > 30 || ckp_between < 18 {
                    self.state = SyncState::SyncLost;
                }
            }
        }
    }

    /// RPM por mediana de los últimos 8 intervalos (robusta a un borde
    /// saltado aislado en amplitud, no en fase: documentado).
    #[must_use]
    pub fn rpm_estimate(&self) -> Option<f64> {
        if self.ckp_edges.len() < 2 {
            return None;
        }
        let n = self.ckp_edges.len();
        let from = n.saturating_sub(9);
        let mut dts: Vec<f64> = self.ckp_edges[from..]
            .windows(2)
            .map(|w| w[1] - w[0])
            .filter(|d| d.is_finite() && *d > 0.0)
            .collect();
        if dts.is_empty() {
            return None;
        }
        dts.sort_by(|a, b| a.total_cmp(b));
        let med = dts[dts.len() / 2];
        // 30° por flanco CKP: rpm = (30/360)/dt × 60.
        Some(30.0 / 360.0 / med * 60.0)
    }

    /// Confianza: fracción de los últimos 8 intervalos dentro de ±25% de
    /// la mediana (un borde saltado la degrada causalmente).
    #[must_use]
    pub fn confidence(&self) -> f64 {
        if self.ckp_edges.len() < 3 {
            return 0.0;
        }
        let n = self.ckp_edges.len();
        let from = n.saturating_sub(9);
        let dts: Vec<f64> = self.ckp_edges[from..]
            .windows(2)
            .map(|w| w[1] - w[0])
            .collect();
        let mut sorted = dts.clone();
        sorted.sort_by(|a, b| a.total_cmp(b));
        let med = sorted[sorted.len() / 2];
        if !(med.is_finite() && med > 0.0) {
            return 0.0;
        }
        let ok = dts
            .iter()
            .filter(|d| (**d - med).abs() <= 0.25 * med)
            .count();
        ok as f64 / dts.len() as f64
    }

    /// Estima fase de ciclo a `now_s` por dead-reckoning desde el último
    /// flanco CKP (fase 0 = flanco CKP n.º 0 del ciclo genérico).
    ///
    /// R02-C1/RV07: entradas no finitas devuelven una estima conservadora
    /// (sin sincronía, fase 0, tiempo finito), nunca NaN/Inf propagado.
    #[must_use]
    pub fn estimate(&self, now_s: f64) -> SyncEstimate {
        let rpm = self.rpm_estimate().filter(|v| v.is_finite()).unwrap_or(0.0);
        if !now_s.is_finite() {
            return SyncEstimate {
                state: SyncState::SyncLost,
                rpm,
                confidence: 0.0,
                cycle_deg_at: 0.0,
                at_time_s: if self.last_time_s.is_finite() {
                    self.last_time_s
                } else {
                    0.0
                },
            };
        }
        let cycle_deg_at = if let Some(&last) = self.ckp_edges.iter().rev().find(|t| **t <= now_s) {
            // Índice del último flanco ⇒ fase base + avance por tiempo.
            let idx = self.ckp_edges.iter().position(|t| *t == last).unwrap_or(0);
            let base = f64::from(idx as u32 % 24) * 30.0;
            let phase = (base + (now_s - last) * rpm / 60.0 * 360.0) % 720.0;
            if phase.is_finite() {
                phase
            } else {
                0.0
            }
        } else {
            0.0
        };
        SyncEstimate {
            state: self.state,
            rpm,
            confidence: self.confidence(),
            cycle_deg_at,
            at_time_s: now_s,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    /// Flancos sintéticos 24+1 a rpm constante (periodo exacto).
    fn edges_24p1(rpm: f64, cycles: usize) -> (Vec<f64>, Vec<f64>) {
        let per_flank = 30.0 / 360.0 / (rpm / 60.0);
        let ckp: Vec<f64> = (0..24 * cycles)
            .map(|i| f64::from(i as u32) * per_flank)
            .collect();
        let cyp: Vec<f64> = (0..cycles)
            .map(|i| f64::from(i as u32) * 24.0 * per_flank)
            .collect();
        (ckp, cyp)
    }

    #[test]
    fn acquires_720_sync_and_rpm() {
        let (ckp, cyp) = edges_24p1(750.0, 3);
        let now = *ckp.last().unwrap();
        let d = Decoder::new(ckp, cyp, now).unwrap();
        let est = d.estimate(now);
        assert_eq!(est.state, SyncState::Synced720);
        assert!((est.rpm - 750.0).abs() < 1e-9);
        assert!((est.confidence - 1.0).abs() < 1e-12);
    }

    #[test]
    fn missing_edge_degrades_confidence_causally() {
        let (mut ckp, cyp) = edges_24p1(750.0, 3);
        // Borde saltado DENTRO de la ventana reciente (la mediana mira atrás).
        let n = ckp.len();
        ckp.remove(n - 5);
        let now = *ckp.last().unwrap();
        let d = Decoder::new(ckp, cyp, now).unwrap();
        // La mediana resiste un borde aislado en amplitud...
        assert!((d.rpm_estimate().unwrap() - 750.0).abs() < 100.0);
        // ...pero la confianza cae y la fase deriva (causal, no etiqueta).
        assert!(d.confidence() < 1.0);
    }

    #[test]
    fn timeout_loses_sync_and_cyp_gives_360_fallback() {
        let (ckp, _) = edges_24p1(750.0, 1);
        let now = ckp.last().unwrap() + 1.0;
        let mut d = Decoder::new(ckp, Vec::new(), now).unwrap();
        d.update(now);
        // Sin CYP: 360° si hay flancos recientes; con 1 s de hueco: SyncLost.
        assert_eq!(d.estimate(now).state, SyncState::SyncLost);
        let (ckp2, _) = edges_24p1(750.0, 3);
        let now2 = ckp2[10];
        let d2 = Decoder::new(ckp2[..11].to_vec(), Vec::new(), now2).unwrap();
        assert_eq!(d2.estimate(now2).state, SyncState::Synced360);
    }

    #[test]
    fn rejects_non_finite_and_future_edges() {
        // R02-F15: NaN/inf y flancos futuros no son observaciones físicas.
        assert!(Decoder::new(vec![0.0, f64::NAN], Vec::new(), 1.0).is_err());
        assert!(Decoder::new(vec![0.0, f64::INFINITY], Vec::new(), 1.0).is_err());
        assert!(Decoder::new(vec![1.0, 2.0], Vec::new(), 0.0).is_err());
        assert!(Decoder::new(Vec::new(), vec![1.0], 0.0).is_err());
        assert!(Decoder::new(vec![0.0, 1.0], Vec::new(), 1.0).is_ok());
    }

    #[test]
    fn nonfinite_observations_do_not_propagate() {
        // R02-C1/RV07: update/estimate con NaN/Inf no propagan tiempo/fase
        // no finitos ni corrompen el estado.
        let mut d = Decoder::new(vec![0.0, 0.01, 0.02, 0.03], vec![], 0.03).unwrap();
        d.update(f64::NAN);
        let est = d.estimate(f64::INFINITY);
        assert!(est.at_time_s.is_finite());
        assert!(est.cycle_deg_at.is_finite());
        assert!(est.rpm.is_finite());
        assert_eq!(est.state, SyncState::SyncLost);
    }
}
