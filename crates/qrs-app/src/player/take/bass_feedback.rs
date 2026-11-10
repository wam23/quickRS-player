use qrs_analysis::take::BarResult;

pub(super) fn timing_line(timing: &str, is_bass: bool, notes: usize, spread_ms: f64) -> String {
    if is_bass && notes >= 3 {
        format!("{timing}  ·  pocket spread {spread_ms:.0} ms")
    } else {
        timing.to_owned()
    }
}

pub(super) fn bar_tooltip(bar: &BarResult, is_bass: bool) -> String {
    let summary = match bar.accuracy() {
        Some(a) => format!("bar {}: {}%", bar.number, (a * 100.0).round() as i64),
        None => format!("bar {}: nothing checked", bar.number),
    };
    match bar.mean_offset_ms {
        Some(offset) => format!(
            "{summary}  ·  {} {offset:+.0} ms",
            if is_bass { "pocket" } else { "timing" }
        ),
        None => summary,
    }
}

#[cfg(test)]
mod tests;
