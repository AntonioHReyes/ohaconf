pub mod env;
pub mod loader;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Archivo de configuración raíz
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Variables de entorno definidas en el YAML (se fusionan con las del sistema)
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Valores por defecto aplicables a todos los tests
    #[serde(default)]
    pub defaults: Defaults,

    /// Lista de tests de carga
    pub tests: Vec<TestCase>,
}

/// Valores por defecto que se aplican a un TestCase cuando no define su propio valor
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Defaults {
    #[serde(default = "default_requests")]
    pub requests: Option<u64>,

    #[serde(default)]
    pub connections: Option<u64>,

    #[serde(default)]
    pub duration: Option<String>,

    #[serde(default)]
    pub headers: HashMap<String, String>,

    #[serde(default)]
    pub query_per_second: Option<f64>,

    #[serde(default)]
    pub timeout: Option<String>,

    #[serde(default)]
    pub http2: Option<bool>,

    #[serde(default)]
    pub disable_keepalive: Option<bool>,

    #[serde(default)]
    pub insecure: Option<bool>,
}

fn default_requests() -> Option<u64> {
    Some(200)
}

/// Un test de carga individual
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestCase {
    /// Nombre único del test
    pub name: String,

    /// Descripción opcional
    #[serde(default)]
    pub description: Option<String>,

    /// Método HTTP (GET, POST, PUT, DELETE, ...)
    #[serde(default = "default_method")]
    pub method: String,

    /// URL destino (puede contener variables ${VAR})
    pub url: String,

    /// Query params adicionales (se añaden a la URL)
    #[serde(default)]
    pub query_params: HashMap<String, String>,

    /// Headers HTTP (se fusionan con defaults)
    #[serde(default)]
    pub headers: HashMap<String, String>,

    /// Body como string literal
    #[serde(default)]
    pub body: Option<String>,

    /// Body leído desde un archivo
    #[serde(default)]
    pub body_file: Option<String>,

    /// Número de requests (equivale a -n en oha)
    #[serde(default)]
    pub requests: Option<u64>,

    /// Número de conexiones concurrentes (equivale a -c en oha)
    #[serde(default)]
    pub connections: Option<u64>,

    /// Duración del test (equivale a -z en oha, ej: "30s", "2m")
    #[serde(default)]
    pub duration: Option<String>,

    /// Límite de QPS (equivale a -q en oha)
    #[serde(default)]
    pub query_per_second: Option<f64>,

    /// Timeout por request (ej: "5s")
    #[serde(default)]
    pub timeout: Option<String>,

    /// Peso para distribución ponderada (run-weighted)
    #[serde(default = "default_weight")]
    pub weight: f64,

    /// Tags para filtrar tests
    #[serde(default)]
    pub tags: Vec<String>,

    /// Usar HTTP/2
    #[serde(default)]
    pub http2: Option<bool>,

    /// Deshabilitar keep-alive
    #[serde(default)]
    pub disable_keepalive: Option<bool>,

    /// Aceptar certificados inválidos
    #[serde(default)]
    pub insecure: Option<bool>,
}

fn default_method() -> String {
    "GET".to_string()
}

fn default_weight() -> f64 {
    1.0
}

impl TestCase {
    /// Aplica los defaults del Config sobre este TestCase (el TestCase tiene precedencia)
    pub fn with_defaults(&self, defaults: &Defaults) -> TestCase {
        let mut merged = self.clone();

        if merged.requests.is_none() && merged.duration.is_none() {
            merged.requests = defaults.requests;
        }
        if merged.connections.is_none() {
            merged.connections = defaults.connections;
        }
        if merged.duration.is_none() {
            merged.duration = defaults.duration.clone();
        }
        if merged.query_per_second.is_none() {
            merged.query_per_second = defaults.query_per_second;
        }
        if merged.timeout.is_none() {
            merged.timeout = defaults.timeout.clone();
        }
        if merged.http2.is_none() {
            merged.http2 = defaults.http2;
        }
        if merged.disable_keepalive.is_none() {
            merged.disable_keepalive = defaults.disable_keepalive;
        }
        if merged.insecure.is_none() {
            merged.insecure = defaults.insecure;
        }

        // Los headers del TestCase tienen precedencia sobre los defaults
        let mut headers = defaults.headers.clone();
        headers.extend(merged.headers.clone());
        merged.headers = headers;

        merged
    }

    /// Devuelve la URL con query_params añadidos
    pub fn effective_url(&self) -> String {
        if self.query_params.is_empty() {
            return self.url.clone();
        }
        let params: Vec<String> = self
            .query_params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        let separator = if self.url.contains('?') { "&" } else { "?" };
        format!("{}{}{}", self.url, separator, params.join("&"))
    }
}
