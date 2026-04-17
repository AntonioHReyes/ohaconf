use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "ohaconf",
    version,
    about = "YAML-driven wrapper for oha HTTP load testing tool",
    long_about = "ohaconf lets you define multiple load tests in a YAML file and run them \
                  interactively or by name, with support for env vars, weighted distribution, \
                  and comparative reports."
)]
pub struct Cli {
    /// Archivo de configuración YAML (por defecto: ./ohaconf.yaml)
    #[arg(short = 'f', long = "config", global = true, default_value = "ohaconf.yaml")]
    pub config: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Lista todos los tests definidos en el archivo de configuración
    List,

    /// Valida el archivo YAML sin ejecutar ningún test
    Validate,

    /// Ejecuta un test (interactivo si no se especifica --test)
    Run {
        /// Nombre del test a ejecutar
        #[arg(short = 't', long = "test")]
        test: Option<String>,

        /// Filtrar por tag
        #[arg(long = "tag")]
        tag: Option<String>,

        /// Mostrar TUI de oha en lugar de capturar output
        #[arg(long = "pass-tui")]
        pass_tui: bool,

        /// Guardar resultado en archivo (JSON)
        #[arg(short = 'o', long = "output")]
        output: Option<String>,

        /// Mostrar el comando oha sin ejecutarlo
        #[arg(long = "dry-run")]
        dry_run: bool,

        /// Mostrar secrets en logs/dry-run (por defecto redactados)
        #[arg(long = "show-secrets")]
        show_secrets: bool,
    },

    /// Ejecuta todos los tests secuencialmente
    RunAll {
        /// Ejecutar en paralelo en lugar de secuencial
        #[arg(long = "parallel")]
        parallel: bool,

        /// Guardar resultados en archivo (JSON)
        #[arg(short = 'o', long = "output")]
        output: Option<String>,

        /// Filtrar por tag
        #[arg(long = "tag")]
        tag: Option<String>,
    },

    /// Ejecuta tests en paralelo distribuidos por su peso (campo weight)
    RunWeighted {
        /// Total de requests a distribuir
        #[arg(short = 'n', long = "requests", conflicts_with = "duration")]
        requests: Option<u64>,

        /// Duración total del test (ej: "60s")
        #[arg(short = 'z', long = "duration", conflicts_with = "requests")]
        duration: Option<String>,

        /// Guardar resultados en archivo (JSON)
        #[arg(short = 'o', long = "output")]
        output: Option<String>,
    },

    /// Imprime el comando oha equivalente sin ejecutarlo (dry-run)
    Export {
        /// Nombre del test a exportar (interactivo si no se especifica)
        #[arg(short = 't', long = "test")]
        test: Option<String>,

        /// Mostrar secrets en los headers
        #[arg(long = "show-secrets")]
        show_secrets: bool,
    },

    /// Lee un archivo de resultados JSON y genera un reporte comparativo
    Report {
        /// Archivo de resultados JSON generado por ohaconf
        #[arg(short = 'i', long = "input")]
        input: String,

        /// Formato de salida: table (default), csv, json
        #[arg(long = "format", default_value = "table")]
        format: String,

        /// Guardar reporte en archivo
        #[arg(short = 'o', long = "output")]
        output: Option<String>,
    },
}
