mod cli;
mod config;
mod report;
mod runner;
mod selector;
mod weights;

use std::path::Path;
use anyhow::Result;
use clap::Parser;
use colored::Colorize;

use cli::{Cli, Commands};
use config::loader;
use report::{aggregator, writer};
use runner::{executor, oha_command};

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let config_path = Path::new(&cli.config);

    match cli.command {
        Commands::List => cmd_list(config_path),
        Commands::Validate => cmd_validate(config_path),
        Commands::Run { test, tag, pass_tui, capture, output, dry_run, show_secrets } => {
            cmd_run(config_path, test, tag, pass_tui, capture, output, dry_run, show_secrets)
        }
        Commands::RunAll { parallel, output, tag } => {
            cmd_run_all(config_path, parallel, output, tag)
        }
        Commands::RunWeighted { requests, duration, output } => {
            cmd_run_weighted(config_path, requests, duration, output)
        }
        Commands::Export { test, show_secrets } => cmd_export(config_path, test, show_secrets),
        Commands::Report { input, format, output } => cmd_report(&input, &format, output),
    }
}

// ── list ─────────────────────────────────────────────────────────────────────

fn cmd_list(config_path: &Path) -> Result<()> {
    let config = loader::load(config_path)?;

    println!();
    println!("{} {}", "📄 Config:".dimmed(), config_path.display());
    println!("{} {} tests\n", "📋 Total:".dimmed(), config.tests.len());

    let col_name = 28usize;
    let col_method = 8usize;
    let col_url = 45usize;
    let col_tags = 20usize;

    println!(
        "{:<col_name$} {:<col_method$} {:<col_url$} {}",
        "NAME".bold(),
        "METHOD".bold(),
        "URL".bold(),
        "TAGS".bold(),
        col_name = col_name,
        col_method = col_method,
        col_url = col_url,
    );
    println!("{}", "─".repeat(col_name + col_method + col_url + col_tags + 3).dimmed());

    for t in &config.tests {
        let tags = t.tags.join(", ");
        let name = if t.name.len() > col_name - 1 {
            format!("{}…", &t.name[..col_name - 2])
        } else {
            t.name.clone()
        };
        let url = if t.url.len() > col_url - 1 {
            format!("{}…", &t.url[..col_url - 2])
        } else {
            t.url.clone()
        };

        println!(
            "{:<col_name$} {:<col_method$} {:<col_url$} {}",
            name.cyan(),
            t.method.yellow(),
            url,
            tags.dimmed(),
            col_name = col_name,
            col_method = col_method,
            col_url = col_url,
        );

        if let Some(desc) = &t.description {
            println!("{:<col_name$}   {}", "", desc.dimmed(), col_name = col_name);
        }
    }

    println!();
    Ok(())
}

// ── validate ──────────────────────────────────────────────────────────────────

fn cmd_validate(config_path: &Path) -> Result<()> {
    loader::validate_only(config_path)?;
    println!("{} '{}' es válido.", "✓".green().bold(), config_path.display());
    Ok(())
}

// ── run ───────────────────────────────────────────────────────────────────────

fn cmd_run(
    config_path: &Path,
    test_name: Option<String>,
    tag: Option<String>,
    pass_tui: bool,
    capture: bool,
    output: Option<String>,
    dry_run: bool,
    show_secrets: bool,
) -> Result<()> {
    loader::check_oha_in_path()?;
    let config = loader::load(config_path)?;

    let test = if let Some(name) = test_name {
        selector::find_by_name(&config.tests, &name)?.clone()
    } else if let Some(tag_val) = tag {
        let filtered = selector::filter_by_tag(&config.tests, &tag_val);
        if filtered.is_empty() {
            anyhow::bail!("No hay tests con el tag '{}'.", tag_val);
        }
        if filtered.len() == 1 {
            filtered[0].clone()
        } else {
            let owned: Vec<_> = filtered.into_iter().cloned().collect();
            selector::interactive_select(&owned)?.clone()
        }
    } else {
        selector::interactive_select(&config.tests)?.clone()
    };

    if dry_run {
        let args = if show_secrets {
            oha_command::build_args(&test, true)
        } else {
            oha_command::build_args_redacted(&test, true)
        };
        println!("\n{}\n", oha_command::format_command(&args).cyan());
        return Ok(());
    }

    // Modo TUI por defecto para un solo test, salvo que el usuario pida capturar
    // (--capture) o guardar el resultado (--output), que requieren JSON.
    let needs_capture = capture || output.is_some();
    let use_tui = pass_tui || !needs_capture;

    if use_tui {
        executor::run_passthrough(&test)?;
        return Ok(());
    }

    let result = executor::run(&test)?;
    let summary = result.to_summary();
    aggregator::print_single(&summary);

    if let Some(out) = output {
        writer::save_results_json(&[result], &config_path.to_string_lossy(), Path::new(&out))?;
    }

    Ok(())
}

// ── run-all ───────────────────────────────────────────────────────────────────

fn cmd_run_all(
    config_path: &Path,
    parallel: bool,
    output: Option<String>,
    tag: Option<String>,
) -> Result<()> {
    loader::check_oha_in_path()?;
    let config = loader::load(config_path)?;

    let tests: Vec<_> = if let Some(tag_val) = tag {
        config.tests
            .iter()
            .filter(|t| t.tags.iter().any(|tg| tg == &tag_val))
            .cloned()
            .collect()
    } else {
        config.tests.clone()
    };

    if tests.is_empty() {
        anyhow::bail!("No hay tests que ejecutar.");
    }

    let results: Vec<_>;

    if parallel {
        println!("{} {} tests en paralelo…\n", "▶".cyan().bold(), tests.len());
        let raw = executor::run_parallel(&tests);
        results = raw
            .into_iter()
            .filter_map(|(_, r)| r.ok())
            .collect();
    } else {
        println!("{} {} tests secuencialmente…\n", "▶".cyan().bold(), tests.len());
        let mut collected = vec![];
        for test in &tests {
            match executor::run(test) {
                Ok(r) => collected.push(r),
                Err(e) => eprintln!("{} '{}': {}", "✗".red(), test.name, e),
            }
        }
        results = collected;
    }

    let summaries: Vec<_> = results.iter().map(|r| r.to_summary()).collect();
    aggregator::print_table(&summaries);

    if let Some(out) = output {
        writer::save_results_json(&results, &config_path.to_string_lossy(), Path::new(&out))?;
    }

    Ok(())
}

// ── run-weighted ──────────────────────────────────────────────────────────────

fn cmd_run_weighted(
    config_path: &Path,
    requests: Option<u64>,
    duration: Option<String>,
    output: Option<String>,
) -> Result<()> {
    loader::check_oha_in_path()?;
    let config = loader::load(config_path)?;

    if config.tests.is_empty() {
        anyhow::bail!("No hay tests definidos.");
    }

    let tests_to_run: Vec<_> = if let Some(total) = requests {
        println!(
            "{} {} requests distribuidos por peso entre {} tests…\n",
            "▶".cyan().bold(),
            total,
            config.tests.len()
        );
        weights::apply_weights(&config.tests, total)
    } else if let Some(dur) = duration {
        println!(
            "{} {} (duración) distribuidos por peso entre {} tests…\n",
            "▶".cyan().bold(),
            dur,
            config.tests.len()
        );
        config.tests
            .iter()
            .map(|t| {
                let mut tc = t.clone();
                tc.duration = Some(dur.clone());
                tc.requests = None;
                tc
            })
            .collect()
    } else {
        anyhow::bail!("Debes especificar --requests o --duration.");
    };

    let raw = executor::run_parallel(&tests_to_run);
    let results: Vec<_> = raw.into_iter().filter_map(|(_, r)| r.ok()).collect();
    let summaries: Vec<_> = results.iter().map(|r| r.to_summary()).collect();
    aggregator::print_table(&summaries);

    if let Some(out) = output {
        writer::save_results_json(&results, &config_path.to_string_lossy(), Path::new(&out))?;
    }

    Ok(())
}

// ── export ────────────────────────────────────────────────────────────────────

fn cmd_export(config_path: &Path, test_name: Option<String>, show_secrets: bool) -> Result<()> {
    let config = loader::load(config_path)?;

    let test = if let Some(name) = test_name {
        selector::find_by_name(&config.tests, &name)?.clone()
    } else {
        selector::interactive_select(&config.tests)?.clone()
    };

    let args = if show_secrets {
        oha_command::build_args(&test, true)
    } else {
        oha_command::build_args_redacted(&test, true)
    };

    println!("\n{}\n", oha_command::format_command(&args).cyan());
    Ok(())
}

// ── report ────────────────────────────────────────────────────────────────────

fn cmd_report(input: &str, format: &str, output: Option<String>) -> Result<()> {
    let results_file = writer::load_results_json(Path::new(input))?;

    println!(
        "\n{} {} — {}\n",
        "📊 Reporte:".bold(),
        input,
        results_file.timestamp.dimmed()
    );

    let summaries: Vec<_> = results_file.results.iter().map(|r| r.to_summary()).collect();

    match format {
        "table" | "text" => aggregator::print_table(&summaries),
        "json" => {
            let json = serde_json::to_string_pretty(&summaries)?;
            if let Some(out) = output {
                std::fs::write(&out, &json)?;
                println!("✓ Reporte JSON guardado en '{}'", out);
            } else {
                println!("{}", json);
            }
            return Ok(());
        }
        "csv" => {
            let out_path = output.as_deref().unwrap_or("report.csv");
            writer::save_summary_csv(&summaries, Path::new(out_path))?;
            return Ok(());
        }
        _ => anyhow::bail!("Formato desconocido: '{}'. Usa: table, json, csv", format),
    }

    if let Some(out) = output {
        let json = serde_json::to_string_pretty(&summaries)?;
        std::fs::write(&out, json)?;
        println!("✓ Guardado en '{}'", out);
    }

    Ok(())
}
