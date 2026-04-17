use colored::Colorize;
use crate::report::TestSummary;

/// Imprime una tabla comparativa de los resultados en el terminal
pub fn print_table(summaries: &[TestSummary]) {
    if summaries.is_empty() {
        println!("{}", "No hay resultados para mostrar.".yellow());
        return;
    }

    let col_name = 28usize;
    let col_num = 10usize;

    // Encabezado
    println!();
    println!("{}", "─".repeat(120).dimmed());
    println!(
        "{:<col_name$} {:>col_num$} {:>col_num$} {:>col_num$} {:>col_num$} {:>col_num$} {:>col_num$} {:>col_num$} {:>col_num$}",
        "TEST".bold(),
        "REQUESTS".bold(),
        "SUCCESS%".bold(),
        "RPS".bold(),
        "AVG".bold(),
        "P50".bold(),
        "P95".bold(),
        "P99".bold(),
        "ERRORS".bold(),
        col_name = col_name,
        col_num = col_num,
    );
    println!("{}", "─".repeat(120).dimmed());

    for s in summaries {
        let errors_str = if s.total_errors > 0 {
            s.total_errors.to_string().red().to_string()
        } else {
            "0".green().to_string()
        };

        let success_str = if s.success_rate >= 99.9 {
            format!("{:.1}%", s.success_rate).green().to_string()
        } else if s.success_rate >= 95.0 {
            format!("{:.1}%", s.success_rate).yellow().to_string()
        } else {
            format!("{:.1}%", s.success_rate).red().to_string()
        };

        let name = if s.test_name.len() > col_name - 1 {
            format!("{}…", &s.test_name[..col_name - 2])
        } else {
            s.test_name.clone()
        };

        println!(
            "{:<col_name$} {:>col_num$} {:>col_num$} {:>col_num$} {:>col_num$} {:>col_num$} {:>col_num$} {:>col_num$} {:>col_num$}",
            name.cyan().to_string(),
            s.total_requests,
            success_str,
            format!("{:.0}", s.requests_per_sec),
            format!("{:.1}ms", s.avg_ms),
            format!("{:.1}ms", s.p50_ms),
            format!("{:.1}ms", s.p95_ms),
            format!("{:.1}ms", s.p99_ms),
            errors_str,
            col_name = col_name,
            col_num = col_num,
        );
    }

    println!("{}", "─".repeat(120).dimmed());
    println!();
}

/// Imprime el resumen de un único test
pub fn print_single(s: &TestSummary) {
    println!();
    println!("{} {}", "●".cyan(), s.test_name.bold());
    println!("  {:<20} {}", "Requests:", s.total_requests);
    println!("  {:<20} {:.1}%", "Success rate:", s.success_rate);
    println!("  {:<20} {:.0} req/s", "Throughput:", s.requests_per_sec);
    println!("  {:<20} {:.1}ms", "Average:", s.avg_ms);
    println!("  {:<20} {:.1}ms", "P50:", s.p50_ms);
    println!("  {:<20} {:.1}ms", "P90:", s.p90_ms);
    println!("  {:<20} {:.1}ms", "P95:", s.p95_ms);
    println!("  {:<20} {:.1}ms", "P99:", s.p99_ms);
    println!("  {:<20} {:.1}ms", "Slowest:", s.slowest_ms);
    println!("  {:<20} {:.1}ms", "Fastest:", s.fastest_ms);
    if s.total_errors > 0 {
        println!("  {:<20} {}", "Errors:".red(), s.total_errors.to_string().red());
    } else {
        println!("  {:<20} {}", "Errors:", "0".green());
    }
    println!();
}
