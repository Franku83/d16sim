//! Física del núcleo — S02: stub tipado, sin solvers.
//!
//! Cada futuro submodelo (S06+) implementará su función aquí retornando
//! `Ok` real. Hasta entonces, toda llamada física retorna
//! [`engine_types::EngineError::PhysicsNotImplemented`]: el runner nunca
//! finge un resultado numérico.

use engine_types::EngineError;

/// Estado del crate física (siempre pendiente en S02).
#[must_use]
pub fn status() -> &'static str {
    "physics: no solvers implemented (S02 stub; first solver lands in S06)"
}

/// Placeholder del futuro ciclo cerrado ideal (S09). Existe para fijar la
/// firma y el contrato de error, no para calcular.
pub fn closed_cycle_placeholder() -> Result<(), EngineError> {
    Err(EngineError::PhysicsNotImplemented {
        feature: "closed-cylinder ideal-gas solver",
        needed_by_sprint: "S09",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physics_reports_unimplemented_without_panicking() {
        let err = closed_cycle_placeholder().expect_err("stub must not compute");
        assert!(err.to_string().contains("S09"));
        assert!(status().contains("S02 stub"));
    }
}
