//! Conversiones de visualización (UI/CLI). **Nunca tipos del core.**
//!
//! Cada función es `f64 → f64` pura y documenta su factor exacto. El core
//! solo acepta SI mediante los constructores de los módulos tipados.

/// psi → Pa (exacto por definición: 6894.757293168 Pa).
#[must_use]
pub fn psi_to_pa(psi: f64) -> f64 {
    psi * 6_894.757_293_168
}

/// Pa → psi.
#[must_use]
pub fn pa_to_psi(pa: f64) -> f64 {
    pa / 6_894.757_293_168
}

/// bar → Pa.
#[must_use]
pub fn bar_to_pa(bar: f64) -> f64 {
    bar * 100_000.0
}

/// Pa → bar.
#[must_use]
pub fn pa_to_bar(pa: f64) -> f64 {
    pa / 100_000.0
}

/// °C → K.
#[must_use]
pub fn celsius_to_kelvin(c: f64) -> f64 {
    c + 273.15
}

/// K → °C.
#[must_use]
pub fn kelvin_to_celsius(k: f64) -> f64 {
    k - 273.15
}

/// N·m → lb·ft.
#[must_use]
pub fn nm_to_lbfft(nm: f64) -> f64 {
    nm / 1.355_817_948_331_400_4
}

/// lb·ft → N·m.
#[must_use]
pub fn lbfft_to_nm(lbft: f64) -> f64 {
    lbft * 1.355_817_948_331_400_4
}

/// kW → hp (mecánico, 745.7 W).
#[must_use]
pub fn kw_to_hp(kw: f64) -> f64 {
    kw * 1_000.0 / 745.7
}

/// hp → kW.
#[must_use]
pub fn hp_to_kw(hp: f64) -> f64 {
    hp * 745.7 / 1_000.0
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::mechanics::Rpm;
    use crate::pressure::PressureAbs;
    use crate::thermal::Kelvin;

    fn lcg(state: &mut u64) -> f64 {
        *state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((*state >> 11) as f64) / ((1u64 << 53) as f64)
    }

    #[test]
    fn display_roundtrips_hold() {
        let mut s = 7;
        for _ in 0..2000 {
            let pa = 10_000.0 + lcg(&mut s) * 900_000.0;
            assert!((psi_to_pa(pa_to_psi(pa)) - pa).abs() <= 1e-9 * pa);
            assert!((bar_to_pa(pa_to_bar(pa)) - pa).abs() <= 1e-9 * pa);
            let k = 200.0 + lcg(&mut s) * 800.0;
            assert!((celsius_to_kelvin(kelvin_to_celsius(k)) - k).abs() < 1e-9);
            let nm = lcg(&mut s) * 500.0;
            assert!((lbfft_to_nm(nm_to_lbfft(nm)) - nm).abs() <= 1e-12 * nm.max(1.0));
            let kw = lcg(&mut s) * 200.0;
            assert!((hp_to_kw(kw_to_hp(kw)) - kw).abs() <= 1e-12 * kw.max(1.0));
        }
    }

    #[test]
    fn display_agrees_with_typed_core() {
        // Simetría/consistencia: la vía visual y la vía tipada coinciden.
        let p = PressureAbs::from_psi_abs(40.0).unwrap();
        assert!((p.as_pa() - psi_to_pa(40.0)).abs() < 1e-9);
        let t = Kelvin::from_kelvin(celsius_to_kelvin(80.0)).unwrap();
        assert!((t.as_kelvin() - 353.15).abs() < 1e-9);
        let rpm = Rpm::from_rpm(3000.0).unwrap();
        assert!((rpm.to_rad_per_s().as_rad_per_s() - 314.159_265_358_979_3).abs() < 1e-9);
        // Chequeo del factor con un valor arbitrario (no es un nominal).
        assert!((nm_to_lbfft(140.0) - 140.0 / 1.355_817_948_331_400_4).abs() < 1e-12);
    }
}
