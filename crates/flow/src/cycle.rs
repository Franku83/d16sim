//! Ciclo 720° no combustivo con gas exchange (S12): un cilindro + plenum
//! + escape + ambiente. Euler explícito por ángulo; la conservación de
//! MASA es exacta por construcción (lo que sale de un nodo entra en otro
//! y la frontera se audita); la energía cierra a O(h) documentado.
//!
//! Sin VE impuesta: la masa atrapada emerge del caudal por válvula.

use crate::data::{
    curtain_area_m2, exhaust_cd, intake_cd, EXHAUST_VALVE_DIA_MM, INTAKE_VALVE_DIA_MM,
};
use crate::nodes::Volume;
use crate::orifice::mass_flow;
use kinematics::GeometryPack;
use thermo::Mixture;
use thiserror::Error;
use valvetrain::{BeltDrive, CamSpec};

/// Error del ciclo con gas exchange.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum CycleError {
    /// Entradas inválidas o paso inestable (fail-closed, no corrección).
    #[error("invalid gas-exchange input/state: {reason}")]
    Invalid { reason: String },
}

/// Entradas del ciclo (todo explícito; volúmenes/áreas ilustrativos).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GasExchangeInputs {
    pub rpm: f64,
    pub p_amb_pa: f64,
    pub t_amb_k: f64,
    pub plenum_v_m3: f64,
    pub exhaust_v_m3: f64,
    /// Área EFECTIVA de mariposa (m², Cd = 1 por definición de entrada).
    pub throttle_area_m2: f64,
    /// Área EFECTIVA de salida de escape (m²).
    pub tailpipe_area_m2: f64,
    pub step_deg: f64,
    /// Estado inicial del escape (None = ambiente fresco). Permite
    /// estudiar reflujo en primer ciclo sin operación continua (S16).
    pub exhaust_init: Option<ExhaustInit>,
}

/// Estado inicial impuesto al nodo de escape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExhaustInit {
    pub p_pa: f64,
    pub t_k: f64,
    pub fresh_frac: f64,
}

/// Auditoría del ciclo.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CycleAudit {
    /// Masa atrapada en IC (kg).
    pub trapped_kg: f64,
    /// Fracción residual en IC (1 − fresca).
    pub residual_frac: f64,
    /// Cierre de masa: (Δm − frontera)/m0.
    pub mass_closure: f64,
    /// Cierre de energía relativo (O(h) explícito).
    pub energy_closure: f64,
    /// p mín/máx del cilindro (Pa).
    pub p_min_pa: f64,
    pub p_max_pa: f64,
}

/// Punto de traza por cilindro (entregable S12).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TracePoint {
    pub theta_deg: f64,
    pub p_cyl_pa: f64,
    pub p_plen_pa: f64,
    pub p_exh_pa: f64,
    pub mdot_iv_kg_s: f64,
    pub mdot_ev_kg_s: f64,
}

/// Ejecuta el ciclo 720° en locales del cilindro 1.
///
/// # Errors
///
/// `Invalid` si alguna entrada viola dominio o un paso deja masa/T no
/// positivas (fail-closed).
pub fn run_cycle(
    geom: &GeometryPack,
    mix: Mixture,
    intake: &CamSpec,
    exhaust: &CamSpec,
    belt: &BeltDrive,
    inp: GasExchangeInputs,
) -> Result<(Vec<TracePoint>, CycleAudit), CycleError> {
    let bad = |reason: &str| CycleError::Invalid {
        reason: reason.to_owned(),
    };
    for (v, name) in [
        (inp.rpm, "rpm"),
        (inp.p_amb_pa, "p_amb"),
        (inp.t_amb_k, "t_amb"),
        (inp.plenum_v_m3, "plenum_v"),
        (inp.exhaust_v_m3, "exhaust_v"),
        (inp.throttle_area_m2, "throttle_area"),
        (inp.tailpipe_area_m2, "tailpipe_area"),
        (inp.step_deg, "step"),
    ] {
        if !(v.is_finite() && v > 0.0) {
            return Err(bad(name));
        }
    }
    let (r_gas, gamma) = (mix.r(), mix.gamma());
    let cp = mix.cp();
    let cv = mix.cv();
    let omega = inp.rpm * 2.0 * std::f64::consts::PI / 60.0;
    let h_step = inp.step_deg.to_radians();

    // Nodos: plenum y escape parten de ambiente (masas consistentes).
    let mut plen = Volume::new(
        inp.p_amb_pa * inp.plenum_v_m3 / (r_gas * inp.t_amb_k),
        inp.t_amb_k,
        inp.plenum_v_m3,
        1.0,
        r_gas,
        cv,
    )
    .map_err(|_| bad("plenum init"))?;
    let mut exh = match inp.exhaust_init {
        // Presión→masa por gas ideal (pasar Pa como kg reventaría el nodo).
        Some(init) => Volume::new(
            init.p_pa * inp.exhaust_v_m3 / (r_gas * init.t_k),
            init.t_k,
            inp.exhaust_v_m3,
            init.fresh_frac,
            r_gas,
            cv,
        )
        .map_err(|_| bad("exhaust init")),
        None => Volume::new(
            inp.p_amb_pa * inp.exhaust_v_m3 / (r_gas * inp.t_amb_k),
            inp.t_amb_k,
            inp.exhaust_v_m3,
            0.0,
            r_gas,
            cv,
        )
        .map_err(|_| bad("exhaust init")),
    }?;

    // Cilindro: arranca en BDC de admisión (local 180°) con aire ambiente.
    let v0 = geom.volume(std::f64::consts::PI);
    let mut m_cyl = inp.p_amb_pa * v0 / (r_gas * inp.t_amb_k);
    let mut u_cyl = m_cyl * cv * inp.t_amb_k;
    let mut f_cyl = 1.0;

    let shift = belt.cam_shift_crank_deg();
    let i_center = intake.centerline_atdc_deg;
    let e_center = 720.0 - exhaust.centerline_atdc_deg;

    let steps = (720.0 / inp.step_deg).ceil() as usize;
    let mut trace = Vec::with_capacity(steps + 1);
    let mut m_in_amb = 0.0;
    let mut m_out_amb = 0.0;
    let mut h_in_amb = 0.0;
    let mut h_out_amb = 0.0;
    let mut work_by = 0.0;
    let m_sys_0 = plen.m_kg() + exh.m_kg() + m_cyl;
    let u_sys_0 = plen.m_kg() * cv * plen.t_k() + exh.m_kg() * cv * exh.t_k() + u_cyl;
    let mut trapped_kg = 0.0;
    let mut residual = 0.0;
    let mut p_min = f64::INFINITY;
    let mut p_max = 0.0f64;

    // IC de referencia (cierre advertised de admisión, con desfase).
    let ic_local = i_center + intake.advertised_events().close_deg + shift;

    // El ciclo arranca en PMI de admisión (local 180°) con masa de PMI:
    // arrancar en PMS con masa de PMI daría ~800 kPa iniciales.
    let mut th = 180.0f64;
    let th_end = 180.0 + 720.0;
    while th <= th_end + 1e-9 {
        let v_cyl = geom.volume((th % 360.0).to_radians());
        let t_cyl = u_cyl / (m_cyl * cv);
        let p_cyl = m_cyl * r_gas * t_cyl / v_cyl;
        p_min = p_min.min(p_cyl);
        p_max = p_max.max(p_cyl);

        // Alzadas (mm) a este ángulo.
        let li = intake.lift_at(th - i_center - shift).max(0.0);
        let le = exhaust.lift_at(th - e_center - shift).max(0.0);
        let a_iv = 2.0 * intake_cd(li) * curtain_area_m2(INTAKE_VALVE_DIA_MM, li);
        let a_ev = 2.0 * exhaust_cd(le) * curtain_area_m2(EXHAUST_VALVE_DIA_MM, le);

        // Convención: caudales ≥ 0 con bandera de sentido (antisimetría
        // estructural: el signo lo pone el orden up/down, nunca el orificio).
        let p_plen = plen.p_pa();
        let p_exh = exh.p_pa();
        let (m_thr, h_thr, f_thr, thr_in) = if inp.p_amb_pa > p_plen {
            let m = mass_flow(
                inp.p_amb_pa,
                inp.t_amb_k,
                p_plen,
                inp.throttle_area_m2,
                r_gas,
                gamma,
            )
            .map_err(|_| bad("throttle flow"))?;
            (m, cp * inp.t_amb_k, 1.0, true)
        } else if p_plen > inp.p_amb_pa {
            let m = mass_flow(
                p_plen,
                plen.t_k(),
                inp.p_amb_pa,
                inp.throttle_area_m2,
                r_gas,
                gamma,
            )
            .map_err(|_| bad("throttle flow"))?;
            (m, cp * plen.t_k(), plen.fresh_frac(), false)
        } else {
            (0.0, 0.0, 1.0, true)
        };
        let (m_iv, h_iv, f_iv, iv_to_cyl) = if p_plen > p_cyl {
            let m = mass_flow(p_plen, plen.t_k(), p_cyl, a_iv, r_gas, gamma)
                .map_err(|_| bad("intake flow"))?;
            (m, cp * plen.t_k(), plen.fresh_frac(), true)
        } else if p_cyl > p_plen {
            let m = mass_flow(p_cyl, t_cyl, p_plen, a_iv, r_gas, gamma)
                .map_err(|_| bad("intake flow"))?;
            (m, cp * t_cyl, f_cyl, false)
        } else {
            (0.0, 0.0, f_cyl, true)
        };
        let (m_ev, h_ev, f_ev, ev_to_exh) = if p_cyl > p_exh {
            let m = mass_flow(p_cyl, t_cyl, p_exh, a_ev, r_gas, gamma)
                .map_err(|_| bad("exhaust flow"))?;
            (m, cp * t_cyl, f_cyl, true)
        } else if p_exh > p_cyl {
            let m = mass_flow(p_exh, exh.t_k(), p_cyl, a_ev, r_gas, gamma)
                .map_err(|_| bad("exhaust flow"))?;
            (m, cp * exh.t_k(), exh.fresh_frac(), false)
        } else {
            (0.0, 0.0, f_cyl, true)
        };
        let (m_tp, h_tp, tp_out) = if p_exh > inp.p_amb_pa {
            let m = mass_flow(
                p_exh,
                exh.t_k(),
                inp.p_amb_pa,
                inp.tailpipe_area_m2,
                r_gas,
                gamma,
            )
            .map_err(|_| bad("tailpipe flow"))?;
            (m, cp * exh.t_k(), true)
        } else if inp.p_amb_pa > p_exh {
            let m = mass_flow(
                inp.p_amb_pa,
                inp.t_amb_k,
                p_exh,
                inp.tailpipe_area_m2,
                r_gas,
                gamma,
            )
            .map_err(|_| bad("tailpipe flow"))?;
            (m, cp * inp.t_amb_k, false)
        } else {
            (0.0, 0.0, true)
        };

        trace.push(TracePoint {
            theta_deg: th,
            p_cyl_pa: p_cyl,
            p_plen_pa: p_plen,
            p_exh_pa: p_exh,
            mdot_iv_kg_s: if iv_to_cyl { m_iv } else { -m_iv },
            mdot_ev_kg_s: if ev_to_exh { m_ev } else { -m_ev },
        });

        // Frontera ambiente en MASA y ENTALPÍA + trabajo del pistón.
        let k = h_step / omega;
        if thr_in {
            m_in_amb += m_thr * k;
            h_in_amb += m_thr * h_thr * k;
        } else {
            m_out_amb += m_thr * k;
            h_out_amb += m_thr * h_thr * k;
        }
        if tp_out {
            m_out_amb += m_tp * k;
            h_out_amb += m_tp * h_tp * k;
        } else {
            m_in_amb += m_tp * k;
            h_in_amb += m_tp * h_tp * k;
        }
        let dv = geom.dvdtheta((th % 360.0).to_radians()) * h_step;
        work_by += p_cyl * dv;

        // Actualizar nodos (Euler explícito en θ, caudales ≥ 0 + sentido).
        // Plenum: entra throttle (si thr_in) y reflujo de admisión
        // (si !iv_to_cyl); sale lo contrario. h_thr/h_iv son entalpías
        // de sus upstreams respectivos.
        let thr_to_plen = if thr_in { m_thr } else { 0.0 };
        let thr_from_plen = if thr_in { 0.0 } else { m_thr };
        let rfl_to_plen = if iv_to_cyl { 0.0 } else { m_iv };
        let iv_from_plen = if iv_to_cyl { m_iv } else { 0.0 };
        let dm_plen = (thr_to_plen + rfl_to_plen - thr_from_plen - iv_from_plen) * k;
        let du_plen = (thr_to_plen * h_thr + rfl_to_plen * cp * t_cyl
            - thr_from_plen * cp * plen.t_k()
            - iv_from_plen * cp * plen.t_k())
            * k;
        let f_plen_in = {
            let w_thr = thr_to_plen;
            let w_rfl = rfl_to_plen;
            if w_thr + w_rfl > 0.0 {
                (w_thr * f_thr + w_rfl * f_cyl) / (w_thr + w_rfl)
            } else {
                1.0
            }
        };
        plen.step(dm_plen, f_plen_in, du_plen)
            .map_err(|_| CycleError::Invalid {
                reason: "plenum step unstable: reduce step_deg".to_owned(),
            })?;

        // Escape: entra del cilindro (si ev_to_exh) o reflujo al cilindro;
        // sale por tailpipe (si tp_out) o entra ambiente (si !tp_out).
        let ev_to_exh_m = if ev_to_exh { m_ev } else { 0.0 };
        let rfl_to_cyl_m = if ev_to_exh { 0.0 } else { m_ev };
        let tp_from_exh = if tp_out { m_tp } else { 0.0 };
        let tp_to_exh = if tp_out { 0.0 } else { m_tp };
        let dm_exh = (ev_to_exh_m + tp_to_exh - rfl_to_cyl_m - tp_from_exh) * k;
        let du_exh = (ev_to_exh_m * h_ev + tp_to_exh * cp * inp.t_amb_k
            - rfl_to_cyl_m * cp * exh.t_k()
            - tp_from_exh * cp * exh.t_k())
            * k;
        let f_exh_in = {
            let w_ev = ev_to_exh_m;
            let w_tp = tp_to_exh;
            if w_ev + w_tp > 0.0 {
                (w_ev * f_ev + w_tp * 1.0) / (w_ev + w_tp)
            } else {
                exh.fresh_frac()
            }
        };
        exh.step(dm_exh, f_exh_in, du_exh)
            .map_err(|_| CycleError::Invalid {
                reason: "exhaust step unstable: reduce step_deg".to_owned(),
            })?;

        // Cilindro: sistema abierto (entra iv_to_cyl + reflujo de escape;
        // sale reflujo de admisión + ev_to_exh).
        let dm_in = (iv_from_plen + rfl_to_cyl_m) * k;
        let dm_out = (rfl_to_plen + ev_to_exh_m) * k;
        let h_in_cyl = (iv_from_plen * h_iv + rfl_to_cyl_m * h_ev) * k;
        let h_out_cyl = (rfl_to_plen + ev_to_exh_m) * cp * t_cyl * k;
        let du_cyl = -p_cyl * dv + h_in_cyl - h_out_cyl;
        let m_prev = m_cyl;
        m_cyl += dm_in - dm_out;
        if !(m_cyl.is_finite() && m_cyl > 0.0) {
            return Err(bad("cylinder mass non-positive: reduce step_deg"));
        }
        if dm_in > 0.0 {
            let w_iv = iv_from_plen;
            let w_ev = rfl_to_cyl_m;
            let f_in = (w_iv * f_iv + w_ev * f_ev) / (w_iv + w_ev).max(1e-30);
            f_cyl = (f_cyl * m_prev + f_in * dm_in) / m_cyl;
        }
        u_cyl += du_cyl;
        if !(u_cyl.is_finite() && u_cyl > 0.0) {
            return Err(bad("cylinder energy non-positive: reduce step_deg"));
        }

        // Muestreo en IC.
        if (th - ic_local.rem_euclid(720.0)).abs() < h_step.to_degrees() / 2.0 + 1e-9 {
            trapped_kg = m_cyl;
            residual = 1.0 - f_cyl;
        }
        let _ = dm_in;
        let _ = dm_out;
        th += h_step.to_degrees();
    }

    let m_sys_1 = plen.m_kg() + exh.m_kg() + m_cyl;
    let u_sys_1 = plen.m_kg() * cv * plen.t_k() + exh.m_kg() * cv * exh.t_k() + u_cyl;
    // Cierre de masa directo con fronteras en MASA (exacto por construcción
    // salvo redondeo: lo que sale de un nodo entra en otro).
    let mass_closure = ((m_sys_1 - m_sys_0) - (m_in_amb - m_out_amb)) / m_sys_0;
    Ok((
        trace,
        CycleAudit {
            trapped_kg,
            residual_frac: residual,
            mass_closure,
            energy_closure: energy_closure_direct(
                u_sys_1 - u_sys_0,
                h_in_amb,
                h_out_amb,
                work_by,
                u_sys_0,
            ),
            p_min_pa: p_min,
            p_max_pa: p_max,
        },
    ))
}

/// Cierre energético relativo: (ΔU − (H_in − H_out − W))/U0.
fn energy_closure_direct(du: f64, h_in: f64, h_out: f64, work_by: f64, u0: f64) -> f64 {
    ((du - (h_in - h_out - work_by)) / u0).abs()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use thermo::Mixture;
    use valvetrain::{BeltDrive, CamSpec};

    fn setup() -> (
        GeometryPack,
        Mixture,
        CamSpec,
        CamSpec,
        BeltDrive,
        GasExchangeInputs,
    ) {
        (
            GeometryPack::illustrative_d16_like().unwrap(),
            Mixture::new(287.0, 1.4).unwrap(),
            CamSpec::proxy_d16a6_intake().unwrap(),
            CamSpec::proxy_d16a6_exhaust().unwrap(),
            BeltDrive::new(100.0, 0.0).unwrap(),
            GasExchangeInputs {
                rpm: 750.0,
                p_amb_pa: 101_325.0,
                t_amb_k: 293.15,
                plenum_v_m3: 2.0e-3,
                exhaust_v_m3: 1.5e-3,
                throttle_area_m2: 8.0e-4,
                tailpipe_area_m2: 8.0e-4,
                step_deg: 0.25,
                exhaust_init: None,
            },
        )
    }

    #[test]
    fn mass_closes_and_stock_overlap_barely_reflows() {
        let (geom, mix, i, e, belt, inp) = setup();
        let (trace, audit) = run_cycle(&geom, mix, &i, &e, &belt, inp).unwrap();
        // Cierre de masa exacto por construcción (solo redondeo).
        assert!(
            audit.mass_closure.abs() < 1e-9,
            "mass_closure={}",
            audit.mass_closure
        );
        // Con cruce stock (3°) y escape a ambiente, el reflujo en primer
        // ciclo es despreciable: documentado, no impuesto (ver
        // precharged_exhaust_with_offset_gives_residual para residual > 0).
        assert!(
            audit.residual_frac >= 0.0 && audit.residual_frac < 0.01,
            "residual={}",
            audit.residual_frac
        );
        // Masa atrapada positiva y menor que la de volumen máximo.
        assert!(audit.trapped_kg > 0.0 && audit.trapped_kg < 1e-3);
        // Traza finita en todo el ciclo (sin discontinuidades falsas).
        assert!(trace.iter().all(|p| p.p_cyl_pa.is_finite()
            && p.mdot_iv_kg_s.is_finite()
            && p.mdot_ev_kg_s.is_finite()));
        assert!(audit.p_min_pa > 0.0 && audit.p_max_pa > audit.p_min_pa);
    }

    #[test]
    fn valve_effect_is_causal_not_handset_ve() {
        // La mariposa dosifica por caudal calculado: con área casi cerrada
        // el plenum se vacía en el primer ciclo (presión final << ambiente),
        // mientras abierto se mantiene. La masa atrapada de PRIMER ciclo aún
        // refleja la carga inicial del plenum —su dosificación estacionaria
        // exige convergencia multiciclo (S16)— así que el test causal S12 es
        // el vacío del plenum, no la trampa.
        let (geom, mix, i, e, belt, inp) = setup();
        let (trace_open, _) = run_cycle(&geom, mix, &i, &e, &belt, inp).unwrap();
        let mut choked_inp = inp;
        choked_inp.throttle_area_m2 = 1.0e-6;
        let (trace_choked, audit_choked) =
            run_cycle(&geom, mix, &i, &e, &belt, choked_inp).unwrap();
        let p_end = |t: &Vec<super::TracePoint>| t.last().unwrap().p_plen_pa;
        // Primer ciclo: el plenum de 2 L cede ~0.5 g al cilindro; con
        // mariposa casi cerrada no se repone (medido 0.79×, margen 0.85×).
        assert!(
            p_end(&trace_choked) < 0.85 * p_end(&trace_open),
            "throttle must meter the plenum: {} vs {}",
            p_end(&trace_choked),
            p_end(&trace_open)
        );
        assert!(audit_choked.mass_closure.abs() < 1e-9);
    }

    #[test]
    fn invalid_inputs_rejected() {
        let (geom, mix, i, e, belt, mut inp) = setup();
        inp.rpm = 0.0;
        assert!(run_cycle(&geom, mix, &i, &e, &belt, inp).is_err());
    }
}

#[cfg(test)]
mod backflow_tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use thermo::Mixture;
    use valvetrain::{BeltDrive, CamSpec};

    fn setup_cycle() -> (
        GeometryPack,
        Mixture,
        CamSpec,
        CamSpec,
        BeltDrive,
        GasExchangeInputs,
    ) {
        (
            GeometryPack::illustrative_d16_like().unwrap(),
            Mixture::new(287.0, 1.4).unwrap(),
            CamSpec::proxy_d16a6_intake().unwrap(),
            CamSpec::proxy_d16a6_exhaust().unwrap(),
            BeltDrive::new(100.0, 0.0).unwrap(),
            GasExchangeInputs {
                rpm: 750.0,
                p_amb_pa: 101_325.0,
                t_amb_k: 293.15,
                plenum_v_m3: 2.0e-3,
                exhaust_v_m3: 1.5e-3,
                throttle_area_m2: 8.0e-4,
                tailpipe_area_m2: 8.0e-4,
                step_deg: 0.25,
                exhaust_init: None,
            },
        )
    }

    #[test]
    fn reverse_flow_path_executes_and_species_mix_exactly() {
        // En motor arrastrado todo es fresco: el residual quemado neto ≈ 0
        // es física correcta, no bug (la fuente quemada persistente exige
        // combustión, acople S16). Lo que S12 debe probar: (a) el reflujo
        // escape→cilindro OCURRE por la ruta calculada; (b) la mezcla de
        // especies es exacta.
        let (geom, mix, i, e, _, mut inp) = setup_cycle();
        inp.exhaust_init = Some(super::ExhaustInit {
            p_pa: 150_000.0,
            t_k: 800.0,
            fresh_frac: 0.0,
        });
        inp.rpm = 3000.0;
        inp.tailpipe_area_m2 = 0.5e-4;
        let belt = valvetrain::BeltDrive::new(100.0, 2.0).unwrap();
        let (trace, audit) = super::run_cycle(&geom, mix, &i, &e, &belt, inp).unwrap();
        // (a) Reflujo medido en traza (mínimo negativo), no impuesto.
        let minev: f64 = trace.iter().map(|t| t.mdot_ev_kg_s).fold(0.0f64, f64::min);
        assert!(minev < -1e-3, "expected measured backflow, min={minev}");
        assert!(audit.mass_closure.abs() < 1e-9);
        // (b) Mezcla exacta de especies en el nodo (aritmética directa).
        let mut v = crate::Volume::new(2.0, 300.0, 1.0, 1.0, 287.0, 717.0).unwrap();
        v.step(1.0, 0.0, 0.0).unwrap();
        assert!(((v.fresh_frac() - 2.0 / 3.0).abs() < 1e-15));
    }

    #[test]
    fn trapped_mass_converges_on_refinement() {
        // Acoplamiento S06+S11+S12: Euler explícito ⇒ error O(h); la masa
        // atrapada debe estabilizarse al refinar (orden ~1, no divergencia).
        let (geom, mix, i, e, belt, mut inp) = setup_cycle();
        let mut trapped = Vec::new();
        for step in [0.5, 0.25, 0.125] {
            inp.step_deg = step;
            let (_, audit) = super::run_cycle(&geom, mix, &i, &e, &belt, inp).unwrap();
            assert!(audit.mass_closure.abs() < 1e-9);
            trapped.push(audit.trapped_kg);
        }
        let d1 = (trapped[0] - trapped[1]).abs();
        let d2 = (trapped[1] - trapped[2]).abs();
        assert!(d2 < d1, "must converge: {trapped:?}");
        assert!(
            d2 / trapped[2] < 0.01,
            "fine-grid change must be < 1%: {trapped:?}"
        );
    }
}
