fn observable_chamber_split(
    net: &leak::LeakNetwork,
    totals: &[f64],
) -> (diagnostics::LeakSplit, bool) {
    let mut to_crankcase = 0.0f64;
    let mut to_coolant = 0.0f64;
    for (p, m) in net.ports.iter().zip(totals) {
        if p.from == 0 {
            match p.to {
                1 => to_crankcase += *m,
                2 => to_coolant += *m,
                _ => {}
            }
        }
    }
    let obs_total = to_crankcase + to_coolant;
    let split = if obs_total > 0.0 {
        diagnostics::LeakSplit {
            to_crankcase_01: to_crankcase / obs_total,
            to_exhaust_01: 0.0,
            to_coolant_01: to_coolant / obs_total,
        }
    } else {
        diagnostics::LeakSplit {
            to_crankcase_01: 0.0,
            to_exhaust_01: 0.0,
            to_coolant_01: 0.0,
        }
    };
    let exhaust_channel = net.ports.iter().any(|p| {
        p.from == 0
            && p.to != 1
            && p.to != 2
            && matches!(
                net.domains.get(p.to).map(|d| d.fluid),
                Some(leak::Fluid::Gas { .. })
            )
    });
    (split, exhaust_channel)
}

