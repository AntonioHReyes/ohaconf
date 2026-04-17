use crate::config::TestCase;

const SENSITIVE_HEADERS: &[&str] = &["authorization", "x-api-key", "x-auth-token", "token", "api-key"];

/// Construye los argumentos para invocar `oha`
pub fn build_args(test: &TestCase, capture_json: bool) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();

    // Número de requests o duración
    if let Some(d) = &test.duration {
        args.extend(["-z".into(), d.clone()]);
    } else if let Some(n) = test.requests {
        args.extend(["-n".into(), n.to_string()]);
    }

    // Conexiones concurrentes
    if let Some(c) = test.connections {
        args.extend(["-c".into(), c.to_string()]);
    }

    // QPS
    if let Some(q) = test.query_per_second {
        args.extend(["-q".into(), q.to_string()]);
    }

    // Método HTTP
    if test.method.to_uppercase() != "GET" {
        args.extend(["-m".into(), test.method.to_uppercase()]);
    }

    // Headers
    for (key, val) in &test.headers {
        args.extend(["-H".into(), format!("{}: {}", key, val)]);
    }

    // Body
    if let Some(body) = &test.body {
        args.extend(["-d".into(), body.clone()]);
    } else if let Some(body_file) = &test.body_file {
        args.extend(["-D".into(), body_file.clone()]);
    }

    // Timeout
    if let Some(t) = &test.timeout {
        args.extend(["-t".into(), t.clone()]);
    }

    // HTTP/2
    if test.http2.unwrap_or(false) {
        args.push("--http2".into());
    }

    // Disable keep-alive
    if test.disable_keepalive.unwrap_or(false) {
        args.push("--disable-keepalive".into());
    }

    // Insecure
    if test.insecure.unwrap_or(false) {
        args.push("--insecure".into());
    }

    // Captura JSON (sin TUI)
    if capture_json {
        args.push("--no-tui".into());
        args.extend(["--output-format".into(), "json".into()]);
    }

    // URL (debe ir al final)
    args.push(test.effective_url());

    args
}

/// Versión redactada para mostrar en logs (oculta headers sensibles)
pub fn build_args_redacted(test: &TestCase, capture_json: bool) -> Vec<String> {
    let mut redacted_test = test.clone();
    for (key, val) in redacted_test.headers.iter_mut() {
        if SENSITIVE_HEADERS.iter().any(|s| key.to_lowercase().contains(s)) {
            *val = "[REDACTED]".to_string();
        }
    }
    build_args(&redacted_test, capture_json)
}

/// Formatea el comando completo como string para display
pub fn format_command(args: &[String]) -> String {
    let escaped: Vec<String> = args
        .iter()
        .map(|a| {
            if a.contains(' ') || a.contains('"') || a.contains('\'') {
                format!("\"{}\"", a.replace('"', "\\\""))
            } else {
                a.clone()
            }
        })
        .collect();
    format!("oha {}", escaped.join(" \\\n    "))
}
