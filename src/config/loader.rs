use std::collections::HashSet;
use std::path::Path;
use anyhow::{bail, Context, Result};

use crate::config::{Config, TestCase};
use crate::config::env::expand_test_case;

/// Carga el archivo YAML, valida y expande variables de entorno.
pub fn load(path: &Path) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("No se pudo leer el archivo '{}'", path.display()))?;

    let mut config: Config = serde_yaml::from_str(&content)
        .with_context(|| format!("Error al parsear YAML en '{}'", path.display()))?;

    validate_unique_names(&config.tests)?;

    // Expandir variables de entorno del YAML en sus propios valores primero
    let yaml_env = config.env.clone();

    for test in &mut config.tests {
        // Aplicar defaults antes de expandir variables
        let merged = test.with_defaults(&config.defaults);
        *test = merged;
        expand_test_case(test, &yaml_env)?;
    }

    Ok(config)
}

/// Valida que no haya nombres de tests duplicados
fn validate_unique_names(tests: &[TestCase]) -> Result<()> {
    let mut seen = HashSet::new();
    for test in tests {
        if test.name.trim().is_empty() {
            bail!("Un test tiene el campo 'name' vacío");
        }
        if !seen.insert(test.name.clone()) {
            bail!("Nombre de test duplicado: '{}'", test.name);
        }
    }
    Ok(())
}

/// Verifica que `oha` esté disponible en PATH
pub fn check_oha_in_path() -> Result<()> {
    which::which("oha").map(|_| ()).map_err(|_| {
        anyhow::anyhow!(
            "'oha' no encontrado en PATH.\nInstálalo con: cargo install oha\nO visita: https://github.com/hatoo/oha"
        )
    })
}

/// Valida el archivo sin necesitar que oha esté instalado
pub fn validate_only(path: &Path) -> Result<Config> {
    load(path)
}
