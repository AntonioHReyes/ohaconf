use anyhow::{bail, Result};
use dialoguer::{theme::ColorfulTheme, Select};

use crate::config::TestCase;

/// Selecciona un test de forma interactiva con un menú
pub fn interactive_select<'a>(tests: &'a [TestCase]) -> Result<&'a TestCase> {
    if tests.is_empty() {
        bail!("No hay tests disponibles en el archivo de configuración.");
    }

    let items: Vec<String> = tests
        .iter()
        .map(|t| {
            let desc = t
                .description
                .as_deref()
                .unwrap_or("");
            let tags = if t.tags.is_empty() {
                String::new()
            } else {
                format!(" [{}]", t.tags.join(", "))
            };
            let method = &t.method;
            if desc.is_empty() {
                format!("{:<30} {} {}{}", t.name, method, t.url, tags)
            } else {
                format!("{:<30} {} {}  — {}{}", t.name, method, t.url, desc, tags)
            }
        })
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Selecciona un test")
        .items(&items)
        .default(0)
        .interact()
        .map_err(|e| anyhow::anyhow!("Error en el selector: {}", e))?;

    Ok(&tests[selection])
}

/// Selecciona múltiples tests de forma interactiva
pub fn interactive_select_multiple<'a>(tests: &'a [TestCase]) -> Result<Vec<&'a TestCase>> {
    use dialoguer::MultiSelect;

    if tests.is_empty() {
        bail!("No hay tests disponibles.");
    }

    let items: Vec<String> = tests
        .iter()
        .map(|t| {
            let tags = if t.tags.is_empty() {
                String::new()
            } else {
                format!(" [{}]", t.tags.join(", "))
            };
            format!("{} ({}){}", t.name, t.method, tags)
        })
        .collect();

    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Selecciona tests (SPACE para marcar, ENTER para confirmar)")
        .items(&items)
        .interact()
        .map_err(|e| anyhow::anyhow!("Error en el selector: {}", e))?;

    if selections.is_empty() {
        bail!("No se seleccionó ningún test.");
    }

    Ok(selections.iter().map(|&i| &tests[i]).collect())
}

/// Filtra tests por nombre exacto
pub fn find_by_name<'a>(tests: &'a [TestCase], name: &str) -> Result<&'a TestCase> {
    tests
        .iter()
        .find(|t| t.name == name)
        .ok_or_else(|| anyhow::anyhow!("Test '{}' no encontrado. Usa 'ohaconf list' para ver los disponibles.", name))
}

/// Filtra tests por tag
pub fn filter_by_tag<'a>(tests: &'a [TestCase], tag: &str) -> Vec<&'a TestCase> {
    tests
        .iter()
        .filter(|t| t.tags.iter().any(|tg| tg == tag))
        .collect()
}
