use std::collections::HashMap;
use anyhow::{bail, Result};

/// Expande variables ${VAR} o $VAR en un string.
/// Orden de precedencia: sistema (std::env) > yaml_env
pub fn expand(input: &str, yaml_env: &HashMap<String, String>) -> Result<String> {
    let result = shellexpand::env_with_context(input, |var| {
        // 1. Variable del sistema
        if let Ok(val) = std::env::var(var) {
            return Ok(Some(val));
        }
        // 2. Variable definida en el YAML
        if let Some(val) = yaml_env.get(var) {
            // El valor del YAML puede a su vez referenciar una var del sistema
            let expanded = shellexpand::env(val)
                .map(|v| v.into_owned())
                .unwrap_or_else(|_| val.clone());
            return Ok(Some(expanded));
        }
        // Variable no encontrada → retornamos error con nombre de la variable
        Err(format!("variable de entorno '{}' no definida", var))
    });

    match result {
        Ok(s) => Ok(s.into_owned()),
        Err(e) => bail!("{}", e),
    }
}

/// Expande variables en todos los campos de un TestCase
pub fn expand_test_case(
    test: &mut crate::config::TestCase,
    yaml_env: &HashMap<String, String>,
) -> Result<()> {
    test.url = expand(&test.url, yaml_env)
        .map_err(|e| anyhow::anyhow!("test '{}' → url: {}", test.name, e))?;

    for val in test.headers.values_mut() {
        *val = expand(val, yaml_env)
            .map_err(|e| anyhow::anyhow!("test '{}' → headers: {}", test.name, e))?;
    }

    for val in test.query_params.values_mut() {
        *val = expand(val, yaml_env)
            .map_err(|e| anyhow::anyhow!("test '{}' → query_params: {}", test.name, e))?;
    }

    if let Some(body) = &test.body {
        test.body = Some(
            expand(body, yaml_env)
                .map_err(|e| anyhow::anyhow!("test '{}' → body: {}", test.name, e))?,
        );
    }

    if let Some(bf) = &test.body_file {
        test.body_file = Some(
            expand(bf, yaml_env)
                .map_err(|e| anyhow::anyhow!("test '{}' → body_file: {}", test.name, e))?,
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_from_system_env() {
        // SAFETY: solo en tests, sin threads concurrentes que lean esta variable
        unsafe { std::env::set_var("_TEST_OHA_VAR", "hello") };
        let result = expand("${_TEST_OHA_VAR}/world", &HashMap::new()).unwrap();
        assert_eq!(result, "hello/world");
    }

    #[test]
    fn expand_from_yaml_env() {
        let mut env = HashMap::new();
        env.insert("BASE_URL".to_string(), "http://localhost:8080".to_string());
        let result = expand("${BASE_URL}/api", &env).unwrap();
        assert_eq!(result, "http://localhost:8080/api");
    }

    #[test]
    fn expand_missing_var_errors() {
        let result = expand("${_MISSING_XYZ_VAR}", &HashMap::new());
        assert!(result.is_err());
    }
}
