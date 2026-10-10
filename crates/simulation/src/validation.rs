//! Validación de esquema de datos S01 (sin física, sin red).

use engine_types::EngineError;

/// Valida `data/d16y7/` contra los contratos de S01. Retorna la lista de
/// problemas; vacía = válido. Todo problema es `SchemaValidation`, nunca
/// un valor inventado que tape el hueco.
pub fn validate_data_dir(data_dir: &std::path::Path) -> Result<Vec<String>, EngineError> {
    let mut issues: Vec<String> = vec![];
    let d16y7 = data_dir.join("d16y7");

    // 1. reference_variant.yaml: campos congelados mínimos.
    let variant_path = d16y7.join("reference_variant.yaml");
    match std::fs::read_to_string(&variant_path) {
        Ok(text) => {
            for required in ["variant_id:", "status:", "transmission:", "obd_generation:"] {
                if !text.contains(required) {
                    issues.push(format!(
                        "{}: missing required field `{required}`",
                        variant_path.display()
                    ));
                }
            }
            if !text.contains("PENDING") && !text.contains("REQUIRES_VERIFICATION") {
                issues.push(format!(
                    "{}: no PENDING/REQUIRES_VERIFICATION marker; S01 forbids confirmed numerics",
                    variant_path.display()
                ));
            }
        }
        Err(e) => {
            return Err(EngineError::Io {
                path: variant_path.display().to_string(),
                source: e,
            });
        }
    }

    // 2. CSVs: cabecera + columna de estado no vacía por fila de datos.
    for csv in [
        "D16Y7_OEM_SPECS.csv",
        "OEM_CATALOG_1997_DX_5MT.csv",
        "unverified_values.csv",
        "parts_inventory.csv",
    ] {
        let path = d16y7.join(csv);
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                let mut lines = text.lines();
                match lines.next() {
                    Some(header) => {
                        if !header.contains("parameter_id")
                            && !header.contains("part_number")
                            && !header.contains("system")
                        {
                            issues.push(format!(
                                "{}: unexpected header (expected parameter_id/part_number/system)",
                                path.display()
                            ));
                        }
                    }
                    None => issues.push(format!("{}: empty file", path.display())),
                }
                for (i, line) in lines.enumerate() {
                    if line.trim().is_empty() {
                        continue;
                    }
                    if line.split(',').count() < 3 {
                        issues.push(format!(
                            "{}:{}: fewer than 3 columns",
                            path.display(),
                            i + 2
                        ));
                    }
                }
            }
            Err(e) => {
                return Err(EngineError::Io {
                    path: path.display().to_string(),
                    source: e,
                });
            }
        }
    }

    // 3. Specs: cada fila de dato debe llevar etiqueta de evidencia.
    let specs_path = d16y7.join("D16Y7_OEM_SPECS.csv");
    match std::fs::read_to_string(&specs_path) {
        Ok(text) => {
            for (i, line) in text.lines().enumerate().skip(1) {
                if line.trim().is_empty() {
                    continue;
                }
                if !line.contains("PENDING_VERIFICATION")
                    && !line.contains("REQUIRES_VERIFICATION")
                    && !line.contains("IDENTITY_ONLY")
                    && !line.contains("DISPUTED")
                {
                    issues.push(format!(
                        "{}:{}: data row without evidence label",
                        specs_path.display(),
                        i + 1
                    ));
                }
            }
        }
        Err(e) => {
            return Err(EngineError::Io {
                path: specs_path.display().to_string(),
                source: e,
            });
        }
    }

    Ok(issues)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn repo_data_dir() -> std::path::PathBuf {
        let mut dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dir.pop();
        dir.pop();
        dir.join("data")
    }

    #[test]
    fn repo_data_passes_schema_validation() {
        let issues = validate_data_dir(&repo_data_dir()).unwrap();
        assert!(issues.is_empty(), "schema issues: {issues:?}");
    }

    #[test]
    fn missing_variant_is_io_error_not_panic() {
        let err = validate_data_dir(std::path::Path::new("/nonexistent-data-dir"))
            .expect_err("must fail on missing dir");
        assert!(matches!(err, EngineError::Io { .. }));
    }
}
