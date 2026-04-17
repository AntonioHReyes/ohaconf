use std::path::Path;
use anyhow::{Context, Result};
use chrono::Utc;

use crate::report::{OhaResult, ResultsFile, TestSummary};

/// Guarda los resultados crudos en un archivo JSON
pub fn save_results_json(
    results: &[OhaResult],
    config_file: &str,
    output_path: &Path,
) -> Result<()> {
    let file = ResultsFile {
        timestamp: Utc::now().to_rfc3339(),
        config_file: config_file.to_string(),
        results: results.to_vec(),
    };

    let json = serde_json::to_string_pretty(&file)
        .context("Error al serializar resultados a JSON")?;

    std::fs::write(output_path, json)
        .with_context(|| format!("No se pudo escribir '{}'", output_path.display()))?;

    println!("✓ Resultados guardados en '{}'", output_path.display());
    Ok(())
}

/// Guarda un resumen en CSV
pub fn save_summary_csv(summaries: &[TestSummary], output_path: &Path) -> Result<()> {
    let mut wtr = csv::Writer::from_path(output_path)
        .with_context(|| format!("No se pudo crear CSV en '{}'", output_path.display()))?;

    wtr.write_record([
        "test_name",
        "requests",
        "success_rate",
        "rps",
        "avg_ms",
        "p50_ms",
        "p90_ms",
        "p95_ms",
        "p99_ms",
        "slowest_ms",
        "fastest_ms",
        "errors",
    ])?;

    for s in summaries {
        wtr.write_record([
            &s.test_name,
            &s.total_requests.to_string(),
            &format!("{:.2}", s.success_rate),
            &format!("{:.2}", s.requests_per_sec),
            &format!("{:.2}", s.avg_ms),
            &format!("{:.2}", s.p50_ms),
            &format!("{:.2}", s.p90_ms),
            &format!("{:.2}", s.p95_ms),
            &format!("{:.2}", s.p99_ms),
            &format!("{:.2}", s.slowest_ms),
            &format!("{:.2}", s.fastest_ms),
            &s.total_errors.to_string(),
        ])?;
    }

    wtr.flush()?;
    println!("✓ CSV guardado en '{}'", output_path.display());
    Ok(())
}

/// Carga un archivo de resultados JSON
pub fn load_results_json(path: &Path) -> Result<ResultsFile> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("No se pudo leer '{}'", path.display()))?;

    serde_json::from_str(&content)
        .with_context(|| format!("Error al parsear JSON de resultados en '{}'", path.display()))
}
