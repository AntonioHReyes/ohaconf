use std::process::Command;
use anyhow::{bail, Context, Result};
use colored::Colorize;

use crate::config::TestCase;
use crate::report::OhaResult;
use crate::runner::oha_command;

/// Ejecuta oha y retorna el JSON parseado
pub fn run(test: &TestCase) -> Result<OhaResult> {
    let args = oha_command::build_args(test, true);
    let redacted = oha_command::build_args_redacted(test, true);

    println!(
        "{} {}",
        "▶ Ejecutando:".cyan().bold(),
        oha_command::format_command(&redacted).dimmed()
    );

    let output = Command::new("oha")
        .args(&args)
        .output()
        .with_context(|| "No se pudo iniciar oha. Verifica que esté instalado y en PATH.")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("oha terminó con error:\n{}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let oha_json: serde_json::Value = serde_json::from_str(&stdout)
        .with_context(|| format!("No se pudo parsear el JSON de oha:\n{}", stdout))?;

    Ok(OhaResult {
        test_name: test.name.clone(),
        raw: oha_json,
    })
}

/// Ejecuta oha pasando el TUI directamente al terminal (sin capturar output)
pub fn run_passthrough(test: &TestCase) -> Result<()> {
    let args = oha_command::build_args(test, false);
    let redacted = oha_command::build_args_redacted(test, false);

    println!(
        "{} {}",
        "▶ Ejecutando:".cyan().bold(),
        oha_command::format_command(&redacted).dimmed()
    );

    let status = Command::new("oha")
        .args(&args)
        .status()
        .with_context(|| "No se pudo iniciar oha.")?;

    if !status.success() {
        bail!("oha terminó con código de error: {}", status);
    }

    Ok(())
}

/// Ejecuta múltiples tests en paralelo usando threads
pub fn run_parallel(tests: &[TestCase]) -> Vec<(String, Result<OhaResult>)> {
    use std::sync::{Arc, Mutex};
    use std::thread;

    let results: Arc<Mutex<Vec<(String, Result<OhaResult>)>>> =
        Arc::new(Mutex::new(Vec::new()));

    let handles: Vec<_> = tests
        .iter()
        .map(|test| {
            let test = test.clone();
            let results = Arc::clone(&results);
            thread::spawn(move || {
                let name = test.name.clone();
                let result = run(&test);
                results.lock().unwrap().push((name, result));
            })
        })
        .collect();

    for handle in handles {
        let _ = handle.join();
    }

    let mut out = results.lock().unwrap().drain(..).collect::<Vec<_>>();
    // Ordenar por nombre para output determinista
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}
