pub mod aggregator;
pub mod writer;

use serde::{Deserialize, Serialize};

/// Resultado crudo de una ejecución de oha
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OhaResult {
    pub test_name: String,
    pub raw: serde_json::Value,
}

/// Métricas resumidas extraídas del JSON de oha
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSummary {
    pub test_name: String,
    pub total_requests: u64,
    pub success_rate: f64,
    pub requests_per_sec: f64,
    pub avg_ms: f64,
    pub p50_ms: f64,
    pub p90_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub slowest_ms: f64,
    pub fastest_ms: f64,
    pub total_errors: u64,
}

/// Archivo de resultados persistidos
#[derive(Debug, Serialize, Deserialize)]
pub struct ResultsFile {
    pub timestamp: String,
    pub config_file: String,
    pub results: Vec<OhaResult>,
}

impl OhaResult {
    /// Extrae las métricas clave del JSON raw de oha
    pub fn to_summary(&self) -> TestSummary {
        let s = &self.raw;

        let summary = &s["summary"];
        let percentiles = &s["latencyPercentiles"];
        let status_codes = &s["statusCodeDistribution"];

        // Total requests = suma de todos los status codes registrados
        let total_requests: u64 = if let Some(obj) = status_codes.as_object() {
            obj.values().filter_map(|v| v.as_u64()).sum()
        } else {
            0
        };

        let success_rate = summary["successRate"].as_f64().unwrap_or(0.0) * 100.0;
        let requests_per_sec = summary["requestsPerSec"].as_f64().unwrap_or(0.0);
        let avg_ms = to_ms(summary["average"].as_f64().unwrap_or(0.0));
        let slowest_ms = to_ms(summary["slowest"].as_f64().unwrap_or(0.0));
        let fastest_ms = to_ms(summary["fastest"].as_f64().unwrap_or(0.0));

        // Percentiles desde latencyPercentiles (objeto plano: {"p50": ..., "p90": ...})
        let p50_ms = to_ms(percentiles["p50"].as_f64().unwrap_or(0.0));
        let p90_ms = to_ms(percentiles["p90"].as_f64().unwrap_or(0.0));
        let p95_ms = to_ms(percentiles["p95"].as_f64().unwrap_or(0.0));
        let p99_ms = to_ms(percentiles["p99"].as_f64().unwrap_or(0.0));

        // Errores = requests con status != 2xx
        let total_success: u64 = if let Some(obj) = status_codes.as_object() {
            obj.iter()
                .filter(|(k, _)| k.starts_with('2'))
                .filter_map(|(_, v)| v.as_u64())
                .sum()
        } else {
            0
        };
        let total_errors = total_requests.saturating_sub(total_success);

        TestSummary {
            test_name: self.test_name.clone(),
            total_requests,
            success_rate,
            requests_per_sec,
            avg_ms,
            p50_ms,
            p90_ms,
            p95_ms,
            p99_ms,
            slowest_ms,
            fastest_ms,
            total_errors,
        }
    }
}

fn to_ms(secs: f64) -> f64 {
    (secs * 1000.0 * 100.0).round() / 100.0
}
