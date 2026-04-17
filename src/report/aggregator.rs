use colored::{ColoredString, Colorize};
use crate::report::TestSummary;

/// Padding "visible" (por caracteres Unicode) sin contar escapes ANSI.
/// Rellena `s` a la izquierda (`align=Align::Right`) o a la derecha
/// (`align=Align::Left`) hasta alcanzar `width` columnas visibles.
enum Align { Left, Right }

fn pad_visible(s: &str, width: usize, align: Align) -> String {
    let visible = s.chars().count();
    if visible >= width {
        return s.to_string();
    }
    let pad = " ".repeat(width - visible);
    match align {
        Align::Left => format!("{}{}", s, pad),
        Align::Right => format!("{}{}", pad, s),
    }
}

fn truncate_ellipsis(s: &str, width: usize) -> String {
    let count = s.chars().count();
    if count <= width {
        return s.to_string();
    }
    let mut out: String = s.chars().take(width.saturating_sub(1)).collect();
    out.push('…');
    out
}

/// Imprime una tabla comparativa de los resultados en el terminal
pub fn print_table(summaries: &[TestSummary]) {
    if summaries.is_empty() {
        println!("{}", "No hay resultados para mostrar.".yellow());
        return;
    }

    let col_name = 28usize;
    let col_num = 10usize;
    let total_width = col_name + (col_num + 1) * 8;

    // Encabezado: primero se hace el padding del texto plano, luego se colorea.
    let header = format!(
        "{} {} {} {} {} {} {} {} {}",
        pad_visible("TEST", col_name, Align::Left).bold(),
        pad_visible("REQUESTS", col_num, Align::Right).bold(),
        pad_visible("SUCCESS%", col_num, Align::Right).bold(),
        pad_visible("RPS", col_num, Align::Right).bold(),
        pad_visible("AVG", col_num, Align::Right).bold(),
        pad_visible("P50", col_num, Align::Right).bold(),
        pad_visible("P95", col_num, Align::Right).bold(),
        pad_visible("P99", col_num, Align::Right).bold(),
        pad_visible("ERRORS", col_num, Align::Right).bold(),
    );

    println!();
    println!("{}", "─".repeat(total_width).dimmed());
    println!("{}", header);
    println!("{}", "─".repeat(total_width).dimmed());

    for s in summaries {
        let name_raw = truncate_ellipsis(&s.test_name, col_name);
        let name_padded = pad_visible(&name_raw, col_name, Align::Left).cyan();

        let requests_padded = pad_visible(&s.total_requests.to_string(), col_num, Align::Right)
            .normal();

        let success_raw = format!("{:.1}%", s.success_rate);
        let success_padded_raw = pad_visible(&success_raw, col_num, Align::Right);
        let success_padded: ColoredString = if s.success_rate >= 99.9 {
            success_padded_raw.green()
        } else if s.success_rate >= 95.0 {
            success_padded_raw.yellow()
        } else {
            success_padded_raw.red()
        };

        let rps_padded = pad_visible(&format!("{:.0}", s.requests_per_sec), col_num, Align::Right)
            .normal();
        let avg_padded = pad_visible(&format!("{:.1}ms", s.avg_ms), col_num, Align::Right).normal();
        let p50_padded = pad_visible(&format!("{:.1}ms", s.p50_ms), col_num, Align::Right).normal();
        let p95_padded = pad_visible(&format!("{:.1}ms", s.p95_ms), col_num, Align::Right).normal();
        let p99_padded = pad_visible(&format!("{:.1}ms", s.p99_ms), col_num, Align::Right).normal();

        let errors_raw = s.total_errors.to_string();
        let errors_padded_raw = pad_visible(&errors_raw, col_num, Align::Right);
        let errors_padded: ColoredString = if s.total_errors > 0 {
            errors_padded_raw.red()
        } else {
            errors_padded_raw.green()
        };

        println!(
            "{} {} {} {} {} {} {} {} {}",
            name_padded,
            requests_padded,
            success_padded,
            rps_padded,
            avg_padded,
            p50_padded,
            p95_padded,
            p99_padded,
            errors_padded,
        );
    }

    println!("{}", "─".repeat(total_width).dimmed());
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
